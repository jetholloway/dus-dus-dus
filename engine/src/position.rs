use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};
use std::str::FromStr;

pub type Path = Vec<Position>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Displacement {
    pub x: i8,
    pub y: i8,
}

pub struct NeighbourIterator {
    origin: Position,
    index: usize,
}

impl Position {
    const FILES: [char; 7] = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];

    pub fn neighbours(self) -> NeighbourIterator {
        NeighbourIterator::new(self)
    }

    pub fn orthogonal_neighbours(self) -> impl Iterator<Item = Position> + Clone {
        const DISPLACEMENTS: [Displacement; 4] = [
            Displacement { x: 1, y: 0 },
            Displacement { x: 0, y: 1 },
            Displacement { x: -1, y: 0 },
            Displacement { x: 0, y: -1 },
        ];

        DISPLACEMENTS
            .into_iter()
            .map(move |displacement| self + displacement)
    }

    pub fn taxicab_distance(self, other: Self) -> i8 {
        (self - other).taxicab_distance()
    }

    pub fn chebyshev_distance(self, other: Self) -> i8 {
        (self - other).chebyschev_distance()
    }

    pub fn try_get_orthogonal_path(self, to: Self) -> Option<Path> {
        if self == to {
            return None;
        }

        let step = (to - self).try_normalised_orthogonal()?;

        let mut path = Path::new();
        let mut cursor = self + step;
        while cursor != to {
            path.push(cursor);
            cursor += step;
        }

        Some(path)
    }

    pub fn try_get_diagonal_path(self, to: Self) -> Option<Path> {
        if self == to {
            return None;
        }

        let step = (to - self).try_normalised_diagonal()?;

        let mut path = Path::new();
        let mut cursor = self + step;
        while cursor != to {
            path.push(cursor);
            cursor += step;
        }

        Some(path)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.x < 0 || self.x >= 7 || self.y < 0 || self.y >= 7 {
            write!(f, "NO")
        } else {
            write!(f, "{}{}", Self::FILES[self.x as usize], self.y + 1)
        }
    }
}

impl FromStr for Position {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("Position missing file".to_string());
        }

        let file = trimmed.chars().next().unwrap().to_ascii_uppercase();
        let x = Self::FILES
            .iter()
            .position(|c| *c == file)
            .map(|x| x as i8)
            .ok_or(format!("Invalid position file: {file} out of range A..G"))?;

        let rank = &trimmed[1..];
        if rank.is_empty() {
            return Err("Position missing rank".to_string());
        }
        let y = i8::from_str(rank).map_err(|err| format!("Invalid position rank: {err}"))?;

        Ok(Self { x, y: y - 1 })
    }
}

impl AddAssign<Displacement> for Position {
    fn add_assign(&mut self, rhs: Displacement) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<Displacement> for Position {
    type Output = Self;

    fn add(mut self, rhs: Displacement) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign<Displacement> for Position {
    fn sub_assign(&mut self, rhs: Displacement) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub<Displacement> for Position {
    type Output = Self;

    fn sub(mut self, rhs: Displacement) -> Self::Output {
        self -= rhs;
        self
    }
}

impl Sub for Position {
    type Output = Displacement;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Displacement {
    pub fn try_normalised_orthogonal(self) -> Option<Self> {
        if (self.x != 0 && self.y != 0) || self.x == self.y {
            return None;
        }

        let x = if self.x > 0 {
            1
        } else if self.x < 0 {
            -1
        } else {
            0
        };
        let y = if self.y > 0 {
            1
        } else if self.y < 0 {
            -1
        } else {
            0
        };

        Some(Self { x, y })
    }

    pub fn try_normalised_diagonal(self) -> Option<Self> {
        if self.x.abs() != self.y.abs() || self.x == 0 {
            return None;
        }

        let x = if self.x > 0 { 1 } else { -1 };
        let y = if self.y > 0 { 1 } else { -1 };

        Some(Self { x, y })
    }

    pub fn taxicab_distance(self) -> i8 {
        self.x.abs() + self.y.abs()
    }

    pub fn chebyschev_distance(self) -> i8 {
        i8::max(self.x.abs(), self.y.abs())
    }
}

impl Neg for Displacement {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl NeighbourIterator {
    const DISPLACEMENTS: [Displacement; 8] = [
        Displacement { x: 1, y: 0 },
        Displacement { x: 1, y: 1 },
        Displacement { x: 0, y: 1 },
        Displacement { x: -1, y: 1 },
        Displacement { x: -1, y: 0 },
        Displacement { x: -1, y: -1 },
        Displacement { x: 0, y: -1 },
        Displacement { x: 1, y: -1 },
    ];

    fn new(origin: Position) -> Self {
        Self { origin, index: 0 }
    }
}

impl Iterator for NeighbourIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= Self::DISPLACEMENTS.len() {
            return None;
        }

        let next = self.origin + Self::DISPLACEMENTS[self.index];
        self.index += 1;

        Some(next)
    }
}
