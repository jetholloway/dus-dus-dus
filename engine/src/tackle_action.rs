use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TackleAction {
    src: Position,
    dst: Position,
}

impl TackleAction {
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
            board.set_space(self.src, Space::Empty);
            board.set_space(self.dst, Space::Piece(state.current_player()));
            board.set_ball(self.dst);
        })
    }

    fn not_current_players_piece(&self, state: &GameState) -> bool {
        state.space(self.src) != Space::Piece(state.current_player())
    }

    fn acting_piece_has_ball(&self, state: &GameState) -> bool {
        self.src == state.ball()
    }

    fn target_space_is_occupied(&self, state: &GameState) -> bool {
        state.space(self.dst) != Space::Empty
    }

    fn target_space_too_far(&self) -> bool {
        self.src.chebyshev_distance(self.dst) > 1
    }

    fn target_space_winning_rank(&self, state: &GameState) -> bool {
        state.current_player() == Player::First && self.dst.y == 6
            || state.current_player() == Player::Second && self.dst.y == 0
    }

    fn ball_not_adjacent(&self, state: &GameState) -> bool {
        self.src.taxicab_distance(state.ball()) != 1
    }

    fn current_player_has_ball(&self, state: &GameState) -> bool {
        state.space(state.ball()) == Space::Piece(state.current_player())
    }
}

impl Display for TackleAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TACKLE {} {}", self.src, self.dst)
    }
}
