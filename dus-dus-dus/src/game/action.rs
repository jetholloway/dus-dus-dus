use super::*;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Action {
    Move(MoveAction),
    Tackle(TackleAction),
    Pass(PassAction),
}

#[derive(Debug)]
pub enum ActionResult {
    Invalid(&'static str),
    Valid { state: GameState },
    Terminal { winner: Player, board: Board },
}

#[derive(Debug, Copy, Clone)]
pub enum ActionType {
    Move,
    Tackle,
    Pass,
}

impl Action {
    pub fn new(action_type: ActionType, from: Position, to: Position) -> Self {
        match action_type {
            ActionType::Move => Self::new_move(from, to),
            ActionType::Tackle => Self::new_tackle(from, to),
            ActionType::Pass => Self::new_pass(from, to),
        }
    }

    pub fn new_move(from: Position, to: Position) -> Self {
        Self::Move(MoveAction::new(from, to))
    }

    pub fn new_tackle(from: Position, to: Position) -> Self {
        Self::Tackle(TackleAction::new(from, to))
    }

    pub fn new_pass(from: Position, to: Position) -> Self {
        Self::Pass(PassAction::new(from, to))
    }

    pub(crate) fn try_apply(&self, state: &GameState) -> ActionResult {
        match self {
            Action::Move(action) => action.try_apply(state),
            Action::Tackle(action) => action.try_apply(state),
            Action::Pass(action) => action.try_apply(state),
        }
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

        let from = split_iterator
            .next()
            .ok_or("Action missing from position".to_string())?;
        let from = Position::from_str(from)
            .map_err(|error| format!("Invalid action from position: {error}"))?;

        let to = split_iterator.next().ok_or("Action missing to position")?;
        let to = Position::from_str(to)
            .map_err(|error| format!("Invalid action from position: {error}"))?;

        if let Some(remainder) = split_iterator
            .map(|s| s.to_string())
            .reduce(|rem, s| format!("{rem} {s}"))
        {
            return Err(format!("Trailing action input: {remainder}"));
        }

        Ok(Action::new(action_type, from, to))
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
