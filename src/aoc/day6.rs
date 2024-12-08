use crate::aoc::AocDay;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

pub struct AocDay6 {
    map: Vec<Vec<bool>>,
    start: (usize, usize, Direction),
}

impl AocDay6 {
    fn width(&self) -> usize {
        self.map[0].len()
    }
    fn height(&self) -> usize {
        self.map.len()
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
        let mut visited = BTreeSet::new();
        let mut pos = self.start;
        visited.insert((pos.0, pos.1));
        loop {
            let next = match pos.2 {
                Direction::Top => {
                    if pos.1 == 0 {
                        break;
                    }
                    (pos.0, pos.1 - 1)
                }
                Direction::Right => {
                    if pos.0 == self.width() - 1 {
                        break;
                    }
                    (pos.0 + 1, pos.1)
                }
                Direction::Bottom => {
                    if pos.1 == self.height() - 1 {
                        break;
                    }
                    (pos.0, pos.1 + 1)
                }
                Direction::Left => {
                    if pos.0 == 0 {
                        break;
                    }
                    (pos.0 - 1, pos.1)
                }
            };
            if self.map[next.1][next.0] {
                pos.2 = pos.2.turn_right();
                continue;
            }
            visited.insert(next);
            pos = (next.0, next.1, pos.2)
        }
        visited.len().to_string()
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
        assert_eq!(day.part1(), "41")
    }
}
