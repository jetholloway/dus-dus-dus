mod arena;
mod game_player;
mod play;

use std::process::ExitCode;
use std::str::FromStr;
use text_io::read;

use arena::ArenaOptions;
use bots::{make_bot, Bot, BOT_NAMES};
use engine::*;

const USAGE: &str = "\
usage:
  dus_dus_dus arena <bot-a> <bot-b> [--games N] [--seed S] [--max-actions M] [--threads T]
      Play bot A against bot B, alternating sides, and report win rates.
      Defaults: 1000 games, seed 0, a draw at 3000 actions, one thread per CPU.
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
        games: 1000,
        seed: 0,
        max_actions: 3000,
        threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
    };

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if !arg.starts_with("--") {
            bots.push(arg.clone());
            continue;
        }
        let value = args.next().ok_or_else(|| format!("{arg} needs a value"))?;
        match arg.as_str() {
            "--games" => options.games = parse_number(arg, value)?,
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
            (options.games, options.seed, options.max_actions),
            (1000, 0, 3000)
        );
    }

    #[test]
    fn arena_options_in_any_order() {
        let options = parse_arena(&args("--seed 9 random --games 10 random")).unwrap();
        assert_eq!(
            (options.bot_a.as_str(), options.bot_b.as_str()),
            ("random", "random")
        );
        assert_eq!((options.games, options.seed), (10, 9));
    }

    #[test]
    fn arena_rejects_bad_arguments() {
        assert!(parse_arena(&args("random")).is_err());
        assert!(parse_arena(&args("random random --games")).is_err());
        assert!(parse_arena(&args("random random --games many")).is_err());
        assert!(parse_arena(&args("random random --speed 3")).is_err());
    }
}
