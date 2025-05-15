use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::{format, vec};
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

    fn part2(&self) -> String {
        let mut map = WideMap::from(&self.map);
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

struct WideMap {
    walls: BTreeSet<(u8, u8)>,
    boxes: BTreeSet<(u8, u8)>,
    robot: (u8, u8),
}

impl From<&Map> for WideMap {
    fn from(value: &Map) -> Self {
        let walls = value.walls.iter().copied().flat_map(|(r, c)| [(r, c * 2), (r, c * 2 + 1)]).collect();
        let boxes = value.boxes.iter().copied().map(|(r, c)| (r, c * 2)).collect();
        let robot = (value.robot.0, value.robot.1 * 2);
        Self {walls, boxes, robot}
    }
}

impl WideMap {
    fn sum_coords(&self) -> u64 {
        self.boxes.iter().map(|(r, c)| 100 * *r as u64 + *c as u64).sum()
    }

    fn move_robot(&mut self, dir: Direction) {
        let mut current_positions = vec![self.robot];
        let mut moving_boxes = vec![];
        loop {
            if current_positions.iter().any(|p| self.walls.contains(&dir.apply(*p))) {
                // blocked by wall
                return;
            }
            let next_boxes : BTreeSet<_> = current_positions.iter().copied()
                .flat_map(|(r, c)| [dir.apply((r, c)), dir.apply((r, c.saturating_sub(1)))])
                .filter(|b| self.boxes.contains(b) && !moving_boxes.contains(b))
                .collect();
            moving_boxes.extend(&next_boxes);
            if next_boxes.is_empty() {
                break;
            }
            current_positions = next_boxes.iter().copied().flat_map(|(r, c)| [(r, c), (r, c + 1)]).collect();
        }
        // move boxes
        for b in moving_boxes.iter() {
            self.boxes.remove(b);
        }
        for b in moving_boxes {
            self.boxes.insert(dir.apply(b));
        }
        self.robot = dir.apply(self.robot);
    }
}
