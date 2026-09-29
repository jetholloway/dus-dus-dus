use engine::{Action, GameState, Player, Position, Space};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::Bot;

/// How much each feature of a position is worth. A position is scored for
/// the player who has just finished a turn: each feature counts for them and
/// the same feature counts against them when the opponent has it. A win is
/// worth +infinity. Only the player moving can win, so a loss never comes up
/// while planning one turn.
///
/// The numbers are starting guesses, meant to be tuned with the arena.
#[derive(Debug, Clone, PartialEq)]
pub struct Weights {
    /// Holding the ball.
    pub possession: f64,
    /// Per rank the ball has advanced from its holder's back rank.
    pub ball_advance: f64,
    /// Per rank each piece has advanced from its owner's back rank.
    pub piece_advance: f64,
    /// Per piece on the opponent's back rank, ready to receive a winning pass.
    pub receiver: f64,
    /// The ball holder has a clear passing line to one of those pieces, so
    /// they threaten to win on their next action.
    pub open_lane: f64,
    /// Per opponent piece sharing an edge with the ball holder, so able to
    /// tackle straight away.
    pub carrier_adjacent: f64,
    /// Per opponent piece two or three squares from the ball holder, close
    /// enough to move in and tackle during one turn.
    pub carrier_nearby: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            possession: 10.0,
            ball_advance: 2.0,
            piece_advance: 0.5,
            receiver: 3.0,
            open_lane: 20.0,
            carrier_adjacent: -6.0,
            carrier_nearby: -2.0,
        }
    }
}

impl Weights {
    pub const NAMES: [&'static str; 7] = [
        "possession",
        "ball_advance",
        "piece_advance",
        "receiver",
        "open_lane",
        "carrier_adjacent",
        "carrier_nearby",
    ];

    fn get_mut(&mut self, name: &str) -> Option<&mut f64> {
        Some(match name {
            "possession" => &mut self.possession,
            "ball_advance" => &mut self.ball_advance,
            "piece_advance" => &mut self.piece_advance,
            "receiver" => &mut self.receiver,
            "open_lane" => &mut self.open_lane,
            "carrier_adjacent" => &mut self.carrier_adjacent,
            "carrier_nearby" => &mut self.carrier_nearby,
            _ => return None,
        })
    }

    /// The defaults with some weights replaced, written like
    /// `possession=12,open_lane=30`.
    pub fn with_overrides(overrides: &str) -> Result<Self, String> {
        let mut weights = Self::default();

        for pair in overrides.split(',').filter(|pair| !pair.trim().is_empty()) {
            let (name, value) = pair
                .split_once('=')
                .ok_or_else(|| format!("expected name=value, got {pair:?}"))?;
            let name = name.trim();
            let weight = weights.get_mut(name).ok_or_else(|| {
                format!(
                    "unknown weight {name:?}; expected one of: {}",
                    Self::NAMES.join(", ")
                )
            })?;
            *weight = value
                .trim()
                .parse()
                .map_err(|_| format!("weight {name} expects a number, got {value:?}"))?;
        }

        Ok(weights)
    }

    /// The value of `state` to `player`, who has just finished a turn.
    pub fn score(&self, state: &GameState, player: Player) -> f64 {
        if state.winner() == Some(player) {
            return f64::INFINITY;
        }
        if state.winner() == Some(opponent(player)) {
            return f64::NEG_INFINITY;
        }

        let mut score = 0.0;
        for position in positions() {
            if let Space::Piece(owner) = state.space(position) {
                score += sign(owner, player)
                    * (self.piece_advance * advance(position, owner) as f64
                        + if position.y == target_rank(owner) {
                            self.receiver
                        } else {
                            0.0
                        });
            }
        }

        let ball = state.ball();
        if let Space::Piece(holder) = state.space(ball) {
            let (adjacent, nearby) = markers(state, ball, holder);
            score += sign(holder, player)
                * (self.possession
                    + self.ball_advance * advance(ball, holder) as f64
                    + if has_open_lane(state, ball, holder) {
                        self.open_lane
                    } else {
                        0.0
                    }
                    + self.carrier_adjacent * adjacent as f64
                    + self.carrier_nearby * nearby as f64);
        }

        score
    }
}

/// Plans the rest of its turn by trying every sequence of actions that
/// finishes it, then plays the first action of the best-scoring sequence.
/// It plans afresh on each call, so it keeps no memory between actions.
/// Ties are broken at random from its seed.
pub struct HeuristicBot {
    weights: Weights,
    rng: StdRng,
}

impl HeuristicBot {
    pub fn new(weights: Weights, seed: u64) -> Self {
        Self {
            weights,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// The best score reachable by the end of `player`'s turn from `state`.
    fn best_score(&self, state: &GameState, player: Player) -> f64 {
        if state.is_terminal() || state.current_player() != player {
            return self.weights.score(state, player);
        }

        state
            .valid_transitions()
            .iter()
            .map(|transition| self.best_score(&transition.state, player))
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

impl Bot for HeuristicBot {
    fn name(&self) -> &str {
        "heuristic"
    }

    fn choose(&mut self, state: &GameState) -> Option<Action> {
        let player = state.current_player();
        let mut best = Vec::new();
        let mut best_score = f64::NEG_INFINITY;

        for transition in state.valid_transitions() {
            let score = self.best_score(&transition.state, player);
            if score > best_score || best.is_empty() {
                best_score = score;
                best.clear();
            }
            if score == best_score {
                best.push(transition.action);
            }
        }

        best.choose(&mut self.rng).cloned()
    }
}

fn positions() -> impl Iterator<Item = Position> {
    (0..7i8).flat_map(|y| (0..7i8).map(move |x| Position { x, y }))
}

fn sign(owner: Player, player: Player) -> f64 {
    if owner == player {
        1.0
    } else {
        -1.0
    }
}

fn opponent(player: Player) -> Player {
    match player {
        Player::First => Player::Second,
        Player::Second => Player::First,
    }
}

/// The rank `player` must get the ball to.
fn target_rank(player: Player) -> i8 {
    match player {
        Player::First => 6,
        Player::Second => 0,
    }
}

/// Ranks advanced from `player`'s own back rank, 0 to 6.
fn advance(position: Position, player: Player) -> i8 {
    match player {
        Player::First => position.y,
        Player::Second => 6 - position.y,
    }
}

/// Opponent pieces sharing an edge with the ball holder, and those two or
/// three squares away.
fn markers(state: &GameState, ball: Position, holder: Player) -> (usize, usize) {
    let mut adjacent = 0;
    let mut nearby = 0;

    for position in positions() {
        if state.space(position) == Space::Piece(opponent(holder)) {
            match position.taxicab_distance(ball) {
                1 => adjacent += 1,
                2 | 3 => nearby += 1,
                _ => {}
            }
        }
    }

    (adjacent, nearby)
}

/// Whether the ball holder could pass straight to one of their pieces on the
/// winning rank. Only the opponent's pieces block a pass.
fn has_open_lane(state: &GameState, ball: Position, holder: Player) -> bool {
    let rank = target_rank(holder);

    (0..7i8)
        .map(|x| Position { x, y: rank })
        .filter(|&receiver| receiver != ball && state.space(receiver) == Space::Piece(holder))
        .filter_map(|receiver| {
            ball.try_get_orthogonal_path(receiver)
                .or_else(|| ball.try_get_diagonal_path(receiver))
        })
        .any(|path| {
            path.iter()
                .all(|&square| state.space(square) != Space::Piece(opponent(holder)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RandomBot;
    use engine::ActionResult;

    fn apply(state: &GameState, action: &Action) -> Option<GameState> {
        match action.try_apply(state) {
            ActionResult::Invalid(error) => panic!("{action} rejected: {error}"),
            ActionResult::Valid { state } => Some(state),
            ActionResult::Terminal { .. } => None,
        }
    }

    /// Positions at the start of a turn from seeded random games, up to the
    /// end of each game.
    fn turn_starts(games: u64) -> Vec<GameState> {
        let mut starts = Vec::new();

        for seed in 0..games {
            let mut bot = RandomBot::new(seed);
            let mut state = GameState::new();
            loop {
                if state.turn_count() > 0 && state.action_count() == engine::ActionCount::First {
                    starts.push(state.clone());
                }
                let action = bot.choose(&state).expect("a legal action");
                match apply(&state, &action) {
                    Some(next) => state = next,
                    None => break,
                }
            }
        }

        starts
    }

    /// Whether the current player can win within the actions left this turn.
    fn can_win_this_turn(state: &GameState) -> bool {
        let player = state.current_player();
        state.valid_transitions().iter().any(|transition| {
            transition.winner.is_some()
                || (transition.state.current_player() == player
                    && can_win_this_turn(&transition.state))
        })
    }

    /// Plays the bot until its turn ends; returns whether it won.
    fn play_turn(bot: &mut HeuristicBot, mut state: GameState) -> bool {
        let player = state.current_player();
        while state.current_player() == player {
            let action = bot.choose(&state).expect("a legal action");
            match apply(&state, &action) {
                Some(next) => state = next,
                None => return true,
            }
        }
        false
    }

    #[test]
    fn takes_a_win_whenever_one_is_available() {
        let mut bot = HeuristicBot::new(Weights::default(), 0);
        let mut chances = 0;

        for state in turn_starts(4).into_iter().step_by(4) {
            if chances == 10 {
                break;
            }
            if can_win_this_turn(&state) {
                chances += 1;
                assert!(
                    play_turn(&mut bot, state.clone()),
                    "missed a win:\n{state:?}"
                );
            }
        }

        assert_eq!(chances, 10, "too few positions with a win were tried");
    }

    #[test]
    fn opening_is_a_legal_setup_move() {
        let mut bot = HeuristicBot::new(Weights::default(), 0);
        let state = GameState::new();
        let action = bot.choose(&state).expect("a legal action");
        assert!(state.valid_actions().contains(&action));
    }

    #[test]
    fn overrides_change_only_the_named_weights() {
        let weights = Weights::with_overrides("possession=1.5, open_lane=-2").unwrap();
        assert_eq!(weights.possession, 1.5);
        assert_eq!(weights.open_lane, -2.0);
        assert_eq!(weights.receiver, Weights::default().receiver);
    }

    #[test]
    fn every_weight_name_can_be_overridden() {
        for name in Weights::NAMES {
            assert!(
                Weights::with_overrides(&format!("{name}=0")).is_ok(),
                "{name}"
            );
        }
    }

    #[test]
    fn bad_overrides_are_errors() {
        assert!(Weights::with_overrides("luck=3").is_err());
        assert!(Weights::with_overrides("possession").is_err());
        assert!(Weights::with_overrides("possession=lots").is_err());
    }

    #[test]
    fn score_is_the_same_from_either_side_with_the_sign_flipped() {
        let weights = Weights::default();
        for state in turn_starts(2) {
            let first = weights.score(&state, Player::First);
            let second = weights.score(&state, Player::Second);
            assert_eq!(first, -second, "{state:?}");
        }
    }
}
