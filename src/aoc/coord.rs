#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Coord {
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Direction {
    Top,
    Right,
    Bottom,
    Left,
}

impl Direction {
    pub fn rotate_right(self) -> Self {
        match self {
            Direction::Top => Direction::Right,
            Direction::Right => Direction::Bottom,
            Direction::Bottom => Direction::Left,
            Direction::Left => Direction::Top,
        }
    }
    
    pub fn rotate_left(self) -> Self {
        match self {
            Direction::Top => Direction::Left,
            Direction::Right => Direction::Top,
            Direction::Bottom => Direction::Right,
            Direction::Left => Direction::Bottom,
        }
    }
    
    pub fn opposite(self) -> Self {
        match self {
            Direction::Top => Direction::Bottom,
            Direction::Right => Direction::Left,
            Direction::Bottom => Direction::Top,
            Direction::Left => Direction::Right,
        }
    }
}

impl core::ops::Add<Direction> for Coord {
    type Output = Coord;

    fn add(self, rhs: Direction) -> Self::Output {
        match rhs {
            Direction::Top => Coord { row: self.row - 1, col: self.col },
            Direction::Right => Coord { row: self.row, col: self.col + 1 },
            Direction::Bottom => Coord { row: self.row + 1, col: self.col },
            Direction::Left => Coord { row: self.row, col: self.col - 1 },
        }
    }
}
