use super::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRecord {
    pub actions: Vec<Action>,
    pub winner: Option<Player>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    InvalidAction { index: usize, error: &'static str },
    ActionAfterWin { index: usize },
    WinnerMismatch { recorded: Option<Player>, replayed: Option<Player> },
}

impl GameRecord {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            winner: None,
        }
    }

    pub fn replay(&self) -> Result<Vec<GameState>, ReplayError> {
        let mut state = GameState::new();
        let mut states = vec![state.clone()];
        let mut winner = None;

        for (index, action) in self.actions.iter().enumerate() {
            if winner.is_some() {
                return Err(ReplayError::ActionAfterWin { index });
            }

            match action.try_apply(&state) {
                ActionResult::Invalid(error) => {
                    return Err(ReplayError::InvalidAction { index, error })
                }
                ActionResult::Valid { state: next_state } => state = next_state,
                ActionResult::Terminal {
                    state: next_state,
                    winner: next_winner,
                } => {
                    state = next_state;
                    winner = Some(next_winner);
                }
            }

            states.push(state.clone());
        }

        if winner != self.winner {
            return Err(ReplayError::WinnerMismatch {
                recorded: self.winner,
                replayed: winner,
            });
        }

        Ok(states)
    }
}

impl Default for GameRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ReplayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::InvalidAction { index, error } => {
                write!(f, "Action {index} invalid: {error}")
            }
            ReplayError::ActionAfterWin { index } => {
                write!(f, "Action {index} follows a winning action")
            }
            ReplayError::WinnerMismatch { recorded, replayed } => {
                write!(f, "Recorded winner {recorded:?} but replayed {replayed:?}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    fn random_game(seed: u64) -> (GameRecord, GameState) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut record = GameRecord::new();
        let mut state = GameState::new();

        loop {
            let action = state
                .valid_actions()
                .choose(&mut rng)
                .expect("non-terminal state should have valid actions")
                .clone();

            match action.try_apply(&state) {
                ActionResult::Invalid(error) => panic!("valid action rejected: {error}"),
                ActionResult::Valid { state: next_state } => {
                    record.actions.push(action);
                    state = next_state;
                }
                ActionResult::Terminal {
                    state: next_state,
                    winner,
                } => {
                    record.actions.push(action);
                    record.winner = Some(winner);
                    return (record, next_state);
                }
            }
        }
    }

    #[test]
    fn replay_reproduces_the_recorded_game() {
        let (record, final_state) = random_game(20260918);

        let states = record.replay().expect("recorded game should replay");

        assert_eq!(states.len(), record.actions.len() + 1);
        assert_eq!(states.first(), Some(&GameState::new()));
        assert_eq!(states.last(), Some(&final_state));
        assert!(record.winner.is_some());
    }

    #[test]
    fn record_survives_a_json_round_trip() {
        let (record, _) = random_game(1234);

        let json = serde_json::to_string(&record).expect("record should serialize");
        let parsed: GameRecord = serde_json::from_str(&json).expect("record should deserialize");

        assert_eq!(parsed, record);

        let first_action = format!("\"{}\"", record.actions[0]);
        assert!(
            json.contains(first_action.as_str()),
            "expected {json} to contain the action notation {first_action}"
        );
    }

    #[test]
    fn replay_rejects_a_corrupted_record() {
        let (mut record, _) = random_game(99);
        record.actions[0] = Action::new_move(Position { x: 0, y: 0 }, Position { x: 6, y: 6 });

        assert!(
            matches!(
                record.replay(),
                Err(ReplayError::InvalidAction { index: 0, .. })
            ),
            "expected a corrupted record to be rejected, got {:?}",
            record.replay()
        );
    }

    #[test]
    fn replay_rejects_a_wrong_winner() {
        let (mut record, _) = random_game(7);
        let replayed = record.winner;
        record.winner = None;

        assert_eq!(
            record.replay(),
            Err(ReplayError::WinnerMismatch {
                recorded: None,
                replayed,
            })
        );
    }
}

