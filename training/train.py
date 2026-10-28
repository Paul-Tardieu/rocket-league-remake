"""PPO training: a 1v1 self-play agent in RocketSim, through RLGym v2 and RLGym-PPO.

Adapted from the RLGym v2 quickstart and its "Training an Agent" tutorial
(https://rlgym.org/Rocket%20League/training_an_agent/): 1v1, dense rewards, tutorial-sized learner. Run from this folder, with `collision_meshes/` next to it:

    .venv/bin/python train.py
"""

import os
from pathlib import Path

# Simulation processes. Each one spends much of its time waiting for the policy to pick actions, so
# running more processes than cores keeps both the CPU and the GPU busier. Override with CARBALL_N_PROC.
N_PROC = int(os.environ.get("CARBALL_N_PROC", (os.cpu_count() or 4) * 3 // 2))
COLLISION_MESHES = Path(__file__).resolve().parent / "collision_meshes"
# Hidden layer sizes of the policy network; replay.py must rebuild the same shape to load it.
POLICY_LAYERS = [1024, 1024, 512, 512]
CRITIC_LAYERS = [1024, 1024, 512, 512]


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

    from rewards import InAirReward, SpeedTowardBallReward, VelocityBallToGoalReward

    team_size = 1
    # The agent picks an action every 8 physics ticks (120 Hz), i.e. 15 decisions per second.
    action_parser = RepeatAction(LookupTableAction(), repeats=8)
    termination = GoalCondition()
    truncation = AnyCondition(
        NoTouchTimeoutCondition(timeout_seconds=30),
        TimeoutCondition(timeout_seconds=300),
    )
    # Goals are what matters, but they are far too rare to learn from at first: the dense terms
    # reward every step spent driving at the ball or pushing it towards the opponent's goal.
    reward = CombinedReward(
        (InAirReward(), 0.002),
        (SpeedTowardBallReward(), 0.01),
        (VelocityBallToGoalReward(), 0.1),
        (TouchReward(), 0.1),
        (GoalReward(), 10.0),
    )
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

    batch = 100_000
    learner = Learner(
        build_env,
        n_proc=N_PROC,
        min_inference_size=max(1, int(round(N_PROC * 0.9))),
        metrics_logger=None,
        ppo_batch_size=batch,
        policy_layer_sizes=POLICY_LAYERS,
        critic_layer_sizes=CRITIC_LAYERS,
        ts_per_iteration=batch,
        exp_buffer_size=3 * batch,
        ppo_minibatch_size=batch // 2,
        ppo_ent_coef=0.01,
        policy_lr=2e-4,
        critic_lr=2e-4,
        ppo_epochs=2,
        standardize_returns=True,
        standardize_obs=False,
        save_every_ts=5_000_000,
        n_checkpoints_to_keep=5,
        timestep_limit=1_000_000_000,
        log_to_wandb=False,
    )
    learner.learn()
