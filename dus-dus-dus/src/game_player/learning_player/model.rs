use burn::grad_clipping::{GradientClipping, GradientClippingConfig};
use burn::nn::{Linear, LinearConfig, Relu};
use burn::optim::AdamWConfig;
use burn::prelude::*;
use burn::tensor::backend::AutodiffBackend;
use rand::random;

#[derive(Config)]
pub struct TrainingConfig {
    #[config(default = 1)]
    num_episodes: usize,
    #[config(default = 128)]
    batch_size: usize,
    #[config(default = 0.99)]
    gamma: f64,
    #[config(default = 0.9)]
    eps_start: f64,
    #[config(default = 0.05)]
    eps_end: f64,
    #[config(default = 1000.0)]
    eps_decay: f64,
    #[config(default = 0.005)]
    tau: f64,
    #[config(default = 0.0001)]
    lr: f64,
    #[config(default = 100.0)]
    gradient_clipping: f32,
    model: ModelConfig,
}

trait Agent<E: Environment> {}

struct LearningAgent<E: Environment, B: Backend> {}

impl<E: Environment, B: Backend> LearningAgent<E, B> {
    fn new(model: Model<E, B>) -> Self;

    fn model(&self) -> Model<E, B>;
}

impl<E: Environment, B: Backend> Agent<E> for LearningAgent<E, B> {}

impl TrainingConfig {
    fn train<E: Environment, B: AutodiffBackend>(&self, device: &B::Device) -> impl Agent<E> {
        let mut env = E::new();

        let model = self.model.init(device);

        let agent = LearningAgent::new();

        let mut optimiser = AdamWConfig::new()
            .with_grad_clipping(Some(GradientClippingConfig::Value(self.gradient_clipping)))
            .init();

        let mut policy_net = agent.model().clone();

        let mut step = 0;

        for episode in 0..self.num_episodes {
            let mut episode_done = false;
            let mut episode_reward = 0.0;
            let mut episode_duration = 0;
            let mut state = env.state();

            while !episode_done {
                let eps_threshold = self.eps_end
                    + (self.eps_start - self.eps_end) * f64::exp(-(step as f64) / self.eps_decay);
                let action = if random::<f64>() > eps_threshold {
                    E::ActionType::from_tensor(policy_net.forward(state.to_tensor()))
                } else {
                    E::ActionType::random()
                };
            }
        }
    }
}

trait Environment {
    type StateType: State;
    type ActionType: Action;

    fn new() -> Self;

    fn state(&self) -> Self::StateType {}

    const INPUT_SIZE: usize;
    const OUTPUT_SIZE: usize;
}

trait State {
    fn to_tensor<B: Backend>(&self) -> Tensor<B, 2>;
}

trait Action {
    fn from_tensor<B: Backend>(tensor: Tensor<B, 2>) -> Self;

    fn random() -> Self;
}

#[derive(Config)]
pub struct ModelConfig {
    #[config(default = 256)]
    hidden_size: usize,
}

impl ModelConfig {
    fn init<E: Environment, B: Backend>(&self, device: &B::Device) -> Model<E, B> {
        Model {
            linear0: LinearConfig::new(E::INPUT_SIZE, self.hidden_size).init(device),
            linear1: LinearConfig::new(self.hidden_size, self.hidden_size).init(device),
            linear2: Linear::new(self.hidden_size, E::OUTPUT_SIZE).init(device),
            activation: Relu::new(),
        }
    }
}

#[derive(Module, Debug)]
pub struct Model<E: Environment, B: Backend> {
    linear0: Linear<B>,
    linear1: Linear<B>,
    linear2: Linear<B>,
    activation: Relu,
}

impl<E: Environment, B: Backend> Model<E, B> {
    fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.linear0.forward(x);
        let x = self.activation.forward(x);
        let x = self.linear1.forward(x);
        let x = self.activation.forward(x);
        self.linear2.forward(x)
    }
}
