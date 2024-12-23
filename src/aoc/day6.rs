use crate::aoc::AocDay;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub enum Direction {
    Top,
    Right,
    Bottom,
    Left,
}

impl Direction {
    fn turn_right(self) -> Direction {
        match self {
            Direction::Top => Direction::Right,
            Direction::Right => Direction::Bottom,
            Direction::Bottom => Direction::Left,
            Direction::Left => Direction::Top,
        }
    }
}

trait Map {
    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn is_wall(&self, x: usize, y: usize) -> bool;
}

pub struct AocDay6 {
    map: Vec<Vec<bool>>,
    start: (usize, usize, Direction),
}

impl Map for AocDay6 {
    fn height(&self) -> usize {
        self.map.len()
    }
    fn width(&self) -> usize {
        self.map[0].len()
    }
    fn is_wall(&self, x: usize, y: usize) -> bool {
        self.map[y][x]
    }
}

impl AocDay6 {
    fn start_pos(&self) -> Position<Self> {
        Position {
            map: self,
            x: self.start.0,
            y: self.start.1,
            direction: self.start.2,
        }
    }

    fn path(&self) -> BTreeSet<(usize, usize)> {
        let mut visited = BTreeSet::new();
        let mut pos = self.start_pos();
        visited.insert((pos.x, pos.y));
        while let Some(next) = pos.next() {
            visited.insert((next.x, next.y));
            pos = next;
        }
        visited
    }
}

#[derive(Clone)]
struct Position<'a, M> {
    map: &'a M,
    x: usize,
    y: usize,
    direction: Direction,
}

impl<M: Map> Position<'_, M> {
    fn next(&self) -> Option<Self> {
        let (x, y) = match self.direction {
            Direction::Top => {
                if self.y == 0 {
                    None
                } else {
                    Some((self.x, self.y - 1))
                }
            }
            Direction::Right => {
                if self.x == self.map.width() - 1 {
                    None
                } else {
                    Some((self.x + 1, self.y))
                }
            }
            Direction::Bottom => {
                if self.y == self.map.height() - 1 {
                    None
                } else {
                    Some((self.x, self.y + 1))
                }
            }
            Direction::Left => {
                if self.x == 0 {
                    None
                } else {
                    Some((self.x - 1, self.y))
                }
            }
        }?;
        if self.map.is_wall(x, y) {
            Some(Position {
                map: self.map,
                x: self.x,
                y: self.y,
                direction: self.direction.turn_right(),
            })
        } else {
            Some(Position {
                map: self.map,
                x,
                y,
                direction: self.direction,
            })
        }
    }
}

struct OneMoreWall<'a, M> {
    map: &'a M,
    wall: (usize, usize),
}

impl<'a> OneMoreWall<'a, AocDay6> {
    fn new(map: &'a AocDay6, wall: (usize, usize)) -> Self {
        Self { map, wall }
    }

    fn start_at(&self, x: usize, y: usize, direction: Direction) -> Position<Self> {
        Position {
            map: self,
            x,
            y,
            direction,
        }
    }
}

impl<M: Map> Map for OneMoreWall<'_, M> {
    fn height(&self) -> usize {
        self.map.height()
    }

    fn width(&self) -> usize {
        self.map.width()
    }

    fn is_wall(&self, x: usize, y: usize) -> bool {
        if self.wall == (x, y) {
            true
        } else {
            self.map.is_wall(x, y)
        }
    }
}

impl AocDay for AocDay6 {
    fn new(input: Vec<String>) -> Self {
        let mut start = (0, 0, Direction::Top);
        let mut map = Vec::new();
        for (y, line) in input.iter().filter(|l| !l.is_empty()).enumerate() {
            map.push(Vec::new());
            for (x, c) in line.chars().enumerate() {
                let wall = match c {
                    '^' => {
                        start = (x, y, Direction::Top);
                        false
                    }
                    'v' => {
                        start = (x, y, Direction::Bottom);
                        false
                    }
                    '>' => {
                        start = (x, y, Direction::Right);
                        false
                    }
                    '<' => {
                        start = (x, y, Direction::Left);
                        false
                    }
                    '#' => true,
                    _ => false,
                };
                map[y].push(wall);
            }
        }

        AocDay6 { map, start }
    }

    fn part1(&self) -> String {
        let visited = self.path();
        visited.len().to_string()
    }

    fn part2(&self) -> String {
        let mut count = 0;
        let mut path = BTreeSet::new();
        let mut pos = self.start_pos();
        path.insert((pos.x, pos.y, pos.direction));
        while let Some(next) = pos.next() {
            if path
                .range((next.x, next.y, Direction::Top)..=(next.x, next.y, Direction::Left))
                .count()
                == 0
            {
                let onemorewall = OneMoreWall::new(self, (next.x, next.y));
                let mut visited = BTreeSet::new();
                let mut pos2 = onemorewall.start_at(pos.x, pos.y, pos.direction);
                while let Some(next2) = pos2.next() {
                    if path.contains(&(next2.x, next2.y, next2.direction))
                        || visited.contains(&(next2.x, next2.y, next2.direction))
                    {
                        count += 1;
                        break;
                    }
                    visited.insert((next2.x, next2.y, next2.direction));
                    pos2 = next2;
                }
            }
            path.insert((next.x, next.y, next.direction));
            pos = next;
        }
        count.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::string::ToString;
    const INPUT: &'static str = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";
    #[test]
    fn test() {
        let day = AocDay6::new(INPUT.lines().map(ToString::to_string).collect());
        assert_eq!(day.part1(), "41");
        assert_eq!(day.part2(), "6");
    }
}
