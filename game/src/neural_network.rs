use crate::default_map::DefaultMap;
use crate::game::Game;
use crate::real::Real;

pub trait NeuralNetwork<G: Game> {
    fn new_random() -> Self;

    fn action_probability_and_reward<'a>(
        &self,
        state: &'a G::State,
    ) -> (DefaultMap<&'a G::Action, Real>, Real);
}
