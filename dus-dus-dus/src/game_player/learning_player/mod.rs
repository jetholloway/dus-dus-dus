mod environment;
mod model;
mod replay_memory;

use crate::*;
use burn::backend::Wgpu;
use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;
use burn::tensor::activation::sigmoid;
use std::cmp::Ordering;
use std::mem::swap;

type Backend = Wgpu;

const INPUT_SIZE: usize = 152;
const OUTPUT_SIZE: usize = 297;

struct LearningPlayer {}

#[derive(Module, Debug)]
struct Model {
    input: Linear<Backend>,
    output: Linear<Backend>,
    activation: Sigmoid<1>,
}

#[derive(Debug)]
struct Sigmoid<const D: usize>;

impl<const D: usize> Sigmoid<1> {
    pub fn forward(&self, tensor: Tensor<Backend, D>) -> Tensor<Backend, D> {
        sigmoid(tensor)
    }
}

#[derive(Config, Debug)]
struct ModelConfig {
    hidden_size: usize,
}

impl ModelConfig {
    pub fn init(&self, device: &Backend::Device) -> Model {
        Model {
            input: LinearConfig::new(INPUT_SIZE, self.hidden_size).init(device),
            output: LinearConfig::new(self.hidden_size, OUTPUT_SIZE).init(device),
            activation: Sigmoid,
        }
    }
}

impl Model {
    pub fn forward(&self, game_state: Tensor<Backend, 1>) -> Tensor<Backend, 1> {
        let x = self.input.forward(game_state);
        let x = self.activation.forward(x);
        let x = self.output.forward(x);
        let x = self.activation.forward(x);
        x
    }
}

impl LearningPlayer {}

impl GameState {
    fn to_tensor(&self, device: &Backend::Device) -> Tensor<Backend, 1> {
        let mut data = [0f32; INPUT_SIZE];

        const TURN_OFFSET: usize = 0;

        if self.setup() {
            match self.current_player() {
                Player::First => data[TURN_OFFSET + 0] = 1f32,
                Player::Second => data[TURN_OFFSET + 1] = 1f32,
            }
        }

        match self.action_count() {
            ActionCount::First => data[TURN_OFFSET + 2] = 1f32,
            ActionCount::Second => data[TURN_OFFSET + 3] = 1f32,
            ActionCount::Third => data[TURN_OFFSET + 4] = 1f32,
        }

        const CURRENT_PIECE_OFFSET: usize = 5;
        const OTHER_PIECE_OFFSET: usize = 54;

        for x in 0..7 {
            for y in 0..7 {
                let position = Position { x, y };
                let offset = position.to_offset();

                match self.space(position.for_player(self.current_player())) {
                    Space::Piece(player) if player == self.current_player() => {
                        data[CURRENT_PIECE_OFFSET + offset] = 1f32
                    }
                    Space::Piece(player) if player == self.other_player() => {
                        data[OTHER_PIECE_OFFSET + offset] = 1f32
                    }
                    _ => {}
                }
            }
        }

        const BALL_OFFSET: usize = 103;
        let offset = self.ball().x + &*self.ball().y;
        data[BALL_OFFSET + offset] = 1f32;

        Tensor::from_floats(data, &device)
    }
}

impl Position {
    fn to_offset(&self) -> usize {
        (self.x + self.y * 7).into()
    }

    fn from_offset(offset: usize) -> Self {
        let x = (offset % 7).into();
        let y = (offset / 7).into();
        Self { x, y }
    }

    fn for_player(self, player: Player) -> Self {
        match player {
            Player::First => self,
            Player::Second => Self {
                x: 7 - self.x,
                y: 7 - self.y,
            },
        }
    }
}

impl Action {
    fn from_tensor(tensor: Tensor<Backend, 1>, player: Player) -> Self {
        const MOVE_OFFSET: usize = 0;
        const TACKLE_OFFSET: usize = 99;
        const PASS_OFFSET: usize = 198;
        const FROM_OFFSET: usize = 1;
        const TO_OFFSET: usize = 50;

        let data = tensor.into_data().value;
        let move_value = data[MOVE_OFFSET].into();
        let tackle_value = data[TACKLE_OFFSET].into();
        let pass_value = data[PASS_OFFSET].into();

        let action_type = if move_value > tackle_value {
            if move_value > pass_value {
                ActionType::Move
            } else {
                ActionType::Pass
            }
        } else if tackle_value > pass_value {
            ActionType::Tackle
        } else {
            ActionType::Pass
        };

        let action_offset = match action_type {
            ActionType::Move => MOVE_OFFSET,
            ActionType::Tackle => TACKLE_OFFSET,
            ActionType::Pass => PASS_OFFSET,
        };

        let from = Position::from_offset(
            (0..49)
                .max_by_key(|offset| OrdF32(data[action_offset + FROM_OFFSET + offset]))
                .unwrap(),
        );
        let to = Position::from_offset(
            (0..49)
                .max_by_key(|offset| OrdF32(data[action_offset + TO_OFFSET + offset]))
                .unwrap(),
        );

        Self::new(action_type, from.for_player(player), to.for_player(player))
    }
}

pub fn train(device: &Backend::Device) {
    let mut first_player_history = Vec::new();
    let mut second_player_history = Vec::new();

    let mut state = GameState::new();

    let model_config = ModelConfig { hidden_size: 300 };

    let mut model = model_config.init(device);

    loop {
        let action = Action::from_tensor(
            model.forward(state.to_tensor(device)),
            state.current_player(),
        );

        match action.try_apply(&state) {
            ActionResult::Invalid(_) => {}
            ActionResult::Valid {
                state: mut next_state,
            } => match state.current_player() {
                Player::First => {
                    swap(&mut state, &mut next_state);
                    first_player_history.push((next_state, action));
                }
                Player::Second => {
                    swap(&mut state, &mut next_state);
                    second_player_history.push((next_state, action));
                }
            },
            ActionResult::Terminal { .. } => {}
        }
    }
}

struct OrdF32(f32);

impl Eq for OrdF32 {}

impl PartialEq<Self> for OrdF32 {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd<Self> for OrdF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(*other.0)
    }
}
