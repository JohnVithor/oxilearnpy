use tch::Tensor;

use crate::{cart_pole::CartPole, dqn::DoubleDeepAgent, OxiLearnErr};

pub type TrainResults = (Vec<f32>, Vec<u32>, Vec<f32>, Vec<f32>, Vec<f32>);

pub struct Trainer {
    env: CartPole,
    eval_env: CartPole,
    pub early_stop: Option<Box<dyn Fn(f32) -> bool>>,
}

impl Trainer {
    pub fn new(env: CartPole, eval_env: CartPole) -> Self {
        Self {
            env,
            eval_env,
            early_stop: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn train_by_steps(
        &mut self,
        agent: &mut DoubleDeepAgent,
        n_steps: u32,
        gradient_steps: u32,
        train_freq: u32,
        batch_size: usize,
        update_freq: u32,
        eval_freq: u32,
        eval_for: u32,
        verbose: usize,
    ) -> Result<TrainResults, OxiLearnErr> {
        let mut curr_obs: Tensor = Tensor::try_from(self.env.reset(None)).unwrap();
        let mut training_reward: Vec<f32> = vec![];
        let mut training_length: Vec<u32> = vec![];
        let mut training_error: Vec<f32> = vec![];
        let mut evaluation_reward: Vec<f32> = vec![];
        let mut evaluation_length: Vec<f32> = vec![];

        let mut n_episodes = 1;
        let mut action_counter: u32 = 0;
        let mut epi_reward: f32 = 0.0;
        agent.reset();

        for step in 1..=n_steps {
            action_counter += 1;
            let curr_action = agent.get_action(&curr_obs);
            // println!("{curr_action}");
            let (next_obs, reward, done, truncated) = self.env.step(curr_action).unwrap();
            let next_obs: Tensor = Tensor::try_from(next_obs).unwrap();

            epi_reward += reward;
            agent.add_transition(&curr_obs, curr_action, reward, done, &next_obs);

            curr_obs = next_obs;

            if step % train_freq == 0 {
                if let Some(td) = agent.update(gradient_steps, batch_size) {
                    training_error.push(td)
                }
            }

            if done || truncated {
                training_reward.push(epi_reward);
                training_length.push(action_counter);
                if n_episodes % update_freq == 0 && agent.update_networks().is_err() {
                    println!("copy error")
                }
                curr_obs = Tensor::try_from(self.env.reset(None)).unwrap();

                agent.action_selection_update(step as f32 / n_steps as f32, epi_reward);
                n_episodes += 1;
                epi_reward = 0.0;
                action_counter = 0;
            }

            if step % eval_freq == 0 {
                let (rewards, eval_lengths) = self.evaluate(agent, eval_for)?;
                let reward_avg = (rewards.iter().sum::<f32>()) / (rewards.len() as f32);
                let eval_lengths_avg = (eval_lengths.iter().map(|x| *x as f32).sum::<f32>())
                    / (eval_lengths.len() as f32);
                if verbose > 0 {
                    println!(
                        "steps number: {step} - eval reward: {reward_avg:.1} - epsilon: {:.1}",
                        agent.get_epsilon()
                    );
                }
                evaluation_reward.push(reward_avg);
                evaluation_length.push(eval_lengths_avg);
                if let Some(s) = &self.early_stop {
                    if (s)(reward_avg) {
                        training_reward.push(epi_reward);
                        training_length.push(action_counter);
                        break;
                    };
                }
            }
        }
        Ok((
            training_reward,
            training_length,
            training_error,
            evaluation_reward,
            evaluation_length,
        ))
    }

    pub fn evaluate(
        &mut self,
        agent: &mut DoubleDeepAgent,
        n_episodes: u32,
    ) -> Result<(Vec<f32>, Vec<u32>), OxiLearnErr> {
        let mut reward_history: Vec<f32> = vec![];
        let mut episode_length: Vec<u32> = vec![];
        for _episode in 0..n_episodes {
            let mut epi_reward: f32 = 0.0;
            let obs_repr = self.eval_env.reset(None);
            let obs_repr = Tensor::try_from(obs_repr).unwrap();
            let mut curr_action = agent.get_best_action(&obs_repr);
            let mut action_counter: u32 = 0;
            loop {
                let (obs, reward, done, truncated) = self.eval_env.step(curr_action).unwrap();
                let next_obs_repr = Tensor::try_from(obs).unwrap();
                let next_action_repr: usize = agent.get_best_action(&next_obs_repr);
                let next_action = next_action_repr;
                curr_action = next_action;
                epi_reward += reward;
                if done || truncated {
                    reward_history.push(epi_reward);
                    episode_length.push(action_counter);
                    break;
                }
                action_counter += 1;
            }
        }
        Ok((reward_history, episode_length))
    }
}

pub fn print_python_like(tensor: &Tensor) {
    let shape = tensor.size();
    let mut s = String::from("tensor([");
    for i in 0..shape[0] - 1 {
        let v = tensor.double_value(&[i as i64]);
        if v > 0.0 {
            s.push_str(&format!(" {:.4}, ", v));
        } else {
            s.push_str(&format!("{:.4}, ", v));
        }
    }
    let v = tensor.double_value(&[shape[0] - 1 as i64]);
    if v > 0.0 {
        s.push_str(&format!(" {:.4}", v));
    } else {
        s.push_str(&format!("{:.4}", v));
    }
    s.push_str("])");
    println!("{}", s);
}
