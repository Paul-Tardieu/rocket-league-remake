"""Checks the RocketSim setup before a long training run: arena loaded, physics sane, speed.

    .venv/bin/python smoke_test.py
"""

import time

import numpy as np

from train import build_env

STEPS = 3000


def main():
    env = build_env()
    env.reset()
    engine = env.rlgym_env.transition_engine
    rng = np.random.default_rng(0)
    n_agents = len(env.agent_map) if hasattr(env, "agent_map") else 2
    lowest_car, lowest_ball = float("inf"), float("inf")

    start = time.perf_counter()
    for _ in range(STEPS):
        actions = rng.integers(0, env.action_space.n, size=(n_agents, 1))
        _, _, done, truncated = env.step(actions)[:4]
        state = engine.state
        lowest_car = min(lowest_car, min(car.physics.position[2] for car in state.cars.values()))
        lowest_ball = min(lowest_ball, state.ball.position[2])
        if np.any(done) or np.any(truncated):
            env.reset()
    elapsed = time.perf_counter() - start

    # Each step repeats the action for 8 physics ticks.
    print(f"{STEPS} steps in {elapsed:.2f}s -> {STEPS * 8 / elapsed:,.0f} physics ticks/s on one core")
    print(f"lowest car z {lowest_car:.1f}, lowest ball z {lowest_ball:.1f}")
    assert lowest_car > 0, "a car went below the floor: collision meshes not loaded"
    assert lowest_ball > 80, "the ball went below the floor: collision meshes not loaded"
    print("OK: arena loaded and physics look sane")


if __name__ == "__main__":
    main()
