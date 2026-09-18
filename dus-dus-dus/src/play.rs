use crate::game_player::GamePlayer;
use engine::*;

pub fn play<'a>(
    first: &'a mut dyn GamePlayer,
    second: &'a mut dyn GamePlayer,
    print: bool,
) -> GameRecord {
    let mut record = GameRecord::new();
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
                record.actions.push(action);
                state = next_state;
                continue;
            }
            ActionResult::Terminal {
                state: next_state,
                winner,
            } => {
                if print {
                    println!("PLAYER {} {action}", state.current_player());
                    next_state.print();
                    println!("PLAYER {winner} WINS TURN {}", state.turn_count());
                }
                record.actions.push(action);
                record.winner = Some(winner);
                return record;
            }
        }
    }
}
