"""Dense shaping rewards, following the ideas of RLGym's "Training an Agent" tutorial.

Goals and touches are too rare for a random policy to ever stumble on: with only those, the first
run learned nothing (entropy stayed at its maximum). These rewards give a signal on every step that
points towards them: drive at the ball, then push the ball towards the opponent's goal.
"""

from typing import Any, Dict, List

import numpy as np
from rlgym.api import AgentID, RewardFunction
from rlgym.rocket_league import common_values
from rlgym.rocket_league.api import GameState


class _PerAgentReward(RewardFunction[AgentID, GameState, float]):
    def reset(self, agents: List[AgentID], initial_state: GameState, shared_info: Dict[str, Any]) -> None:
        pass

    def get_rewards(
        self,
        agents: List[AgentID],
        state: GameState,
        is_terminated: Dict[AgentID, bool],
        is_truncated: Dict[AgentID, bool],
        shared_info: Dict[str, Any],
    ) -> Dict[AgentID, float]:
        return {agent: self.reward(agent, state) for agent in agents}

    def reward(self, agent: AgentID, state: GameState) -> float:
        raise NotImplementedError


class SpeedTowardBallReward(_PerAgentReward):
    """Car speed in the direction of the ball, as a fraction of top speed; never negative."""

    def reward(self, agent, state):
        car = state.cars[agent].physics
        to_ball = state.ball.position - car.position
        distance = np.linalg.norm(to_ball)
        if distance == 0:
            return 0.0
        speed = float(np.dot(car.linear_velocity, to_ball / distance))
        return max(0.0, speed / common_values.CAR_MAX_SPEED)


class VelocityBallToGoalReward(_PerAgentReward):
    """Ball speed towards the opponent's goal, as a fraction of the ball's top speed (negative if away)."""

    def reward(self, agent, state):
        blue = state.cars[agent].team_num == common_values.BLUE_TEAM
        target = np.asarray(common_values.ORANGE_GOAL_BACK if blue else common_values.BLUE_GOAL_BACK)
        to_goal = target - state.ball.position
        distance = np.linalg.norm(to_goal)
        if distance == 0:
            return 0.0
        return float(np.dot(state.ball.linear_velocity, to_goal / distance)) / common_values.BALL_MAX_SPEED


class InAirReward(_PerAgentReward):
    """A tiny bonus for leaving the ground, so the bot discovers jumping."""

    def reward(self, agent, state):
        return 0.0 if state.cars[agent].on_ground else 1.0
