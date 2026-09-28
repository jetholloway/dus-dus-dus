//! Computer players. The engine knows the rules; this crate decides what to
//! play.

mod random_bot;

pub use random_bot::*;

use engine::{Action, GameState};

pub trait Bot {
    fn name(&self) -> &str;

    /// Picks one legal action for the current player, or `None` if there is
    /// none. Called once per action, so three times in a normal turn.
    fn choose(&mut self, state: &GameState) -> Option<Action>;
}

/// The names `make_bot` accepts.
pub const BOT_NAMES: [&str; 1] = ["random"];

/// Makes a bot by name. The seed makes any randomness in it repeatable.
pub fn make_bot(name: &str, seed: u64) -> Result<Box<dyn Bot>, String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "random" => Ok(Box::new(RandomBot::new(seed))),
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
}
