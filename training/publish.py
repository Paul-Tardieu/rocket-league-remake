"""Publishes the training progress to https://projects.tardieu.ovh/rocket-league/.

    uv run python publish.py              # every 5 minutes, while training runs next to it
    uv run python publish.py --once       # one snapshot, then exit
    uv run python publish.py --dry-run    # only write data/snapshot.json, push nothing
    uv run python publish.py --site       # push the page itself (site/rocket-league/) to the main branch

Each snapshot is one JSON file built here from the training log and a fresh replay of the latest
checkpoint: learning curves, status, touch/goal stats, replay frames. It is force-pushed as the only
commit of the `data` branch of the site repo, so the repo never grows; the page reads it from
raw.githubusercontent.com, so updates trigger no GitHub Pages build. Nothing else from this folder
is ever uploaded: no checkpoint, no collision mesh.
"""

import argparse
import json
import os
import re
import shutil
import socket
import subprocess
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
DATA = HERE / "data"
REPO = os.environ.get("CARBALL_PUBLISH_REPO", "git@github.com:Paul-Tardieu/projects.git")
MACHINE = os.environ.get("CARBALL_MACHINE", socket.gethostname())
SNAPSHOT_PATH = "rocket-league/snapshot.json"
# rlgym-ppo prints a report every few seconds; a log silent for longer than this means training stopped.
RUNNING_IF_LOG_YOUNGER_THAN = 180
MAX_CURVE_POINTS = 1500
REPORT_FIELDS = {
    "reward": "Policy Reward",
    "entropy": "Policy Entropy",
    "kl": "Mean KL Divergence",
    "steps_per_second": "Overall Steps per Second",
    "timesteps": "Cumulative Timesteps",
}
GIT_IDENTITY = {
    "GIT_AUTHOR_NAME": "carball publisher",
    "GIT_AUTHOR_EMAIL": "publisher@carball.invalid",
    "GIT_COMMITTER_NAME": "carball publisher",
    "GIT_COMMITTER_EMAIL": "publisher@carball.invalid",
}


def parse_reports(log: Path) -> dict:
    """One list per REPORT_FIELDS key, one value per rlgym-ppo iteration report (None for nan)."""
    curves = {key: [] for key in REPORT_FIELDS}
    if not log.exists():
        return curves
    text = log.read_text(errors="ignore")
    for block in re.findall(r"BEGIN ITERATION REPORT-+(.*?)-+END ITERATION REPORT", text, re.S):
        values = {}
        for key, label in REPORT_FIELDS.items():
            match = re.search(rf"{label}: ([-\d.,naif]+)", block)
            values[key] = to_number(match.group(1)) if match else None
        if values["timesteps"] is None:
            continue
        # A restarted run counts from zero again: keep only the latest run.
        if curves["timesteps"] and values["timesteps"] < curves["timesteps"][-1]:
            curves = {key: [] for key in REPORT_FIELDS}
        for key, value in values.items():
            curves[key].append(value)
    return thin(curves)


def to_number(text: str):
    value = float(text.replace(",", ""))
    return None if value != value or value in (float("inf"), float("-inf")) else round(value, 6)


def thin(curves: dict) -> dict:
    """Keeps at most MAX_CURVE_POINTS evenly spaced iterations, always including the last one."""
    n = len(curves["timesteps"])
    if n <= MAX_CURVE_POINTS:
        return curves
    step = n / MAX_CURVE_POINTS
    keep = sorted({int(i * step) for i in range(MAX_CURVE_POINTS)} | {n - 1})
    return {key: [values[i] for i in keep] for key, values in curves.items()}


def latest_checkpoint_steps():
    from replay import latest_checkpoint

    try:
        return int(latest_checkpoint().name)
    except (FileNotFoundError, IndexError):
        return None


def build_snapshot(log: Path, replay: dict | None) -> dict:
    log_age = time.time() - log.stat().st_mtime if log.exists() else None
    return {
        "updated": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "machine": MACHINE,
        "running": log_age is not None and log_age < RUNNING_IF_LOG_YOUNGER_THAN,
        "log_age_seconds": None if log_age is None else round(log_age),
        "curves": parse_reports(log),
        "replay": replay,
    }


def git(*args, cwd: Path, data: bytes | None = None) -> str:
    result = subprocess.run(
        ["git", *args], cwd=cwd, input=data, capture_output=True, check=True, env={**os.environ, **GIT_IDENTITY}
    )
    return result.stdout.decode().strip()


def push_snapshot(payload: bytes):
    """Force-pushes a one-commit `data` branch holding only the snapshot file."""
    store = DATA / "publish.git"
    if not store.exists():
        subprocess.run(["git", "init", "--bare", "--quiet", str(store)], check=True)
    blob = git("hash-object", "-w", "--stdin", cwd=store, data=payload)
    folder, name = SNAPSHOT_PATH.split("/")
    inner = git("mktree", cwd=store, data=f"100644 blob {blob}\t{name}\n".encode())
    root = git("mktree", cwd=store, data=f"040000 tree {inner}\t{folder}\n".encode())
    commit = git("commit-tree", root, "-m", f"Training snapshot from {MACHINE}", cwd=store)
    git("push", "--force", "--quiet", REPO, f"{commit}:refs/heads/data", cwd=store)
    # Old snapshots are unreachable now; drop them so the local store stays small too.
    git("gc", "--prune=now", "--quiet", cwd=store)


def push_site():
    """Copies site/rocket-league/ into the site repo's main branch and pushes it."""
    with tempfile.TemporaryDirectory() as tmp:
        clone = Path(tmp) / "site"
        subprocess.run(["git", "clone", "--quiet", "--depth", "1", REPO, str(clone)], check=True)
        target = clone / "rocket-league"
        shutil.rmtree(target, ignore_errors=True)
        shutil.copytree(HERE / "site" / "rocket-league", target)
        git("add", "rocket-league", cwd=clone)
        if not git("status", "--porcelain", cwd=clone):
            print("page already up to date")
            return
        git("commit", "--quiet", "-m", "Update the Rocket League training page", cwd=clone)
        git("push", "--quiet", "origin", "HEAD", cwd=clone)
        print("page pushed: live in about a minute")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--log", type=Path, default=DATA / "train.log")
    parser.add_argument("--every", type=float, default=300, help="seconds between snapshots")
    parser.add_argument("--replay-seconds", type=float, default=60)
    parser.add_argument("--once", action="store_true")
    parser.add_argument("--dry-run", action="store_true", help="write data/snapshot.json, push nothing")
    parser.add_argument("--site", action="store_true", help="push the page, not a snapshot")
    args = parser.parse_args()

    if args.site:
        push_site()
        return

    replay_cache = DATA / "publish_replay.json"
    replay = json.loads(replay_cache.read_text()) if replay_cache.exists() else None
    while True:
        steps = latest_checkpoint_steps()
        if steps is not None and (replay is None or replay["checkpoint_timesteps"] != steps):
            from replay import record

            print(f"new checkpoint {steps}: recording a {args.replay_seconds:.0f} s replay")
            replay = record(args.replay_seconds, args.log)
            replay.pop("reward_curve")  # the snapshot carries the full curves already
            replay_cache.write_text(json.dumps(replay, separators=(",", ":")))

        snapshot = build_snapshot(args.log, replay)
        payload = json.dumps(snapshot, separators=(",", ":")).encode()
        (DATA / "snapshot.json").write_bytes(payload)
        status = "running" if snapshot["running"] else "stopped"
        iterations = len(snapshot["curves"]["timesteps"])
        if args.dry_run:
            print(f"{status}, {iterations} iterations -> data/snapshot.json ({len(payload) // 1024} KB), not pushed")
        else:
            try:
                push_snapshot(payload)
                print(f"{time.strftime('%H:%M:%S')} {status}, {iterations} iterations, pushed {len(payload) // 1024} KB")
            except subprocess.CalledProcessError as error:
                # A network hiccup must not stop the loop: the next snapshot will try again.
                print(f"push failed: {error.stderr.decode().strip()}")
        if args.once or args.dry_run:
            return
        time.sleep(args.every)


if __name__ == "__main__":
    main()
