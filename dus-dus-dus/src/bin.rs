mod game;
mod game_player;

use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng, RngCore};
use std::io::{stdout, Write};
use std::str::FromStr;
use text_io::read;

use game::*;
use game_player::*;

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
    u0to6: Uniform<i8>,
    positions: [Position; 49],
}

impl RandomPlayer {
    const MOVE_TOS: [Displacement; 8] = [
        Displacement { x: -2, y: 0 },
        Displacement { x: -1, y: 0 },
        Displacement { x: 1, y: 0 },
        Displacement { x: 2, y: 0 },
        Displacement { x: 0, y: -2 },
        Displacement { x: 0, y: -1 },
        Displacement { x: 0, y: 1 },
        Displacement { x: 0, y: 2 },
    ];

    const TACKLE_TOS: [Displacement; 9] = [
        Displacement { x: 0, y: 0 },
        Displacement { x: 1, y: 0 },
        Displacement { x: 1, y: 1 },
        Displacement { x: 0, y: 1 },
        Displacement { x: -1, y: 1 },
        Displacement { x: -1, y: 0 },
        Displacement { x: -1, y: -1 },
        Displacement { x: 0, y: -1 },
        Displacement { x: 1, y: -1 },
    ];

    pub fn new() -> Self {
        let mut new = Self {
            rng: Box::new(thread_rng()),
            u0to6: Uniform::from(0..7),
            positions: [Position { x: 0, y: 0 }; 49],
        };

        for x in 0..7i8 {
            for y in 0..7i8 {
                new.positions[(x + 7 * y) as usize] = Position { x, y };
            }
        }

        new
    }

    fn gen_position(&mut self) -> Position {
        let x = self.u0to6.sample(&mut self.rng);
        let y = self.u0to6.sample(&mut self.rng);
        Position { x, y }
    }
}

impl GamePlayer for RandomPlayer {
    fn next_action(&mut self, state: &GameState) -> Action {
        let move_froms: Vec<Position> = self
            .positions
            .iter()
            .filter(|position| state.space(**position) == Space::Piece(state.current_player()))
            .cloned()
            .collect();

        let tackle_froms: Vec<Position> = move_froms
            .iter()
            .filter(|position| position.taxicab_distance(state.ball()) == 1)
            .cloned()
            .collect();

        let pass_froms = if state.space(state.ball()) == Space::Piece(state.current_player()) {
            vec![state.ball()]
        } else {
            Vec::new()
        };

        let mut action_type_froms = Vec::new();

        for move_from in &move_froms {
            action_type_froms.push((ActionType::Move, *move_from));
        }

        for tackle_from in tackle_froms {
            action_type_froms.push((ActionType::Tackle, tackle_from));
        }

        for pass_from in pass_froms {
            action_type_froms.push((ActionType::Pass, pass_from));
        }

        let (action_type, from) =
            &action_type_froms[self.rng.gen_range(0..action_type_froms.len())];

        let to = match action_type {
            ActionType::Move => *from + Self::MOVE_TOS[self.rng.gen_range(0..Self::MOVE_TOS.len())],
            ActionType::Tackle => {
                *from + Self::TACKLE_TOS[self.rng.gen_range(0..Self::TACKLE_TOS.len())]
            }
            ActionType::Pass => move_froms[self.rng.gen_range(0..7)],
        };

        Action::new(*action_type, *from, to)
    }

    fn invalid_action(&mut self, _error: &str) {}
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
