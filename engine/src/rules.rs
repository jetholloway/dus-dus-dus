use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::*;

/// Variations on the setup, for trying to even out Orange's advantage from
/// moving first. The default is the rulebook's.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rules {
    /// Whose first setup move places the ball on the piece it moved.
    pub setup_ball: Player,
    /// How many setup moves Teal (Second) makes, each with a different
    /// piece: 1 in the rulebook, or 2.
    pub second_setup_moves: u8,
}

impl Default for Rules {
    fn default() -> Self {
        Self::STANDARD
    }
}

impl Rules {
    pub const STANDARD: Self = Self {
        setup_ball: Player::First,
        second_setup_moves: 1,
    };

    /// The names `parse` accepts, each changing one thing from the rulebook.
    pub const VARIANTS: [&'static str; 2] = ["teal-ball", "teal-double-setup"];

    /// `standard`, or variants joined with `+`, e.g.
    /// `teal-ball+teal-double-setup`.
    pub fn parse(spec: &str) -> Result<Self, String> {
        let mut rules = Self::STANDARD;
        if spec == "standard" {
            return Ok(rules);
        }

        for variant in spec.split('+') {
            match variant {
                "teal-ball" => rules.setup_ball = Player::Second,
                "teal-double-setup" => rules.second_setup_moves = 2,
                _ => {
                    return Err(format!(
                        "unknown rules {variant:?}; expected standard or any of {} joined with +",
                        Self::VARIANTS.join(", ")
                    ))
                }
            }
        }

        Ok(rules)
    }
}

impl Display for Rules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut variants = Vec::new();
        if self.setup_ball == Player::Second {
            variants.push("teal-ball");
        }
        if self.second_setup_moves == 2 {
            variants.push("teal-double-setup");
        }

        if variants.is_empty() {
            write!(f, "standard")
        } else {
            write!(f, "{}", variants.join("+"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(state: &GameState, notation: &str) -> GameState {
        let action: Action = notation.parse().unwrap();
        match action.try_apply(state) {
            ActionResult::Valid { state } => state,
            other => panic!("{notation}: {other:?}"),
        }
    }

    fn refused(state: &GameState, notation: &str) -> bool {
        let action: Action = notation.parse().unwrap();
        matches!(action.try_apply(state), ActionResult::Invalid(_))
    }

    fn position(notation: &str) -> Position {
        notation.parse().unwrap()
    }

    #[test]
    fn names_round_trip() {
        for spec in [
            "standard",
            "teal-ball",
            "teal-double-setup",
            "teal-ball+teal-double-setup",
        ] {
            assert_eq!(Rules::parse(spec).unwrap().to_string(), spec);
        }
        assert!(Rules::parse("turbo").is_err());
        assert!(Rules::parse("standard+teal-ball").is_err());
    }

    #[test]
    fn standard_rules_give_orange_the_ball_and_teal_one_setup_move() {
        let state = GameState::new();
        let state = play(&state, "MOVE D1 D3");
        assert_eq!(state.ball(), position("D3"));
        let state = play(&state, "MOVE D7 D5");
        assert_eq!(
            (state.current_player(), state.setup()),
            (Player::First, false)
        );
    }

    #[test]
    fn teal_ball_puts_the_ball_on_teals_setup_piece() {
        let state = GameState::with_rules(Rules::parse("teal-ball").unwrap());
        let state = play(&state, "MOVE D1 D3");
        assert_eq!(state.space(state.ball()), Space::Invalid, "no ball yet");
        let state = play(&state, "MOVE B7 B5");
        assert_eq!(state.ball(), position("B5"));
        assert_eq!(state.current_player(), Player::First);
    }

    #[test]
    fn teal_double_setup_moves_two_different_pieces_forward() {
        let state = GameState::with_rules(Rules::parse("teal-double-setup").unwrap());
        let state = play(&state, "MOVE D1 D3");
        assert_eq!(state.ball(), position("D3"));
        let state = play(&state, "MOVE C7 C5");
        assert_eq!(
            (state.current_player(), state.setup()),
            (Player::Second, true)
        );

        // Not the same piece again, forward or sideways.
        assert!(refused(&state, "MOVE C5 C3"));
        assert!(refused(&state, "MOVE C5 A5"));
        let state = play(&state, "MOVE F7 F5");
        assert_eq!(
            (state.current_player(), state.setup()),
            (Player::First, false)
        );
    }

    #[test]
    fn with_both_the_ball_goes_on_teals_first_setup_piece() {
        let state = GameState::with_rules(Rules::parse("teal-ball+teal-double-setup").unwrap());
        let state = play(&state, "MOVE D1 D3");
        let state = play(&state, "MOVE C7 C5");
        assert_eq!(state.ball(), position("C5"));
        // Setup is moves only: Teal can't pass its new ball.
        assert!(refused(&state, "PASS C5 C7"));
        let state = play(&state, "MOVE F7 F5");
        assert_eq!(state.ball(), position("C5"));
    }
}
