mod arena;
mod game_player;
mod play;

use std::process::ExitCode;
use std::str::FromStr;
use text_io::read;

use arena::{ArenaOptions, Opening};
use bots::{make_bot, Bot, BOT_NAMES};
use engine::*;

const USAGE: &str = "\
usage:
  dus_dus_dus arena <bot-a> <bot-b> [--trials N] [--opening start|random:N]
                    [--openings FILE] [--seed S] [--max-actions M] [--threads T] [--json]
      Play bot A against bot B and report win rates. Each trial is two games
      from the same opening, A playing Orange in one and Teal in the other.
      --opening random:N starts each trial from N turns of random play, each
      side's setup move counting as a turn; --openings reads one opening per
      line, actions separated by commas. --json prints the results as JSON.
      Defaults: 500 trials, the normal start, seed 0, a draw at 3000 actions,
      one thread per CPU.
  dus_dus_dus play
      Play in the terminal. Each side is `console` (you type actions) or a bot.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("arena") => parse_arena(&args[1..]).and_then(|options| arena::run(&options)),
        Some("play") if args.len() == 1 => {
            main_play();
            Ok(())
        }
        Some("help" | "-h" | "--help") => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err("expected a command".into()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn parse_arena(args: &[String]) -> Result<ArenaOptions, String> {
    let mut bots = Vec::new();
    let mut options = ArenaOptions {
        bot_a: String::new(),
        bot_b: String::new(),
        trials: 500,
        seed: 0,
        max_actions: 3000,
        threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
        opening: Opening::Start,
        json: false,
    };
    let mut opening_given = false;

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if !arg.starts_with("--") {
            bots.push(arg.clone());
            continue;
        }
        if arg == "--json" {
            options.json = true;
            continue;
        }
        let value = args.next().ok_or_else(|| format!("{arg} needs a value"))?;
        match arg.as_str() {
            "--trials" => options.trials = parse_number(arg, value)?,
            "--opening" | "--openings" if opening_given => {
                return Err("give only one of --opening and --openings".into())
            }
            "--opening" => {
                options.opening = Opening::parse(value)?;
                opening_given = true;
            }
            "--openings" => {
                options.opening = Opening::from_file(value)?;
                opening_given = true;
            }
            "--seed" => options.seed = parse_number(arg, value)?,
            "--max-actions" => options.max_actions = parse_number(arg, value)?,
            "--threads" => options.threads = parse_number(arg, value)?,
            _ => return Err(format!("unknown option {arg}")),
        }
    }

    let [a, b] = <[String; 2]>::try_from(bots)
        .map_err(|bots| format!("expected two bots, got {}", bots.len()))?;
    options.bot_a = a;
    options.bot_b = b;
    Ok(options)
}

fn parse_number<T: FromStr>(option: &str, value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{option} expects a number, got {value:?}"))
}

fn main_play() {
    let mut first = read_player(Player::First);
    let mut second = read_player(Player::Second);

    play::play(first.as_mut(), second.as_mut(), usize::MAX, true);
}

fn read_player(player: Player) -> Box<dyn Bot> {
    loop {
        print!("Player {player} (console or {}): ", BOT_NAMES.join(", "));
        let line: String = read!("{}\n");
        if line.trim().eq_ignore_ascii_case("console") {
            return Box::new(ConsolePlayer);
        }
        match make_bot(&line, rand_seed()) {
            Ok(bot) => return bot,
            Err(error) => eprintln!("Input error: {error}"),
        }
    }
}

/// A seed that differs between runs, so terminal games vary.
fn rand_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |time| time.as_nanos() as u64)
}

/// Reads actions from the terminal until one is legal.
pub struct ConsolePlayer;

impl Bot for ConsolePlayer {
    fn name(&self) -> &str {
        "console"
    }

    fn choose(&mut self, state: &GameState) -> Option<Action> {
        if state.valid_actions().is_empty() {
            return None;
        }
        loop {
            print!("Player {} move: ", state.current_player());
            let line: String = read!("{}\n");
            match Action::from_str(line.as_str()) {
                Ok(action) => match action.try_apply(state) {
                    ActionResult::Invalid(error) => eprintln!("Invalid action: {error}"),
                    _ => return Some(action),
                },
                Err(error) => eprintln!("Input error: {error}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn arena_defaults() {
        let options = parse_arena(&args("random random")).unwrap();
        assert_eq!(
            (options.trials, options.seed, options.max_actions),
            (500, 0, 3000)
        );
        assert_eq!(options.opening, Opening::Start);
        assert!(!options.json);
    }

    #[test]
    fn arena_options_in_any_order() {
        let options = parse_arena(&args(
            "--seed 9 random --trials 10 --json random --opening random:4",
        ))
        .unwrap();
        assert_eq!(
            (options.bot_a.as_str(), options.bot_b.as_str()),
            ("random", "random")
        );
        assert_eq!((options.trials, options.seed), (10, 9));
        assert_eq!(options.opening, Opening::Random { turns: 4 });
        assert!(options.json);
    }

    #[test]
    fn arena_rejects_bad_arguments() {
        assert!(parse_arena(&args("random")).is_err());
        assert!(parse_arena(&args("random random --trials")).is_err());
        assert!(parse_arena(&args("random random --trials many")).is_err());
        assert!(parse_arena(&args("random random --games 10")).is_err());
        assert!(parse_arena(&args("random random --opening book")).is_err());
        assert!(parse_arena(&args("random random --opening start --opening start")).is_err());
        assert!(parse_arena(&args("random random --speed 3")).is_err());
    }
}
