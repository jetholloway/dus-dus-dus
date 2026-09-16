use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone)]
pub struct MoveAction {
    from: Position,
    to: Position,
}

impl MoveAction {
    pub(crate) fn new(from: Position, to: Position) -> Self {
        Self { from, to }
    }

    pub(crate) fn try_apply(&self, state: &GameState) -> ActionResult {
        if self.not_current_players_piece(state) {
            return ActionResult::Invalid("Not current player's piece");
        }

        if self.acting_piece_has_ball(state) {
            return ActionResult::Invalid("Acting piece has ball");
        }

        if self.target_space_occupied(state) {
            return ActionResult::Invalid("Target space occupied");
        }

        let Some(path) = self.try_get_orthogonal_path() else {
            return ActionResult::Invalid("Path not orthogonal");
        };

        if self.path_too_log(&path) {
            return ActionResult::Invalid("Path too long");
        }

        if self.path_too_short(state, &path) {
            return ActionResult::Invalid("Path too short");
        }

        if self.path_blocked(state, &path) {
            return ActionResult::Invalid("Path blocked");
        }

        state.try_apply(|board| {
            board.set_space(self.from, Space::Empty);
            board.set_space(self.to, Space::Piece(state.current_player()));

            if state.setup() && state.current_player() == Player::First {
                board.set_ball(self.to);
            }
        })
    }

    fn not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.from) != Space::Piece(state.current_player())
    }

    fn acting_piece_has_ball(&self, state: &GameState) -> bool {
        self.from == state.ball()
    }

    fn target_space_occupied(&self, state: &GameState) -> bool {
        state.space(self.to) != Space::Empty
    }

    fn try_get_orthogonal_path(&self) -> Option<Path> {
        self.from.try_get_orthogonal_path(self.to)
    }

    fn path_too_log(&self, path: &Path) -> bool {
        path.len() > 1
    }

    fn path_too_short(&self, state: &GameState, path: &Path) -> bool {
        state.setup() && path.len() == 0
    }

    fn path_blocked(&self, state: &GameState, path: &Vec<Position>) -> bool {
        for position in path {
            if state.space(*position) != Space::Empty {
                return true;
            }
        }

        false
    }
}

impl Display for MoveAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MOVE {} {}", self.from, self.to)
    }
}
