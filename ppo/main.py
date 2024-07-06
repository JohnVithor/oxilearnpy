import gymnasium as gym
import torch
import torch.optim as optim
import numpy as np
import random

from model import Policy
from ppo import PPO


if __name__ == "__main__":
    env_id = "CartPole-v1"
    num_envs = 1

    seed = 42
    # TRY NOT TO MODIFY: seeding
    random.seed(seed)
    np.random.seed(seed)
    torch.manual_seed(seed)
    torch.backends.cudnn.deterministic = True

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

    envs = gym.vector.SyncVectorEnv(
        [
            lambda: gym.wrappers.RecordEpisodeStatistics(gym.make(env_id))
            for _ in range(num_envs)
        ],
    )
    assert isinstance(
        envs.single_action_space, gym.spaces.Discrete
    ), "only discrete action space is supported"
    policy = Policy(
        np.array(envs.single_observation_space.shape).prod(), envs.single_action_space.n
    ).to(device)
    optimizer = optim.Adam(policy.parameters(), lr=2.5e-4, eps=1e-5)

    trainer = PPO(
        policy,
        optimizer,
        envs,
        device=device,
    )

    trainer.learn(100_000, seed)

    envs.close()