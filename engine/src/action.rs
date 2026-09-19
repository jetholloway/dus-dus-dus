use super::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Action {
    Move(MoveAction),
    Tackle(TackleAction),
    Pass(PassAction),
}

#[derive(Debug)]
pub enum ActionResult {
    Invalid(&'static str),
    Valid { state: GameState },
    Terminal { state: GameState, winner: Player },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    Tackle,
    Pass,
}

impl Action {
    pub fn new(action_type: ActionType, src: Position, dst: Position) -> Self {
        match action_type {
            ActionType::Move => Self::new_move(src, dst),
            ActionType::Tackle => Self::new_tackle(src, dst),
            ActionType::Pass => Self::new_pass(src, dst),
        }
    }

    pub fn new_move(src: Position, dst: Position) -> Self {
        Self::Move(MoveAction::new(src, dst))
    }

    pub fn new_tackle(src: Position, dst: Position) -> Self {
        Self::Tackle(TackleAction::new(src, dst))
    }

    pub fn new_pass(src: Position, dst: Position) -> Self {
        Self::Pass(PassAction::new(src, dst))
    }

    pub fn action_type(&self) -> ActionType {
        match self {
            Action::Move(_) => ActionType::Move,
            Action::Tackle(_) => ActionType::Tackle,
            Action::Pass(_) => ActionType::Pass,
        }
    }

    pub fn src(&self) -> Position {
        match self {
            Action::Move(action) => action.src(),
            Action::Tackle(action) => action.src(),
            Action::Pass(action) => action.src(),
        }
    }

    pub fn dst(&self) -> Position {
        match self {
            Action::Move(action) => action.dst(),
            Action::Tackle(action) => action.dst(),
            Action::Pass(action) => action.dst(),
        }
    }

    pub fn try_apply(&self, state: &GameState) -> ActionResult {
        if state.is_terminal() {
            return ActionResult::Invalid("Game is over");
        }

        match self {
            Action::Move(action) => action.try_apply(state),
            Action::Tackle(action) => action.try_apply(state),
            Action::Pass(action) => action.try_apply(state),
        }
    }
}

impl From<Action> for String {
    fn from(action: Action) -> Self {
        action.to_string()
    }
}

impl TryFrom<String> for Action {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Move(move_action) => write!(f, "{}", move_action),
            Action::Tackle(tackle_action) => write!(f, "{}", tackle_action),
            Action::Pass(pass_action) => write!(f, "{}", pass_action),
        }
    }
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_iterator = s.split_whitespace();

        let action_type = split_iterator
            .next()
            .ok_or("Action missing type".to_string())?;
        let action_type = ActionType::from_str(action_type)?;

        let src = split_iterator
            .next()
            .ok_or("Action missing source position".to_string())?;
        let src = Position::from_str(src)
            .map_err(|error| format!("Invalid action source position: {error}"))?;

        let dst = split_iterator
            .next()
            .ok_or("Action missing destination position")?;
        let dst = Position::from_str(dst)
            .map_err(|error| format!("Invalid action destination position: {error}"))?;

        if let Some(remainder) = split_iterator
            .map(|s| s.to_string())
            .reduce(|rem, s| format!("{rem} {s}"))
        {
            return Err(format!("Trailing action input: {remainder}"));
        }

        Ok(Action::new(action_type, src, dst))
    }
}

impl ActionType {
    const NAMES: [&'static str; 3] = ["MOVE", "TACKLE", "PASS"];
}

impl FromStr for ActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let candidates: Vec<&str> = Self::NAMES
            .iter()
            .filter(|name| name.starts_with(s.to_ascii_uppercase().as_str()))
            .map(|name| *name)
            .collect();

        if candidates.len() != 1 {
            return Err(format!(
                "Invalid action type: {s} not like MOVE, TACKLE, PASS"
            ));
        }

        match candidates[0] {
            "MOVE" => Ok(ActionType::Move),
            "TACKLE" => Ok(ActionType::Tackle),
            "PASS" => Ok(ActionType::Pass),
            _ => Err(format!(
                "Invalid action type: {s} not like MOVE, TACKLE, PASS"
            )),
        }
    }
}
