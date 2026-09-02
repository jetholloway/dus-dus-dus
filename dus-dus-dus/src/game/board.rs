use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone)]
pub struct Board {
    spaces: [[Space; 7]; 7],
    ball: Position,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Space {
    Invalid,
    Empty,
    Piece(Player),
}

#[derive(Eq, PartialEq)]
struct ClosestPositionToRank {
    position: Position,
    y: i8,
}

impl Board {
    pub(super) fn new() -> Self {
        let mut new = Self {
            spaces: [
                [Space::Empty; 7],
                [Space::Empty; 7],
                [Space::Empty; 7],
                [Space::Empty; 7],
                [Space::Empty; 7],
                [Space::Empty; 7],
                [Space::Empty; 7],
            ],
            ball: Position { x: -1, y: -1 },
        };

        for x in 0..7 {
            new.set_space(Position { x, y: 0 }, Space::Piece(Player::First));
            new.set_space(Position { x, y: 6 }, Space::Piece(Player::Second));
        }

        new
    }

    pub fn print(&self) {
        for y in (0..7).rev() {
            let mut buffer = format!("{} ", y + 1);

            for x in 0..7 {
                let position = Position { x, y };
                buffer += format!("{}", self.space(position)).as_str();
                buffer += if self.ball() == position { "B" } else { " " };
                buffer += if x < 6 { " " } else { "" };
            }

            println!("{buffer}");
        }

        println!("  A  B  C  D  E  F  G")
    }

    pub(super) fn ball_trapped(&self) -> bool {
        !self.path_to_rank(self.ball, 0) || !self.path_to_rank(self.ball, 6)
    }

    pub(super) fn has_winner(&self, player: Player) -> bool {
        if (player == Player::First && self.ball.y != 6)
            || (player == Player::Second && self.ball.y != 0)
        {
            return false;
        }

        self.space(self.ball) == Space::Piece(player)
    }

    pub(super) fn space(&self, position: Position) -> Space {
        if Self::invalid_position(position) {
            return Space::Invalid;
        }

        self.spaces[position.x as usize][position.y as usize]
    }

    pub(super) fn ball(&self) -> Position {
        self.ball
    }

    pub(super) fn set_space(&mut self, position: Position, space: Space) {
        if Self::invalid_position(position) {
            return;
        }

        self.spaces[position.x as usize][position.y as usize] = space;
    }

    pub(super) fn set_ball(&mut self, position: Position) {
        self.ball = position;
    }

    fn path_to_rank(&self, from: Position, y: i8) -> bool {
        let mut closed_spaces = HashSet::new();
        let mut open_spaces = BinaryHeap::new();

        open_spaces.push(ClosestPositionToRank { position: from, y });

        while let Some(next) = open_spaces.pop() {
            if next.position.y == y {
                return true;
            }

            closed_spaces.insert(next.position);

            for neighbour in next.position.neighbours() {
                if self.space(neighbour) != Space::Empty {
                    continue;
                }

                if closed_spaces.contains(&neighbour) {
                    continue;
                }

                open_spaces.push(ClosestPositionToRank {
                    position: neighbour,
                    y,
                })
            }
        }

        false
    }

    fn invalid_position(position: Position) -> bool {
        position.x < 0 || position.x >= 7 || position.y < 0 || position.y >= 7
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Space::Invalid => 'N',
            Space::Empty => '_',
            Space::Piece(Player::First) => 'O',
            Space::Piece(Player::Second) => 'X',
        };
        write!(f, "{}", char)
    }
}

impl PartialOrd<Self> for ClosestPositionToRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClosestPositionToRank {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y.cmp(&other.y).then(
            (other.position.y - self.y)
                .abs()
                .cmp(&(self.position.y - self.y).abs())
                .then(self.position.y.cmp(&other.position.y))
                .then(self.position.x.cmp(&other.position.x)),
        )
    }
}
