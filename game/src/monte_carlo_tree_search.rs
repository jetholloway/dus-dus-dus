use crate::default_map::DefaultMap;
use crate::game::Game;
use crate::neural_network::NeuralNetwork;
use crate::real::Real;
use std::collections::HashSet;

pub struct MonteCarloTreeSearch<'a, G: Game, N: NeuralNetwork<G>> {
    game: &'a G,
    net: &'a N,
    exploration_bias: Real,
    visited: HashSet<&'a G::State>,
    state_action_probability: DefaultMap<&'a G::State, DefaultMap<&'a G::Action, Real>>,
    state_action_frequency: DefaultMap<&'a G::State, DefaultMap<&'a G::Action, Real>>,
    state_action_reward: DefaultMap<&'a G::State, DefaultMap<&'a G::Action, Real>>,
}

impl<'a, G: Game, N: NeuralNetwork<G>> MonteCarloTreeSearch<'a, G, N> {
    pub fn search(&mut self, state: &'a G::State) {
        let _ = self.search_reward(state);
    }

    fn search_reward(&mut self, state: &'a G::State) -> Real {
        let game = self.game;
        let net = self.net;

        if game.is_terminal(state) {
            return game.reward(state);
        }

        if !self.visited.contains(state) {
            let _ = self.visited.insert(state);
            let (action_probability, reward) = net.action_probability_and_reward(state);
            *self.state_action_probability.at(state) = action_probability;
            return reward;
        }

        let action = game
            .valid_actions(state)
            .max_by_key(|action| -> Real {
                let action = *action;

                let total_frequency = self
                    .state_action_frequency
                    .at(state)
                    .values()
                    .cloned()
                    .sum::<Real>();
                let frequency = *self.state_action_frequency.at(state).at(action);
                let probability = *self.state_action_probability.at(state).at(action);
                let reward = *self.state_action_reward.at(state).at(action);

                reward
                    + self.exploration_bias * probability * total_frequency.sqrt()
                        / (Real::from(1.0) + frequency)
            })
            .expect("non-terminal state should have valid actions");

        let next_state = game
            .next_state(state, action)
            .expect("valid action should produce next state");
        let reward_factor = if game.acting_player(state) != game.acting_player(next_state) {
            Real::from(-1.0)
        } else {
            Real::from(1.0)
        };
        let reward = reward_factor * self.search_reward(next_state);

        let frequency = *self.state_action_frequency.at(state).at(action);
        let mean_reward = *self.state_action_reward.at(state).at(action);
        *self.state_action_reward.at(state).at(action) =
            (frequency * mean_reward + reward) / (frequency + Real::from(1.0));
        *self.state_action_frequency.at(state).at(action) = frequency + Real::from(1.0);
        reward
    }
}
