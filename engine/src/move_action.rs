use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveAction {
    src: Position,
    dst: Position,
}

impl MoveAction {
    pub(crate) fn new(src: Position, dst: Position) -> Self {
        Self { src, dst }
    }

    pub fn src(&self) -> Position {
        self.src
    }

    pub fn dst(&self) -> Position {
        self.dst
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
            board.set_space(self.src, Space::Empty);
            board.set_space(self.dst, Space::Piece(state.current_player()));

            if state.setup() && state.current_player() == Player::First {
                board.set_ball(self.dst);
            }
        })
    }

    fn not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.src) != Space::Piece(state.current_player())
    }

    fn acting_piece_has_ball(&self, state: &GameState) -> bool {
        self.src == state.ball()
    }

    fn target_space_occupied(&self, state: &GameState) -> bool {
        state.space(self.dst) != Space::Empty
    }

    fn try_get_orthogonal_path(&self) -> Option<Path> {
        self.src.try_get_orthogonal_path(self.dst)
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
        write!(f, "MOVE {} {}", self.src, self.dst)
    }
}
