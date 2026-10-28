"""Records the latest trained policy playing itself and writes `data/replay.json` for viewing.

    uv run python replay.py [--seconds 60] [--log path/to/train.log]

Also measures how often the trained bot touches the ball compared with random play, which is the
first skill the reward teaches.
"""

import argparse
import json
import re
from pathlib import Path

import numpy as np
import torch
from rlgym_ppo.ppo.discrete_policy import DiscreteFF

from train import POLICY_LAYERS, build_env

HERE = Path(__file__).resolve().parent
CHECKPOINTS = HERE / "data" / "checkpoints"
ACTION_REPEAT_TICKS = 8
TICKS_PER_SECOND = 120


def latest_checkpoint() -> Path:
    runs = sorted(p for p in CHECKPOINTS.iterdir() if p.is_dir())
    if not runs:
        raise FileNotFoundError(f"no training run in {CHECKPOINTS}")
    saves = sorted((p for p in runs[-1].iterdir() if p.name.isdigit()), key=lambda p: int(p.name))
    return saves[-1]


def load_policy(folder: Path, obs_size: int, n_actions: int) -> DiscreteFF:
    policy = DiscreteFF(obs_size, n_actions, POLICY_LAYERS, "cpu")
    policy.load_state_dict(torch.load(folder / "PPO_POLICY.pt", map_location="cpu"))
    policy.eval()
    return policy


def play(env, choose_actions, seconds: float, record: bool):
    """Plays `seconds` of game time; returns recorded frames and touch/goal counts."""
    obs = env.reset()
    frames, touches, goals = [], 0, 0
    for _ in range(int(seconds * TICKS_PER_SECOND / ACTION_REPEAT_TICKS)):
        obs, _, done, truncated, info = env.step(choose_actions(obs))
        state = info["state"]
        touches += sum(car.ball_touches for car in state.cars.values())
        if done:
            goals += 1
        if record:
            frames.append(snapshot(state, goal=bool(done)))
        if done or truncated:
            obs = env.reset()
    return frames, touches, goals


def snapshot(state, goal: bool) -> dict:
    cars = []
    for car in state.cars.values():
        p = car.physics
        cars.append(
            {
                "team": int(car.team_num),
                "pos": [round(float(v), 1) for v in p.position],
                "yaw": round(float(p.yaw), 3),
                "boost": round(float(car.boost_amount), 1),
                "ground": bool(car.on_ground),
            }
        )
    ball = state.ball.position
    return {"ball": [round(float(v), 1) for v in ball], "cars": cars, "goal": goal}


def reward_curve(log: Path) -> list:
    """(timesteps, mean policy reward) per iteration, parsed from the rlgym-ppo console report."""
    if not log.exists():
        return []
    text = log.read_text(errors="ignore")
    rewards = [float(x) for x in re.findall(r"Policy Reward: ([-\d.]+)", text)]
    steps = [int(x.replace(",", "")) for x in re.findall(r"Cumulative Timesteps: ([\d,]+)", text)]
    return [[s, r] for s, r in zip(steps, rewards)]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seconds", type=float, default=60)
    parser.add_argument("--log", type=Path, default=HERE / "data" / "train.log")
    args = parser.parse_args()

    env = build_env()
    folder = latest_checkpoint()
    policy = load_policy(folder, env.observation_space.shape[0], env.action_space.n)
    rng = np.random.default_rng(0)

    def trained(obs):
        with torch.no_grad():
            return np.array([[policy.get_action(o, deterministic=False)[0].item()] for o in obs])

    def random(obs):
        return rng.integers(0, env.action_space.n, size=(len(obs), 1))

    frames, touches, goals = play(env, trained, args.seconds, record=True)
    _, random_touches, random_goals = play(env, random, args.seconds, record=False)
    out = {
        "checkpoint_timesteps": int(folder.name),
        "seconds_per_frame": ACTION_REPEAT_TICKS / TICKS_PER_SECOND,
        "frames": frames,
        "stats": {
            "seconds": args.seconds,
            "trained": {"touches": touches, "goals": goals},
            "random": {"touches": random_touches, "goals": random_goals},
        },
        "reward_curve": reward_curve(args.log),
    }
    payload = json.dumps(out, separators=(",", ":"))
    (HERE / "data" / "replay.json").write_text(payload)
    # "</" would end the <script> tag that embeds the data in the page.
    page = (HERE / "viewer_template.html").read_text().replace("__REPLAY_JSON__", payload.replace("</", "<\\/"))
    path = HERE / "data" / "replay.html"
    path.write_text(page)
    print(f"checkpoint {folder.name} steps -> {path} (open it in a browser)")
    print(f"{args.seconds:.0f}s of play: trained bot {touches} touches / {goals} goals, random {random_touches} / {random_goals}")


if __name__ == "__main__":
    main()
