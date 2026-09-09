mod action;
mod board;
mod game_state;
mod move_action;
mod pass_action;
mod position;
mod tackle_action;

use crate::game_player::GamePlayer;

pub use action::*;
pub use board::*;
pub use game_state::*;
use move_action::*;
use pass_action::*;
pub use position::*;
use tackle_action::*;

pub fn play<'a>(
    first: &'a mut dyn GamePlayer,
    second: &'a mut dyn GamePlayer,
    print: bool,
) -> Player {
    let mut state = GameState::new();
    if print {
        state.print();
    }

    loop {
        let action = match state.current_player() {
            Player::First => first.next_action(&state),
            Player::Second => second.next_action(&state),
        };

        match action.try_apply(&state) {
            ActionResult::Invalid(error) => {
                match state.current_player() {
                    Player::First => first.invalid_action(error),
                    Player::Second => second.invalid_action(error),
                };
                continue;
            }
            ActionResult::Valid { state: next_state } => {
                if print {
                    println!("PLAYER {} {action}", state.current_player());
                    next_state.print();
                }
                state = next_state;
                continue;
            }
            ActionResult::Terminal { winner, board } => {
                if print {
                    println!("PLAYER {} {action}", state.current_player());
                    board.print();
                    println!("PLAYER {winner} WINS TURN {}", state.turn_count());
                }
                return winner;
            }
        }
    }
}
