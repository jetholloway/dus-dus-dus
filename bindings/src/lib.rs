use engine::ActionResult;
use pyo3::create_exception;
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use std::str::FromStr;

create_exception!(dus_engine, InvalidAction, PyValueError);

#[pyclass(name = "Player", eq, frozen, hash, from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PyPlayer {
    First,
    Second,
}

impl From<engine::Player> for PyPlayer {
    fn from(player: engine::Player) -> Self {
        match player {
            engine::Player::First => PyPlayer::First,
            engine::Player::Second => PyPlayer::Second,
        }
    }
}

#[pyclass(name = "Position", eq, frozen, hash, from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PyPosition(engine::Position);

impl PyPosition {
    fn on_board(position: engine::Position) -> PyResult<Self> {
        if (0..7).contains(&position.x) && (0..7).contains(&position.y) {
            Ok(Self(position))
        } else {
            Err(PyIndexError::new_err(format!(
                "Position ({}, {}) is off the board",
                position.x, position.y
            )))
        }
    }
}

#[pymethods]
impl PyPosition {
    #[new]
    fn new(x: i8, y: i8) -> PyResult<Self> {
        Self::on_board(engine::Position { x, y })
    }

    #[staticmethod]
    fn parse(notation: &str) -> PyResult<Self> {
        let position = engine::Position::from_str(notation).map_err(PyValueError::new_err)?;
        Self::on_board(position)
    }

    #[getter]
    fn x(&self) -> i8 {
        self.0.x
    }

    #[getter]
    fn y(&self) -> i8 {
        self.0.y
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Position.parse('{}')", self.0)
    }
}

#[pyclass(name = "Action", eq, frozen, hash, from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PyAction(engine::Action);

#[pymethods]
impl PyAction {
    #[new]
    fn new(notation: &str) -> PyResult<Self> {
        let action = engine::Action::from_str(notation).map_err(PyValueError::new_err)?;
        PyPosition::on_board(action.src())?;
        PyPosition::on_board(action.dst())?;
        Ok(Self(action))
    }

    #[getter]
    fn action_type(&self) -> &'static str {
        match self.0.action_type() {
            engine::ActionType::Move => "MOVE",
            engine::ActionType::Tackle => "TACKLE",
            engine::ActionType::Pass => "PASS",
        }
    }

    #[getter]
    fn src(&self) -> PyPosition {
        PyPosition(self.0.src())
    }

    #[getter]
    fn dst(&self) -> PyPosition {
        PyPosition(self.0.dst())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Action('{}')", self.0)
    }
}

#[pyclass(name = "GameState", eq, frozen, from_py_object)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct PyGameState(engine::GameState);

#[pymethods]
impl PyGameState {
    #[new]
    fn new() -> Self {
        Self(engine::GameState::new())
    }

    #[getter]
    fn current_player(&self) -> PyPlayer {
        self.0.current_player().into()
    }

    #[getter]
    fn setup(&self) -> bool {
        self.0.setup()
    }

    #[getter]
    fn turn_count(&self) -> u32 {
        self.0.turn_count()
    }

    #[getter]
    fn action_count(&self) -> u8 {
        match self.0.action_count() {
            engine::ActionCount::First => 1,
            engine::ActionCount::Second => 2,
            engine::ActionCount::Third => 3,
        }
    }

    #[getter]
    fn winner(&self) -> Option<PyPlayer> {
        self.0.winner().map(PyPlayer::from)
    }

    #[getter]
    fn is_terminal(&self) -> bool {
        self.0.is_terminal()
    }

    #[getter]
    fn ball(&self) -> Option<PyPosition> {
        PyPosition::on_board(self.0.ball()).ok()
    }

    fn piece_at(&self, position: PyPosition) -> Option<PyPlayer> {
        match self.0.space(position.0) {
            engine::Space::Piece(player) => Some(player.into()),
            engine::Space::Empty | engine::Space::Invalid => None,
        }
    }

    fn valid_actions(&self) -> Vec<PyAction> {
        self.0.valid_actions().into_iter().map(PyAction).collect()
    }

    fn apply(&self, action: PyAction) -> PyResult<Self> {
        match action.0.try_apply(&self.0) {
            ActionResult::Invalid(error) => Err(InvalidAction::new_err(error)),
            ActionResult::Valid { state } | ActionResult::Terminal { state, .. } => Ok(Self(state)),
        }
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        serde_json::from_str(json)
            .map(Self)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "<GameState turn={} player={:?} action={} winner={:?}>",
            self.turn_count(),
            self.current_player(),
            self.action_count(),
            self.winner()
        )
    }
}

#[pyclass(name = "GameRecord", eq, skip_from_py_object)]
#[derive(Debug, Clone, PartialEq)]
struct PyGameRecord(engine::GameRecord);

#[pymethods]
impl PyGameRecord {
    #[new]
    #[pyo3(signature = (actions = None, winner = None))]
    fn new(actions: Option<Vec<PyAction>>, winner: Option<PyPlayer>) -> Self {
        Self(engine::GameRecord {
            actions: actions
                .unwrap_or_default()
                .into_iter()
                .map(|action| action.0)
                .collect(),
            winner: winner.map(player_to_engine),
        })
    }

    #[getter]
    fn actions(&self) -> Vec<PyAction> {
        self.0.actions.iter().cloned().map(PyAction).collect()
    }

    #[getter]
    fn winner(&self) -> Option<PyPlayer> {
        self.0.winner.map(PyPlayer::from)
    }

    #[setter]
    fn set_winner(&mut self, winner: Option<PyPlayer>) {
        self.0.winner = winner.map(player_to_engine);
    }

    fn push(&mut self, action: PyAction) {
        self.0.actions.push(action.0);
    }

    fn replay(&self) -> PyResult<Vec<PyGameState>> {
        self.0
            .replay()
            .map(|states| states.into_iter().map(PyGameState).collect())
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        serde_json::from_str(json)
            .map(Self)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    fn __len__(&self) -> usize {
        self.0.actions.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "<GameRecord actions={} winner={:?}>",
            self.0.actions.len(),
            self.winner()
        )
    }
}

fn player_to_engine(player: PyPlayer) -> engine::Player {
    match player {
        PyPlayer::First => engine::Player::First,
        PyPlayer::Second => engine::Player::Second,
    }
}

#[pymodule]
fn dus_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPlayer>()?;
    m.add_class::<PyPosition>()?;
    m.add_class::<PyAction>()?;
    m.add_class::<PyGameState>()?;
    m.add_class::<PyGameRecord>()?;
    m.add("InvalidAction", m.py().get_type::<InvalidAction>())?;
    Ok(())
}
