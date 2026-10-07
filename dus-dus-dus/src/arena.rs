use bots::{make_bot, Bot, RandomBot};
use engine::{Action, ActionResult, GameState, Player};
use serde_json::json;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::play::{play_from, GameOutcome};

pub struct ArenaOptions {
    pub bot_a: String,
    pub bot_b: String,
    pub trials: usize,
    pub seed: u64,
    pub max_actions: usize,
    pub threads: usize,
    pub opening: Opening,
    pub json: bool,
}

/// Where each trial's two games start.
#[derive(Debug, Clone, PartialEq)]
pub enum Opening {
    /// The normal start of the game.
    Start,
    /// Random legal play for this many turns, counting each side's setup
    /// move as a turn. Drawn afresh for each trial.
    Random { turns: usize },
    /// Positions after each of these action lists, used in turn.
    List(Vec<Vec<Action>>),
}

impl Opening {
    /// `start` or `random:N`. A file of openings is read by `from_file`.
    pub fn parse(spec: &str) -> Result<Self, String> {
        if spec == "start" {
            return Ok(Self::Start);
        }
        spec.strip_prefix("random:")
            .and_then(|turns| turns.parse().ok())
            .filter(|&turns| turns > 0)
            .map(|turns| Self::Random { turns })
            .ok_or_else(|| format!("expected --opening start or random:N, got {spec:?}"))
    }

    /// One opening per line, its actions separated by commas, such as
    /// `MOVE D1 D3, MOVE B7 B5`. Blank lines and `#` comments are ignored.
    /// An opening the rules forbid, or that already ends the game, is
    /// skipped with a warning.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|error| format!("can't read {path}: {error}"))?;
        let mut openings = Vec::new();

        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match parse_opening(line) {
                Ok(actions) => openings.push(actions),
                Err(error) => eprintln!("{path}:{}: skipped: {error}", index + 1),
            }
        }

        if openings.is_empty() {
            return Err(format!("{path} has no usable openings"));
        }
        Ok(Self::List(openings))
    }

    fn describe(&self) -> String {
        match self {
            Opening::Start => "start".into(),
            Opening::Random { turns } => format!("random:{turns}"),
            Opening::List(openings) => format!("{} from a file", openings.len()),
        }
    }

    /// The position trial `trial` starts from, and the actions leading there.
    fn position(&self, seed: u64, trial: usize) -> GameState {
        match self {
            Opening::Start => GameState::new(),
            Opening::Random { turns } => random_opening(*turns, seed, trial),
            Opening::List(openings) => {
                replay(&openings[trial % openings.len()]).expect("checked when read")
            }
        }
    }
}

fn parse_opening(line: &str) -> Result<Vec<Action>, String> {
    let actions = line
        .split(',')
        .map(|action| Action::from_str(action.trim()))
        .collect::<Result<Vec<_>, _>>()?;
    let state = replay(&actions)?;
    if state.is_terminal() {
        return Err("the game is already over".into());
    }
    Ok(actions)
}

fn replay(actions: &[Action]) -> Result<GameState, String> {
    let mut state = GameState::new();
    for action in actions {
        state = match action.try_apply(&state) {
            ActionResult::Invalid(error) => return Err(format!("{action}: {error}")),
            ActionResult::Valid { state } | ActionResult::Terminal { state, .. } => state,
        };
    }
    Ok(state)
}

/// Random play for `turns` turns. An opening that ends the game, or leaves a
/// side with no legal action, is thrown away and drawn again.
fn random_opening(turns: usize, seed: u64, trial: usize) -> GameState {
    for attempt in 0u64.. {
        let mut bot = RandomBot::new(mix(seed ^ mix(trial as u64) ^ mix(!attempt)));
        let mut state = GameState::new();
        let mut finished_turns = 0;

        while finished_turns < turns {
            let player = state.current_player();
            let Some(action) = bot.choose(&state) else {
                break;
            };
            match action.try_apply(&state) {
                ActionResult::Valid { state: next } => state = next,
                _ => break,
            }
            if state.current_player() != player {
                finished_turns += 1;
            }
        }

        if finished_turns == turns {
            return state;
        }
    }
    unreachable!("an endless loop only ends by returning")
}

/// A mixing function so nearby seeds give unrelated streams (SplitMix64).
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Each trial is two games from the same opening: game 2t with bot A
/// playing Orange (First), game 2t + 1 with A playing Teal. Bots are seeded
/// from the match seed and the game number, so a match gives the same
/// results whatever the thread count.
fn play_game(options: &ArenaOptions, game: usize) -> GameOutcome {
    let state = options.opening.position(options.seed, game / 2);
    let game_seed = mix(options.seed ^ mix(game as u64));
    let mut a = bot(&options.bot_a, mix(game_seed));
    let mut b = bot(&options.bot_b, mix(game_seed ^ 1));

    if a_is_first(game) {
        play_from(state, a.as_mut(), b.as_mut(), options.max_actions, false)
    } else {
        play_from(state, b.as_mut(), a.as_mut(), options.max_actions, false)
    }
}

fn a_is_first(game: usize) -> bool {
    game % 2 == 0
}

fn bot(name: &str, seed: u64) -> Box<dyn Bot> {
    make_bot(name, seed).expect("checked before the match")
}

pub fn run(options: &ArenaOptions) -> Result<(), String> {
    make_bot(&options.bot_a, 0)?;
    make_bot(&options.bot_b, 0)?;
    if options.trials == 0 {
        return Err("--trials must be at least 1".into());
    }

    let games = 2 * options.trials;
    let next_game = AtomicUsize::new(0);
    let outcomes: Mutex<Vec<Option<GameOutcome>>> = Mutex::new((0..games).map(|_| None).collect());
    let finished = AtomicUsize::new(0);

    thread::scope(|scope| {
        for _ in 0..options.threads.max(1) {
            scope.spawn(|| loop {
                let game = next_game.fetch_add(1, Ordering::Relaxed);
                if game >= games {
                    break;
                }
                let outcome = play_game(options, game);
                outcomes.lock().unwrap()[game] = Some(outcome);
                let done = finished.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 100 == 0 || done == games {
                    eprint!("\r{done} / {games} games");
                }
            });
        }
    });
    eprintln!();

    let outcomes: Vec<GameOutcome> = outcomes
        .into_inner()
        .unwrap()
        .into_iter()
        .map(|outcome| outcome.expect("every game was played"))
        .collect();
    let summary = summarize(options, &outcomes);
    if options.json {
        println!("{}", to_json(options, &summary));
    } else {
        print!("{}", report(options, &summary));
    }
    Ok(())
}

#[derive(Default)]
struct Tally {
    wins_as_first: usize,
    wins_as_second: usize,
    think: Duration,
    choices: usize,
}

impl Tally {
    fn wins(&self) -> usize {
        self.wins_as_first + self.wins_as_second
    }

    fn think_per_action(&self) -> Duration {
        if self.choices == 0 {
            Duration::ZERO
        } else {
            self.think / self.choices as u32
        }
    }
}

/// How each trial's pair of games went.
#[derive(Default, Debug, PartialEq)]
struct Pairs {
    a_both: usize,
    b_both: usize,
    /// One win each: the opening or the colour decided it.
    split: usize,
    /// At least one game drawn.
    with_draw: usize,
}

#[derive(Default)]
struct Summary {
    games: usize,
    a: Tally,
    b: Tally,
    capped: usize,
    stuck: usize,
    total_actions: usize,
    pairs: Pairs,
}

/// Which bot won a game: Some(true) for A, Some(false) for B.
fn a_won(game: usize, outcome: &GameOutcome) -> Option<bool> {
    outcome
        .winner
        .map(|winner| (winner == Player::First) == a_is_first(game))
}

fn summarize(options: &ArenaOptions, outcomes: &[GameOutcome]) -> Summary {
    let mut summary = Summary {
        games: outcomes.len(),
        ..Summary::default()
    };

    for (game, outcome) in outcomes.iter().enumerate() {
        let (first, second) = if a_is_first(game) {
            (&mut summary.a, &mut summary.b)
        } else {
            (&mut summary.b, &mut summary.a)
        };

        match outcome.winner {
            Some(Player::First) => first.wins_as_first += 1,
            Some(Player::Second) => second.wins_as_second += 1,
            None if outcome.actions >= options.max_actions => summary.capped += 1,
            None => summary.stuck += 1,
        }
        first.think += outcome.think[0].0;
        first.choices += outcome.think[0].1;
        second.think += outcome.think[1].0;
        second.choices += outcome.think[1].1;
        summary.total_actions += outcome.actions;
    }

    for (pair, games) in outcomes.chunks(2).enumerate() {
        let results: Vec<Option<bool>> = games
            .iter()
            .enumerate()
            .map(|(index, outcome)| a_won(2 * pair + index, outcome))
            .collect();
        match results[..] {
            [Some(true), Some(true)] => summary.pairs.a_both += 1,
            [Some(false), Some(false)] => summary.pairs.b_both += 1,
            [Some(_), Some(_)] => summary.pairs.split += 1,
            _ => summary.pairs.with_draw += 1,
        }
    }

    summary
}

/// A 95% confidence interval for a proportion (the Wilson score interval),
/// which stays sensible near 0% and 100%.
fn wilson(successes: usize, trials: usize) -> (f64, f64) {
    if trials == 0 {
        return (0.0, 1.0);
    }
    let z = 1.96;
    let n = trials as f64;
    let p = successes as f64 / n;
    let denominator = 1.0 + z * z / n;
    let centre = (p + z * z / (2.0 * n)) / denominator;
    let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / denominator;
    ((centre - half).max(0.0), (centre + half).min(1.0))
}

fn report(options: &ArenaOptions, summary: &Summary) -> String {
    let games = summary.games;
    let width = 16.max(2 + options.bot_a.len().max(options.bot_b.len()));
    let row = |label: String, tally: &Tally| {
        let wins = tally.wins();
        let (low, high) = wilson(wins, games);
        let range = format!("{:.1}–{:.1}%", 100.0 * low, 100.0 * high);
        format!(
            "{label:<width$} {wins:>6} {:>6} {:>6}  {:>5.1}%  {range:>12}  {:>10.1?}\n",
            tally.wins_as_first,
            tally.wins_as_second,
            100.0 * wins as f64 / games as f64,
            tally.think_per_action(),
        )
    };
    let pairs = &summary.pairs;

    let mut out = format!(
        "{} vs {}: {} trials ({games} games), opening {}, seed {}, draw after {} actions\n\n",
        options.bot_a,
        options.bot_b,
        options.trials,
        options.opening.describe(),
        options.seed,
        options.max_actions
    );
    out += &format!(
        "{:<width$} {:>6} {:>6} {:>6}  {:>6}  {:>12}  {:>10}\n",
        "", "wins", "as O", "as X", "rate", "95% range", "per action"
    );
    out += &row(format!("A {}", options.bot_a), &summary.a);
    out += &row(format!("B {}", options.bot_b), &summary.b);
    out += &format!(
        "{:<width$} {:>6}  ({} at the cap, {} with no legal action)\n\n",
        "draws",
        summary.capped + summary.stuck,
        summary.capped,
        summary.stuck,
    );
    out += &format!(
        "trials: A won both {}, B won both {}, split {}, with a draw {}\n",
        pairs.a_both, pairs.b_both, pairs.split, pairs.with_draw
    );
    out += &format!(
        "mean game length {:.0} actions after the opening\n",
        summary.total_actions as f64 / games as f64
    );
    out
}

fn to_json(options: &ArenaOptions, summary: &Summary) -> serde_json::Value {
    let games = summary.games;
    let bot = |name: &str, tally: &Tally| {
        let (low, high) = wilson(tally.wins(), games);
        json!({
            "name": name,
            "wins": tally.wins(),
            "wins_as_first": tally.wins_as_first,
            "wins_as_second": tally.wins_as_second,
            "win_rate": tally.wins() as f64 / games as f64,
            "win_rate_low": low,
            "win_rate_high": high,
            "seconds_per_action": tally.think_per_action().as_secs_f64(),
        })
    };

    json!({
        "a": bot(&options.bot_a, &summary.a),
        "b": bot(&options.bot_b, &summary.b),
        "trials": options.trials,
        "games": games,
        "opening": options.opening.describe(),
        "seed": options.seed,
        "max_actions": options.max_actions,
        "draws": summary.capped + summary.stuck,
        "draws_at_cap": summary.capped,
        "draws_stuck": summary.stuck,
        "pairs": {
            "a_both": summary.pairs.a_both,
            "b_both": summary.pairs.b_both,
            "split": summary.pairs.split,
            "with_draw": summary.pairs.with_draw,
        },
        "mean_game_length": summary.total_actions as f64 / games as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> ArenaOptions {
        ArenaOptions {
            bot_a: "random".into(),
            bot_b: "random".into(),
            trials: 10,
            seed: 5,
            max_actions: 3000,
            threads: 1,
            opening: Opening::Random { turns: 4 },
            json: false,
        }
    }

    fn winners(options: &ArenaOptions) -> Vec<Option<Player>> {
        (0..2 * options.trials)
            .map(|game| play_game(options, game).winner)
            .collect()
    }

    #[test]
    fn a_game_replays_the_same_from_the_match_seed() {
        assert_eq!(winners(&options()), winners(&options()));
    }

    #[test]
    fn both_games_of_a_trial_start_from_the_same_random_opening() {
        let opening = Opening::Random { turns: 4 };
        let first = opening.position(5, 3);

        assert_eq!(first, opening.position(5, 3));
        assert_ne!(first, opening.position(5, 4));
        assert_ne!(first, opening.position(6, 3));
        // Two setup moves, then a turn each, which the engine counts as
        // turns 0, 0, 1 and 2: Orange to move, at the start of turn 3.
        assert_eq!(first.turn_count(), 3);
        assert_eq!(first.current_player(), Player::First);
        assert_eq!(first.action_count(), engine::ActionCount::First);
    }

    #[test]
    fn opening_specs_parse() {
        assert_eq!(Opening::parse("start"), Ok(Opening::Start));
        assert_eq!(Opening::parse("random:4"), Ok(Opening::Random { turns: 4 }));
        for bad in ["random", "random:0", "random:x", "book"] {
            assert!(Opening::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn an_openings_file_skips_lines_the_rules_forbid() {
        let path = std::env::temp_dir().join(format!("openings-{}.txt", std::process::id()));
        std::fs::write(
            &path,
            "# a comment\n\nMOVE D1 D3, MOVE B7 B5\nMOVE D1 D2\nWAFFLE\n",
        )
        .unwrap();

        let opening = Opening::from_file(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).unwrap();

        let Opening::List(openings) = &opening else {
            panic!("expected a list, got {opening:?}");
        };
        assert_eq!(openings.len(), 1);
        assert_eq!(opening.position(0, 7).turn_count(), 1);
    }

    fn outcome(winner: Option<Player>, actions: usize) -> GameOutcome {
        GameOutcome {
            winner,
            actions,
            think: [(Duration::ZERO, 0); 2],
        }
    }

    #[test]
    fn summary_credits_wins_to_the_bot_on_that_side() {
        let options = options();
        // A is First in even games, Second in odd ones.
        let outcomes = [
            outcome(Some(Player::First), 10),  // A as O
            outcome(Some(Player::First), 10),  // B as O
            outcome(Some(Player::Second), 10), // B as X
            outcome(Some(Player::Second), 10), // A as X
            outcome(None, 3000),               // draw at the cap
            outcome(None, 7),                  // draw, no legal action
        ];

        let summary = summarize(&options, &outcomes);

        assert_eq!((summary.a.wins_as_first, summary.a.wins_as_second), (1, 1));
        assert_eq!((summary.b.wins_as_first, summary.b.wins_as_second), (1, 1));
        assert_eq!((summary.capped, summary.stuck), (1, 1));
        assert_eq!(summary.total_actions, 3047);
    }

    #[test]
    fn summary_says_how_each_trial_went() {
        let options = options();
        let outcomes = [
            // A as O wins, A as X wins.
            outcome(Some(Player::First), 10),
            outcome(Some(Player::Second), 10),
            // B as X wins, B as O wins.
            outcome(Some(Player::Second), 10),
            outcome(Some(Player::First), 10),
            // Orange wins both: one each.
            outcome(Some(Player::First), 10),
            outcome(Some(Player::First), 10),
            // A draw.
            outcome(None, 3000),
            outcome(Some(Player::First), 10),
        ];

        let pairs = summarize(&options, &outcomes).pairs;

        assert_eq!(
            pairs,
            Pairs {
                a_both: 1,
                b_both: 1,
                split: 1,
                with_draw: 1
            }
        );
    }

    #[test]
    fn wilson_interval_stays_inside_zero_to_one() {
        let (low, high) = wilson(100, 100);
        assert!(low > 0.95 && low < 1.0 && high > 0.9999, "{low} {high}");
        let (low, high) = wilson(50, 100);
        assert!((low - 0.404).abs() < 0.001 && (high - 0.596).abs() < 0.001);
    }

    #[test]
    fn unknown_bot_is_an_error() {
        let mut options = options();
        options.bot_b = "genius".into();
        assert!(run(&options).is_err());
    }
}
