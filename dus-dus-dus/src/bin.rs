mod game_player;
mod play;

use rand::{thread_rng, Rng, RngCore};
use std::io::{stdout, Write};
use std::str::FromStr;
use text_io::read;

use engine::*;
use game_player::*;
use play::*;

fn main() {
    main_trial();
}

fn main_play() {
    let mut first = read_player(Player::First);
    let mut second = read_player(Player::Second);

    play(first.as_mut(), second.as_mut(), true);
}

fn main_trial() {
    const TRIALS: i32 = 10000;

    let mut first = RandomPlayer::new();
    let mut second = RandomPlayer::new();

    let mut first_count = 0;
    let mut second_count = 0;

    for trial in 1..=TRIALS {
        print!(".");
        let _ = stdout().flush();

        if trial % 100 == 0 {
            println!(
                " PLAYER {} {first_count} vs PLAYER {} {second_count} / {trial}",
                Player::First,
                Player::Second
            );
        }

        match play(&mut first, &mut second, false) {
            Player::First => first_count += 1,
            Player::Second => second_count += 1,
        }
    }

    println!("PLAYER {} WINS {first_count} / {TRIALS}", Player::First);
    println!("PLAYER {} WINS {second_count} / {TRIALS}", Player::Second);
}

fn read_player(player: Player) -> Box<dyn GamePlayer> {
    loop {
        print!("Player {} type: ", player);
        let line: String = read!("{}\n");
        match Box::<dyn GamePlayer>::from_str(line.as_str()) {
            Ok(player) => return player,
            Err(error) => eprintln!("Input error: {error}"),
        }
    }
}

pub struct ConsolePlayer;

impl ConsolePlayer {
    fn new() -> Self {
        Self
    }
}

impl GamePlayer for ConsolePlayer {
    fn next_action(&mut self, state: &GameState) -> Action {
        loop {
            print!("Player {} move: ", state.current_player());
            let line: String = read!("{}\n");
            match Action::from_str(line.as_str()) {
                Ok(action) => return action,
                Err(error) => eprintln!("Input error: {error}"),
            }
        }
    }

    fn invalid_action(&mut self, error: &str) {
        eprintln!("Invalid action: {error}");
    }
}

pub struct RandomPlayer {
    rng: Box<dyn RngCore>,
}

impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            rng: Box::new(thread_rng()),
        }
    }
}

impl GamePlayer for RandomPlayer {
    fn next_action(&mut self, state: &GameState) -> Action {
        let actions = state.valid_actions();
        actions[self.rng.gen_range(0..actions.len())].clone()
    }

    fn invalid_action(&mut self, error: &str) {
        panic!("RandomPlayer chose an invalid action: {error}");
    }
}

impl FromStr for Box<dyn GamePlayer> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim().to_ascii_uppercase();
        if "CONSOLE".starts_with(input.as_str()) {
            Ok(Box::new(ConsolePlayer::new()))
        } else if "RANDOM".starts_with(input.as_str()) {
            Ok(Box::new(RandomPlayer::new()))
        } else {
            Err(format!("Invalid player: {s} not like CONSOLE or RANDOM"))
        }
    }
}
