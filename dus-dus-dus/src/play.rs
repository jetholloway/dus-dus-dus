use bots::Bot;
use engine::*;
use std::time::{Duration, Instant};

pub struct GameOutcome {
    /// None for a draw: the action cap was reached or a player had no legal
    /// action.
    pub winner: Option<Player>,
    pub actions: usize,
    /// Time spent choosing, and the number of choices, for First then Second.
    pub think: [(Duration, usize); 2],
}

/// Plays one game to the end, or to `max_actions`. Panics if a bot chooses an
/// illegal action, since that is a bug in the bot.
pub fn play<'a>(
    first: &mut (dyn Bot + 'a),
    second: &mut (dyn Bot + 'a),
    max_actions: usize,
    print: bool,
) -> GameOutcome {
    let mut state = GameState::new();
    let mut think = [(Duration::ZERO, 0); 2];
    if print {
        state.print();
    }

    for actions in 0..max_actions {
        let player = state.current_player();
        let (bot, side) = match player {
            Player::First => (&mut *first, 0),
            Player::Second => (&mut *second, 1),
        };

        let start = Instant::now();
        let choice = bot.choose(&state);
        think[side].0 += start.elapsed();
        think[side].1 += 1;

        let Some(action) = choice else {
            if print {
                println!("PLAYER {player} HAS NO LEGAL ACTION: DRAW");
            }
            return GameOutcome {
                winner: None,
                actions,
                think,
            };
        };

        match action.try_apply(&state) {
            ActionResult::Invalid(error) => {
                panic!("bot {} chose {action}: {error}\n{state:?}", bot.name())
            }
            ActionResult::Valid { state: next_state } => {
                if print {
                    println!("PLAYER {player} {action}");
                    next_state.print();
                }
                state = next_state;
            }
            ActionResult::Terminal {
                state: next_state,
                winner,
            } => {
                if print {
                    println!("PLAYER {player} {action}");
                    next_state.print();
                    println!("PLAYER {winner} WINS TURN {}", state.turn_count());
                }
                return GameOutcome {
                    winner: Some(winner),
                    actions: actions + 1,
                    think,
                };
            }
        }
    }

    if print {
        println!("NO WINNER AFTER {max_actions} ACTIONS: DRAW");
    }
    GameOutcome {
        winner: None,
        actions: max_actions,
        think,
    }
}
