use crate::game_player::GamePlayer;
use engine::*;

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
