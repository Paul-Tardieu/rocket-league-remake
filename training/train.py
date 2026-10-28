"""First PPO training run: a 1v1 self-play agent in RocketSim, through RLGym v2 and RLGym-PPO.

Adapted from the RLGym v2 quickstart (https://rlgym.org/Getting%20Started/quickstart/), switched
to 1v1 and sized for a 16-core machine. Run from this folder, with `collision_meshes/` next to it:

    .venv/bin/python train.py
"""

import os
from pathlib import Path

# Leave a few cores for the learner process and the rest of the machine.
N_PROC = max(1, (os.cpu_count() or 4) - 4)
COLLISION_MESHES = Path(__file__).resolve().parent / "collision_meshes"


def init_rocketsim():
    """Loads the arena collision meshes from this folder.

    RLGym's RocketSimEngine looks for them inside its own package folder and silently ignores a
    failure, which would leave an arena without floor or walls. Loading them explicitly first makes
    a missing folder a loud error instead.
    """
    import RocketSim

    if not (COLLISION_MESHES / "soccar").is_dir():
        raise FileNotFoundError(
            f"{COLLISION_MESHES}/soccar not found: dump it from Rocket League with "
            "https://github.com/ZealanL/RLArenaCollisionDumper (see README)"
        )
    RocketSim.init(str(COLLISION_MESHES))


def build_env():
    init_rocketsim()
    import numpy as np
    from rlgym.api import RLGym
    from rlgym.rocket_league import common_values
    from rlgym.rocket_league.action_parsers import LookupTableAction, RepeatAction
    from rlgym.rocket_league.done_conditions import (
        AnyCondition,
        GoalCondition,
        NoTouchTimeoutCondition,
        TimeoutCondition,
    )
    from rlgym.rocket_league.obs_builders import DefaultObs
    from rlgym.rocket_league.reward_functions import CombinedReward, GoalReward, TouchReward
    from rlgym.rocket_league.sim import RocketSimEngine
    from rlgym.rocket_league.state_mutators import FixedTeamSizeMutator, KickoffMutator, MutatorSequence
    from rlgym_ppo.util import RLGymV2GymWrapper

    team_size = 1
    # The agent picks an action every 8 physics ticks (120 Hz), i.e. 15 decisions per second.
    action_parser = RepeatAction(LookupTableAction(), repeats=8)
    termination = GoalCondition()
    truncation = AnyCondition(
        NoTouchTimeoutCondition(timeout_seconds=30),
        TimeoutCondition(timeout_seconds=300),
    )
    # Scoring is what matters; touching the ball is a small hint to get learning started.
    reward = CombinedReward((GoalReward(), 10), (TouchReward(), 0.1))
    obs_builder = DefaultObs(
        zero_padding=None,
        pos_coef=np.asarray(
            [1 / common_values.SIDE_WALL_X, 1 / common_values.BACK_NET_Y, 1 / common_values.CEILING_Z]
        ),
        ang_coef=1 / np.pi,
        lin_vel_coef=1 / common_values.CAR_MAX_SPEED,
        ang_vel_coef=1 / common_values.CAR_MAX_ANG_VEL,
        boost_coef=1 / 100.0,
    )
    state_mutator = MutatorSequence(
        FixedTeamSizeMutator(blue_size=team_size, orange_size=team_size),
        KickoffMutator(),
    )
    env = RLGym(
        state_mutator=state_mutator,
        obs_builder=obs_builder,
        action_parser=action_parser,
        reward_fn=reward,
        termination_cond=termination,
        truncation_cond=truncation,
        transition_engine=RocketSimEngine(),
    )
    return RLGymV2GymWrapper(env)


if __name__ == "__main__":
    from rlgym_ppo import Learner

    batch = 50_000
    learner = Learner(
        build_env,
        n_proc=N_PROC,
        min_inference_size=max(1, int(round(N_PROC * 0.9))),
        metrics_logger=None,
        ppo_batch_size=batch,
        policy_layer_sizes=[512, 512],
        critic_layer_sizes=[512, 512],
        ts_per_iteration=batch,
        exp_buffer_size=3 * batch,
        ppo_minibatch_size=batch,
        ppo_ent_coef=0.01,
        policy_lr=5e-5,
        critic_lr=5e-5,
        ppo_epochs=1,
        standardize_returns=True,
        standardize_obs=False,
        save_every_ts=1_000_000,
        timestep_limit=1_000_000_000,
        log_to_wandb=False,
    )
    learner.learn()
