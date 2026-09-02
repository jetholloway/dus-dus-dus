use crate::game::{Agent, Game, Player};

pub trait PlayOne<P: Player> {
    fn play<G: Game<Player = P>, A: Agent<G>>(game: &G, agent: &A) -> Option<P> {}
}
