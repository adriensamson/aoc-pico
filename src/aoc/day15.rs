use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay15 {
    map: Map,
    directions: Vec<Direction>,
}

#[derive(Clone, Default)]
struct Map {
    walls: BTreeSet<(u8, u8)>,
    boxes: BTreeSet<(u8, u8)>,
    robot: (u8, u8),
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Right,
    Bottom,
    Left,
}

impl AocDay for AocDay15 {
    fn new(input: Vec<String>) -> Self {
        let mut map = Map::default();
        let mut directions = Vec::new();

        for (r, row) in input.iter().filter(|s| !s.is_empty()).enumerate() {
            if row.starts_with('#') {
                for (c, char) in row.chars().enumerate() {
                    match char {
                        '#' => { map.walls.insert((r as u8, c as u8)); },
                        'O' => { map.boxes.insert((r as u8, c as u8)); },
                        '@' => map.robot = (r as u8, c as u8),
                        _ => (),
                    };
                }
            } else {
                for char in row.chars() {
                    match char {
                        '^' => directions.push(Direction::Up),
                        '>' => directions.push(Direction::Right),
                        'v' => directions.push(Direction::Bottom),
                        '<' => directions.push(Direction::Left),
                        _ => ()
                    }
                }
            }
        }

        Self {map, directions}
    }

    fn part1(&self) -> String {
        let mut map = self.map.clone();
        for dir in self.directions.iter().copied() {
            map.move_robot(dir);
        }
        format!("{}", map.sum_coords())
    }
}

impl Direction {
    fn apply(self, (r, c): (u8, u8)) -> (u8, u8) {
        match self {
            Direction::Up => (r - 1, c),
            Direction::Right => (r, c + 1),
            Direction::Bottom => (r + 1, c),
            Direction::Left => (r, c - 1),
        }
    }
}

impl Map {
    fn move_robot(&mut self, dir: Direction) {
        let first = dir.apply(self.robot);
        if self.walls.contains(&first) {
            // blocked
            return;
        }
        if self.boxes.contains(&first) {
            let mut next = dir.apply(first);
            while self.boxes.contains(&next) {
                next = dir.apply(next);
            }
            if self.walls.contains(&next) {
                // blocked
                return;
            }
            // push
            self.boxes.insert(next);
            self.boxes.remove(&first);
        }
        // move
        self.robot = first;
    }

    fn sum_coords(&self) -> u64 {
        self.boxes.iter().map(|(r, c)| 100 * *r as u64 + *c as u64).sum()
    }
}
