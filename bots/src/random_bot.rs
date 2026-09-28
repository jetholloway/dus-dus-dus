use engine::{Action, GameState};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::Bot;

/// Plays a uniformly random legal action. The baseline every other bot
/// should beat.
pub struct RandomBot {
    rng: StdRng,
}

impl RandomBot {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl Bot for RandomBot {
    fn name(&self) -> &str {
        "random"
    }

    fn choose(&mut self, state: &GameState) -> Option<Action> {
        state.valid_actions().choose(&mut self.rng).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::ActionResult;

    fn play_out(seed: u64, max_actions: usize) -> Vec<Action> {
        let mut bot = RandomBot::new(seed);
        let mut state = GameState::new();
        let mut actions = Vec::new();

        for _ in 0..max_actions {
            let action = bot.choose(&state).expect("a legal action");
            match action.try_apply(&state) {
                ActionResult::Invalid(error) => panic!("RandomBot chose {action}: {error}"),
                ActionResult::Valid { state: next_state } => state = next_state,
                ActionResult::Terminal { .. } => break,
            }
            actions.push(action);
        }

        actions
    }

    #[test]
    fn chooses_only_legal_actions() {
        play_out(1, 300);
    }

    #[test]
    fn same_seed_plays_the_same_game() {
        assert_eq!(play_out(7, 100), play_out(7, 100));
    }

    #[test]
    fn no_action_once_the_game_is_over() {
        let mut bot = RandomBot::new(0);
        let mut state = GameState::new();
        loop {
            let action = bot.choose(&state).expect("a legal action");
            match action.try_apply(&state) {
                ActionResult::Invalid(error) => panic!("{error}"),
                ActionResult::Valid { state: next_state } => state = next_state,
                ActionResult::Terminal {
                    state: next_state, ..
                } => {
                    state = next_state;
                    break;
                }
            }
        }
        assert_eq!(bot.choose(&state), None);
    }
}
