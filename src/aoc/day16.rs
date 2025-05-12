use alloc::collections::btree_map::Entry;
use alloc::collections::{BTreeSet, BTreeMap};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::aoc::AocDay;
use crate::aoc::coord::{Coord, Direction};

pub struct AocDay16 {
    walls: BTreeSet<Coord>,
    start: Coord,
    end: Coord,
}

impl AocDay for AocDay16 {
    fn new(input: Vec<String>) -> Self {
        let mut walls = BTreeSet::new();
        let mut start = Coord::default();
        let mut end = Coord::default();
        for (r, row) in input.iter().filter(|s| !s.is_empty()).enumerate() {
            for (c, char) in row.chars().enumerate() {
                match char {
                    'S' => { start = Coord {row: r as u8, col: c as u8} },
                    'E' => { end = Coord {row: r as u8, col: c as u8} },
                    '#' => { walls.insert(Coord {row: r as u8, col: c as u8}); },
                    _ => {}
                }
            }
        }
        Self {walls, start, end}
    }

    fn part1(&self) -> String {
        let start = State {
            position: self.start,
            direction: Direction::Right,
        };
        let mut states = BTreeMap::new();
        let mut done = BTreeSet::new();
        states.insert(start, 0);
        let score = loop {
            let (state, score) = states.iter().min_by_key(|(_, score)| *score).map(|(state, score)| (*state, *score)).unwrap();
            if state.position == self.end {
                break score;
            }
            done.insert(state);
            states.remove(&state);
            for (next, incr) in state.next(&self.walls) {
                if done.contains(&next) {
                    continue;
                }
                let score2 = score + incr;
                match states.entry(next) {
                    Entry::Occupied(mut entry) => {
                        if score2 < *entry.get() {
                            entry.insert(score2);
                        }
                    },
                    Entry::Vacant(entry) => {
                        entry.insert(score2);
                    }
                }
            }
        };
        format!("{score}")
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct State {
    position: Coord,
    direction: Direction,
}

impl State {
    fn next(&self, walls: &BTreeSet<Coord>) -> Vec<(State, usize)> {
        let mut vec = Vec::with_capacity(4);

        let mut next = self.position + self.direction;
        let mut score = 1;
        loop {
            if walls.contains(&next) {
                break;
            }
            if walls.contains(&(next + self.direction.rotate_left())) && walls.contains(&(next + self.direction.rotate_right())) {
                next = next + self.direction;
                score += 1;
            } else {
                vec.push((Self {position: next, direction: self.direction}, score));
                break;
            }
        }

        for (dir, score) in [
            (self.direction.rotate_right(), 1000),
            (self.direction.rotate_left(), 1000),
            (self.direction.opposite(), 2000),
        ] {
            if !walls.contains(&(self.position + dir)) {
                vec.push((Self {position: self.position, direction: dir}, score));
            }
        }
        vec
    }
}
