//! Computer players. The engine knows the rules; this crate decides what to
//! play.

mod heuristic_bot;
mod random_bot;

pub use heuristic_bot::*;
pub use random_bot::*;

use engine::{Action, GameState};

/// `Send` so a bot can be handed between threads, as the Python bindings
/// require.
pub trait Bot: Send {
    fn name(&self) -> &str;

    /// Picks one legal action for the current player, or `None` if there is
    /// none. Called once per action, so three times in a normal turn.
    fn choose(&mut self, state: &GameState) -> Option<Action>;
}

/// The names `make_bot` accepts.
pub const BOT_NAMES: [&str; 2] = ["random", "heuristic"];

/// Makes a bot by name. The seed makes any randomness in it repeatable.
///
/// `heuristic` takes options after a colon: `pieces=N` to notice only N of
/// its 7 pieces each turn, which weakens it, and weight overrides, e.g.
/// `heuristic:pieces=5,possession=12`; see [`HeuristicBot`] and [`Weights`].
pub fn make_bot(name: &str, seed: u64) -> Result<Box<dyn Bot>, String> {
    let (kind, options) = name.trim().split_once(':').unwrap_or((name.trim(), ""));
    match (kind.to_ascii_lowercase().as_str(), options) {
        ("random", "") => Ok(Box::new(RandomBot::new(seed))),
        ("heuristic", options) => Ok(Box::new(HeuristicBot::from_options(options, seed)?)),
        _ => Err(format!(
            "unknown bot {name:?}; expected one of: {}",
            BOT_NAMES.join(", ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_name_makes_a_bot_of_that_name() {
        for name in BOT_NAMES {
            assert_eq!(make_bot(name, 0).unwrap().name(), name);
        }
    }

    #[test]
    fn unknown_name_is_an_error() {
        assert!(make_bot("genius", 0).is_err());
    }

    #[test]
    fn heuristic_takes_weight_overrides() {
        assert!(make_bot("heuristic:possession=12,open_lane=30", 0).is_ok());
        assert!(make_bot("heuristic:luck=3", 0).is_err());
        assert!(make_bot("random:possession=12", 0).is_err());
    }
}
