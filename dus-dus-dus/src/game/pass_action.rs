use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone)]
pub struct PassAction {
    from: Position,
    to: Position,
}

impl PassAction {
    pub(super) fn new(from: Position, to: Position) -> Self {
        Self { from, to }
    }

    pub(super) fn try_apply(&self, state: &GameState) -> ActionResult {
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
            board.set_ball(self.to);
        })
    }

    fn not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.from) != Space::Piece(state.current_player())
    }

    fn acting_piece_has_no_ball(&self, state: &GameState) -> bool {
        self.from != state.ball()
    }

    fn target_space_not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.to) != Space::Piece(state.current_player())
    }

    fn try_get_straight_path(&self) -> Option<Path> {
        self.from
            .try_get_orthogonal_path(self.to)
            .or_else(|| self.from.try_get_diagonal_path(self.to))
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
        write!(f, "PASS {} {}", self.from, self.to)
    }
}
