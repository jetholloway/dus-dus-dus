use bots::make_bot;
use engine::Player;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::play::{play, GameOutcome};

pub struct ArenaOptions {
    pub bot_a: String,
    pub bot_b: String,
    pub games: usize,
    pub seed: u64,
    pub max_actions: usize,
    pub threads: usize,
}

/// A mixing function so nearby seeds give unrelated streams (SplitMix64).
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Bot A plays First in even-numbered games and Second in odd ones. Each
/// game's bots are seeded from the match seed and the game number, so a
/// match gives the same results whatever the thread count.
fn play_game(options: &ArenaOptions, game: usize) -> GameOutcome {
    let game_seed = mix(options.seed ^ mix(game as u64));
    let mut a = make_bot(&options.bot_a, mix(game_seed)).expect("checked before the match");
    let mut b = make_bot(&options.bot_b, mix(game_seed ^ 1)).expect("checked before the match");

    if game % 2 == 0 {
        play(a.as_mut(), b.as_mut(), options.max_actions, false)
    } else {
        play(b.as_mut(), a.as_mut(), options.max_actions, false)
    }
}

pub fn run(options: &ArenaOptions) -> Result<(), String> {
    make_bot(&options.bot_a, 0)?;
    make_bot(&options.bot_b, 0)?;
    if options.games == 0 {
        return Err("--games must be at least 1".into());
    }

    let next_game = AtomicUsize::new(0);
    let outcomes: Mutex<Vec<Option<GameOutcome>>> =
        Mutex::new((0..options.games).map(|_| None).collect());
    let finished = AtomicUsize::new(0);

    thread::scope(|scope| {
        for _ in 0..options.threads.max(1) {
            scope.spawn(|| loop {
                let game = next_game.fetch_add(1, Ordering::Relaxed);
                if game >= options.games {
                    break;
                }
                let outcome = play_game(options, game);
                outcomes.lock().unwrap()[game] = Some(outcome);
                let done = finished.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 100 == 0 || done == options.games {
                    eprint!("\r{done} / {} games", options.games);
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
    print!("{}", report(options, &outcomes));
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
}

#[derive(Default)]
struct Summary {
    a: Tally,
    b: Tally,
    capped: usize,
    stuck: usize,
    total_actions: usize,
}

fn summarize(options: &ArenaOptions, outcomes: &[GameOutcome]) -> Summary {
    let mut summary = Summary::default();
    let Summary {
        a,
        b,
        capped,
        stuck,
        total_actions,
    } = &mut summary;

    for (game, outcome) in outcomes.iter().enumerate() {
        let a_is_first = game % 2 == 0;
        let (first, second) = if a_is_first {
            (&mut *a, &mut *b)
        } else {
            (&mut *b, &mut *a)
        };

        match outcome.winner {
            Some(Player::First) => first.wins_as_first += 1,
            Some(Player::Second) => second.wins_as_second += 1,
            None if outcome.actions >= options.max_actions => *capped += 1,
            None => *stuck += 1,
        }
        first.think += outcome.think[0].0;
        first.choices += outcome.think[0].1;
        second.think += outcome.think[1].0;
        second.choices += outcome.think[1].1;
        *total_actions += outcome.actions;
    }

    summary
}

fn report(options: &ArenaOptions, outcomes: &[GameOutcome]) -> String {
    let Summary {
        a,
        b,
        capped,
        stuck,
        total_actions,
    } = summarize(options, outcomes);
    let games = outcomes.len();
    let width = 16.max(2 + options.bot_a.len().max(options.bot_b.len()));
    let row = |label: String, tally: &Tally| {
        let wins = tally.wins();
        let rate = wins as f64 / games as f64;
        let margin = 1.96 * (rate * (1.0 - rate) / games as f64).sqrt();
        let think = if tally.choices == 0 {
            Duration::ZERO
        } else {
            tally.think / tally.choices as u32
        };
        format!(
            "{label:<width$} {wins:>6} {:>6} {:>6}  {:>5.1}% ± {:>4.1}%  {think:>10.1?}\n",
            tally.wins_as_first,
            tally.wins_as_second,
            100.0 * rate,
            100.0 * margin,
        )
    };

    let mut out = format!(
        "{} vs {}: {games} games, seed {}, draw after {} actions\n\n",
        options.bot_a, options.bot_b, options.seed, options.max_actions
    );
    out += &format!(
        "{:<width$} {:>6} {:>6} {:>6}  {:>14}  {:>10}\n",
        "", "wins", "as O", "as X", "win rate", "per action"
    );
    out += &row(format!("A {}", options.bot_a), &a);
    out += &row(format!("B {}", options.bot_b), &b);
    out += &format!(
        "{:<width$} {:>6}  ({capped} at the cap, {stuck} with no legal action)\n\n",
        "draws",
        capped + stuck
    );
    out += &format!(
        "mean game length {:.0} actions\n",
        total_actions as f64 / games as f64
    );
    out += "win rate margin is a 95% confidence interval\n";
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> ArenaOptions {
        ArenaOptions {
            bot_a: "random".into(),
            bot_b: "random".into(),
            games: 20,
            seed: 5,
            max_actions: 3000,
            threads: 1,
        }
    }

    fn winners(options: &ArenaOptions) -> Vec<Option<Player>> {
        (0..options.games)
            .map(|game| play_game(options, game).winner)
            .collect()
    }

    #[test]
    fn a_game_replays_the_same_from_the_match_seed() {
        assert_eq!(winners(&options()), winners(&options()));
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
    fn unknown_bot_is_an_error() {
        let mut options = options();
        options.bot_b = "genius".into();
        assert!(run(&options).is_err());
    }
}
