use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TackleAction {
    from: Position,
    to: Position,
}

impl TackleAction {
    pub(crate) fn new(from: Position, to: Position) -> Self {
        Self { from, to }
    }

    pub fn from(&self) -> Position {
        self.from
    }

    pub fn to(&self) -> Position {
        self.to
    }

    pub(crate) fn try_apply(&self, state: &GameState) -> ActionResult {
        if self.not_current_players_piece(state) {
            return ActionResult::Invalid("Not current player's piece");
        }

        if self.acting_piece_has_ball(state) {
            return ActionResult::Invalid("Acting piece has ball");
        }

        if self.target_space_is_occupied(state) {
            return ActionResult::Invalid("Target space is occupied");
        }

        if self.target_space_too_far() {
            return ActionResult::Invalid("Target space too far");
        }

        if self.target_space_winning_rank(state) {
            return ActionResult::Invalid("Target space winning rank");
        }

        if self.ball_not_adjacent(state) {
            return ActionResult::Invalid("Ball not adjacent");
        }

        if self.current_player_has_ball(state) {
            return ActionResult::Invalid("Current player has ball");
        }

        state.try_apply(|board| {
            board.set_space(self.from, Space::Empty);
            board.set_space(self.to, Space::Piece(state.current_player()));
            board.set_ball(self.to);
        })
    }

    fn not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.from) != Space::Piece(state.current_player())
    }

    fn acting_piece_has_ball(&self, state: &GameState) -> bool {
        self.from == state.ball()
    }

    fn target_space_is_occupied(&self, state: &GameState) -> bool {
        state.space(self.to) != Space::Empty
    }

    fn target_space_too_far(&self) -> bool {
        self.from.chebyshev_distance(self.to) > 1
    }

    fn target_space_winning_rank(&self, state: &GameState) -> bool {
        state.current_player() == Player::First && self.to.y == 6
            || state.current_player() == Player::Second && self.to.y == 0
    }

    fn ball_not_adjacent(&self, state: &GameState) -> bool {
        self.from.taxicab_distance(state.ball()) != 1
    }

    fn current_player_has_ball(&self, state: &GameState) -> bool {
        state.space(state.ball()) == Space::Piece(state.current_player())
    }
}

impl Display for TackleAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TACKLE {} {}", self.from, self.to)
    }
}
