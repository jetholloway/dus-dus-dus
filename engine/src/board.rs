use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    spaces: [[Space; 7]; 7],
    ball: Position,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Space {
    Invalid,
    Empty,
    Piece(Player),
}

impl Board {
    pub(crate) fn new() -> Self {
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

    /// The stall this board is in, if any. The rulebook forbids both kinds at
    /// all times after setup.
    pub(crate) fn stall(&self) -> Option<&'static str> {
        if self.ball_stall() {
            return Some("Ball stall");
        }

        if self.wall_stall(Player::First) || self.wall_stall(Player::Second) {
            return Some("Wall stall");
        }

        None
    }

    pub(crate) fn has_winner(&self, player: Player) -> bool {
        if (player == Player::First && self.ball.y != 6)
            || (player == Player::Second && self.ball.y != 0)
        {
            return false;
        }

        self.space(self.ball) == Space::Piece(player)
    }

    pub fn space(&self, position: Position) -> Space {
        if Self::invalid_position(position) {
            return Space::Invalid;
        }

        self.spaces[position.x as usize][position.y as usize]
    }

    pub fn ball(&self) -> Position {
        self.ball
    }

    pub(crate) fn set_space(&mut self, position: Position, space: Space) {
        if Self::invalid_position(position) {
            return;
        }

        self.spaces[position.x as usize][position.y as usize] = space;
    }

    pub(crate) fn set_ball(&mut self, position: Position) {
        self.ball = position;
    }

    /// No wall stall: the defender must leave a gap through which the attacker
    /// can still bring new pieces into the defender's end zone.
    ///
    /// Only the defender's pieces form a wall; the attacker's own pieces never
    /// block the attacker, since they can move out of the way. An attacker
    /// piece already in the end zone is not a new piece, so it doesn't count
    /// as a way in.
    fn wall_stall(&self, defender: Player) -> bool {
        let end_zone = match defender {
            Player::First => 0,
            Player::Second => 6,
        };
        let attacker = defender.other();

        let open_end_zone = (0..7)
            .map(|x| Position { x, y: end_zone })
            .filter(|position| self.space(*position) != Space::Piece(defender));

        !self.reaches(
            open_end_zone,
            |space| space != Space::Piece(defender),
            |position, space| space == Space::Piece(attacker) && position.y != end_zone,
        )
    }

    /// No ball stall: the player without the ball must be able to get a piece
    /// onto a square sharing an edge with the ball, so a tackle stays possible.
    fn ball_stall(&self) -> bool {
        let Space::Piece(holder) = self.space(self.ball) else {
            return false;
        };
        let challenger = holder.other();

        let edges = self.ball.orthogonal_neighbours();

        if edges
            .clone()
            .any(|position| self.space(position) == Space::Piece(challenger))
        {
            return false;
        }

        let empty_edges = edges.filter(|position| self.space(*position) == Space::Empty);

        !self.reaches(
            empty_edges,
            |space| space == Space::Empty,
            |_, space| space == Space::Piece(challenger),
        )
    }

    /// Flood fills orthogonally from `starts` through squares that are
    /// `passable`, and reports whether it reaches a square that is a `goal`.
    /// Pieces only move orthogonally, so a gap that is only diagonal is not a
    /// way through.
    fn reaches(
        &self,
        starts: impl Iterator<Item = Position>,
        passable: impl Fn(Space) -> bool,
        goal: impl Fn(Position, Space) -> bool,
    ) -> bool {
        let mut visited = [[false; 7]; 7];
        let mut stack = [Position { x: 0, y: 0 }; 49];
        let mut len = 0;

        for start in starts {
            visited[start.x as usize][start.y as usize] = true;
            stack[len] = start;
            len += 1;
        }

        while len > 0 {
            len -= 1;
            let position = stack[len];

            for neighbour in position.orthogonal_neighbours() {
                let space = self.space(neighbour);

                if space == Space::Invalid || visited[neighbour.x as usize][neighbour.y as usize] {
                    continue;
                }

                if goal(neighbour, space) {
                    return true;
                }

                if passable(space) {
                    visited[neighbour.x as usize][neighbour.y as usize] = true;
                    stack[len] = neighbour;
                    len += 1;
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a board from rows written top to bottom, rank 7 first. `O` and
    /// `X` are pieces, `.` is empty, and a lowercase `o` or `x` is the piece
    /// holding the ball.
    fn board(rows: [&str; 7]) -> Board {
        let mut board = Board::new();

        for (row, line) in rows.iter().enumerate() {
            let y = 6 - row as i8;

            for (x, char) in line.chars().enumerate() {
                let position = Position { x: x as i8, y };
                let space = match char.to_ascii_uppercase() {
                    'O' => Space::Piece(Player::First),
                    'X' => Space::Piece(Player::Second),
                    '.' => Space::Empty,
                    other => panic!("unexpected square {other:?}"),
                };

                board.set_space(position, space);

                if char.is_ascii_lowercase() {
                    board.set_ball(position);
                }
            }
        }

        board
    }

    #[test]
    fn position_after_setup_has_no_stall() {
        let board = board([
            "XXX.XXX", //
            ".......", //
            "...X...", //
            ".......", //
            "...o...", //
            ".......", //
            "OOO.OOO", //
        ]);

        assert_eq!(board.stall(), None);
    }

    #[test]
    fn a_full_end_zone_is_a_wall_stall() {
        let board = board([
            "XXX.XXX", //
            ".......", //
            "...X...", //
            ".......", //
            "...o...", //
            ".......", //
            "OOOOOOO", //
        ]);

        assert!(board.wall_stall(Player::First));
        assert_eq!(board.stall(), Some("Wall stall"));
    }

    #[test]
    fn a_diagonal_wall_is_a_wall_stall() {
        // The rulebook's "No Wall Stall" counter-example: pieces touching only
        // at their corners still block, because nothing moves diagonally.
        let board = board([
            "xXX.XXX", //
            ".......", //
            ".......", //
            "O.O.O.O", //
            ".O.O.O.", //
            ".......", //
            ".......", //
        ]);

        assert!(board.wall_stall(Player::First));
        assert!(!board.wall_stall(Player::Second));
    }

    #[test]
    fn a_wall_with_a_gap_is_not_a_wall_stall() {
        let board = board([
            "xXX.XXX", //
            ".......", //
            ".......", //
            "O.O.O.O", //
            ".O.O...", //
            ".......", //
            ".......", //
        ]);

        assert!(!board.wall_stall(Player::First));
    }

    #[test]
    fn an_attacker_already_in_the_end_zone_is_not_a_way_in() {
        // From a real game: Teal's wall A7-B6-C7-D6-E5-F6-G7 is complete.
        // Orange's piece on E7 touches the empty D7 and F7, but it is already
        // in the end zone, so no new Orange piece can get there.
        let board = board([
            "X.X.O.X", //
            ".X.X.X.", //
            "....X.O", //
            ".......", //
            ".O.....", //
            ".......", //
            "o..OOO.", //
        ]);

        assert!(board.wall_stall(Player::Second));
        assert_eq!(board.stall(), Some("Wall stall"));
    }

    #[test]
    fn an_attacker_filling_the_only_gap_is_not_a_wall_stall() {
        // From a real game: Orange has just moved onto B7, the only square in
        // Teal's end zone it could reach; E7 and G7 are walled off by Teal.
        // Orange's own piece in the gap doesn't close it.
        let board = board([
            "XOXX.X.", //
            "....X.X", //
            "....o..", //
            "..X....", //
            ".......", //
            ".......", //
            "OO..OOO", //
        ]);

        assert!(!board.wall_stall(Player::Second));
        assert_eq!(board.stall(), None);
    }

    #[test]
    fn surrounding_the_ball_on_every_edge_is_a_ball_stall() {
        // Empty diagonals do not help: a tackle needs a shared edge.
        let board = board([
            "XXX.XXX", //
            ".......", //
            "...O...", //
            "..OoO..", //
            "...O...", //
            ".......", //
            "OOO.OOO", //
        ]);

        assert!(board.ball_stall());
        assert_eq!(board.stall(), Some("Ball stall"));
    }

    #[test]
    fn an_opponent_already_on_an_edge_is_not_a_ball_stall() {
        let board = board([
            "XXX.XXX", //
            ".......", //
            "...O...", //
            "..OoX..", //
            "...O...", //
            ".......", //
            "OOO.OOO", //
        ]);

        assert!(!board.ball_stall());
    }

    #[test]
    fn an_open_edge_the_opponent_can_reach_is_not_a_ball_stall() {
        let board = board([
            "XXX.XXX", //
            ".......", //
            ".......", //
            "..OoO..", //
            "...O...", //
            ".......", //
            "OOO.OOO", //
        ]);

        assert!(!board.ball_stall());
    }
}
