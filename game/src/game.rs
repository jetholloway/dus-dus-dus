use std::hash::Hash;

use crate::real::Real;

pub trait Game {
    type State: State;
    type Player: Player;
    type Action: Action;

    fn initial_state(&self) -> &Self::State;

    fn next_state(&self, state: &Self::State, action: &Self::Action) -> Option<&Self::State>;

    fn acting_player(&self, state: &Self::State) -> &Self::Player;

    fn valid_actions(&self, state: &Self::State) -> impl Iterator<Item = &Self::Action>;

    fn is_terminal(&self, state: &Self::State) -> bool;

    fn reward(&self, state: &Self::State) -> Real;
}

pub trait Agent<G: Game> {
    fn next_action(&mut self, game: &G, state: &G::State) -> &G::Action;
}

pub trait State: Eq + PartialEq + Hash {}

pub trait Player: Eq + PartialEq + Hash {}

pub trait Action: Eq + PartialEq + Hash {}

#[derive(Eq, PartialEq, Hash)]
pub struct OnePlayer;

impl Player for OnePlayer {}

#[derive(Eq, PartialEq, Hash)]
pub enum TwoPlayer {
    A,
    B,
}

impl Player for TwoPlayer {}
