mod learning_player;

use crate::*;

pub trait GamePlayer {
    fn next_action(&mut self, state: &GameState) -> Action;

    fn invalid_action(&mut self, error: &str);
}
