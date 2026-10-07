use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    turn: Turn,
    board: Board,
    // States saved before rule variants existed were played by the standard
    // rules, which is what a missing field gives.
    #[serde(default)]
    rules: Rules,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Turn {
    turn_count: u32,
    player: Player,
    action_count: ActionCount,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Player {
    First,
    Second,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ActionCount {
    First,
    Second,
    Third,
}

impl GameState {
    pub fn new() -> Self {
        Self::with_rules(Rules::STANDARD)
    }

    pub fn with_rules(rules: Rules) -> Self {
        Self {
            turn: Turn::new(),
            board: Board::new(),
            rules,
        }
    }

    pub fn rules(&self) -> Rules {
        self.rules
    }

    pub fn print(&self) {
        println!("{}", self.turn);
        self.board.print();
    }

    pub(crate) fn try_apply<F>(&self, update_board: F) -> ActionResult
    where
        F: FnOnce(&mut Board),
    {
        let mut board = self.board.clone();
        update_board(&mut board);

        if !self.setup() {
            if let Some(stall) = board.stall() {
                return ActionResult::Invalid(stall);
            }
        }

        let state = Self {
            turn: self.turn.next(&self.rules),
            board,
            rules: self.rules,
        };

        if state.board.has_winner(self.turn.player) {
            return ActionResult::Terminal {
                state,
                winner: self.turn.player,
            };
        }

        ActionResult::Valid { state }
    }

    pub fn setup(&self) -> bool {
        self.turn.turn_count == 0
    }

    pub fn action_count(&self) -> ActionCount {
        self.turn.action_count
    }

    pub fn turn_count(&self) -> u32 {
        self.turn.turn_count
    }

    pub fn current_player(&self) -> Player {
        self.turn.player
    }

    pub fn winner(&self) -> Option<Player> {
        if self.board.has_winner(Player::First) {
            Some(Player::First)
        } else if self.board.has_winner(Player::Second) {
            Some(Player::Second)
        } else {
            None
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.winner().is_some()
    }

    pub fn other_player(&self) -> Player {
        self.turn.player.other()
    }

    pub fn space(&self, position: Position) -> Space {
        self.board.space(position)
    }

    pub fn ball(&self) -> Position {
        self.board.ball()
    }
}

impl Turn {
    fn new() -> Self {
        Self {
            turn_count: 0,
            player: Player::First,
            action_count: ActionCount::First,
        }
    }

    fn next(&self, rules: &Rules) -> Self {
        if self.turn_count == 0 {
            // Each side makes as many setup moves as the rules give it.
            let next_action = match self.action_count {
                ActionCount::First => Some(ActionCount::Second),
                ActionCount::Second => Some(ActionCount::Third),
                ActionCount::Third => None,
            }
            .filter(|_| self.action_count.number() < rules.setup_moves(self.player));

            return match (next_action, self.player) {
                (Some(action_count), player) => Self {
                    turn_count: 0,
                    player,
                    action_count,
                },
                (None, Player::First) => Self {
                    turn_count: 0,
                    player: Player::Second,
                    action_count: ActionCount::First,
                },
                (None, Player::Second) => Self {
                    turn_count: 1,
                    player: Player::First,
                    action_count: ActionCount::First,
                },
            };
        }

        match self.action_count {
            ActionCount::First => Self {
                turn_count: self.turn_count,
                player: self.player,
                action_count: ActionCount::Second,
            },
            ActionCount::Second => Self {
                turn_count: self.turn_count,
                player: self.player,
                action_count: ActionCount::Third,
            },
            ActionCount::Third => Self {
                turn_count: self.turn_count + 1,
                player: self.player.other(),
                action_count: ActionCount::First,
            },
        }
    }
}

impl Display for Turn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.turn_count == 0 {
            write!(f, "PLAYER {} SETUP", self.player)
        } else {
            write!(
                f,
                "PLAYER {} ACTION {} / 3 TURN {}",
                self.player, self.action_count, self.turn_count
            )
        }
    }
}

impl ActionCount {
    /// 1, 2 or 3.
    pub fn number(self) -> u8 {
        match self {
            ActionCount::First => 1,
            ActionCount::Second => 2,
            ActionCount::Third => 3,
        }
    }
}

impl Player {
    pub(crate) fn other(&self) -> Self {
        match self {
            Player::First => Player::Second,
            Player::Second => Player::First,
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Player::First => 'O',
            Player::Second => 'X',
        };
        write!(f, "{}", char)
    }
}

impl Display for ActionCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            ActionCount::First => '1',
            ActionCount::Second => '2',
            ActionCount::Third => '3',
        };
        write!(f, "{}", char)
    }
}
