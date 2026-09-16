// learning_player/ is an unfinished draft and is excluded from the build.

use engine::*;

pub trait GamePlayer {
    fn next_action(&mut self, state: &GameState) -> Action;

    fn invalid_action(&mut self, error: &str);
}
