use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PassAction {
    src: Position,
    dst: Position,
}

impl PassAction {
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

        if self.acting_piece_has_no_ball(state) {
            return ActionResult::Invalid("Acting piece has no ball");
        }

        if self.target_space_not_current_players_piece(state) {
            return ActionResult::Invalid("Target space not current player's piece");
        }

        let Some(path) = self.try_get_straight_path() else {
            return ActionResult::Invalid("Path not straight");
        };

        if self.path_blocked(state, &path) {
            return ActionResult::Invalid("Path blocked");
        }

        state.try_apply(|board| {
            board.set_ball(self.dst);
        })
    }

    fn not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.src) != Space::Piece(state.current_player())
    }

    fn acting_piece_has_no_ball(&self, state: &GameState) -> bool {
        self.src != state.ball()
    }

    fn target_space_not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.dst) != Space::Piece(state.current_player())
    }

    fn try_get_straight_path(&self) -> Option<Path> {
        self.src
            .try_get_orthogonal_path(self.dst)
            .or_else(|| self.src.try_get_diagonal_path(self.dst))
    }

    fn path_blocked(&self, state: &GameState, path: &Vec<Position>) -> bool {
        for position in path {
            if state.space(*position) == Space::Invalid
                || state.space(*position) == Space::Piece(state.other_player())
            {
                return true;
            }
        }

        false
    }
}

impl Display for PassAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PASS {} {}", self.src, self.dst)
    }
}
