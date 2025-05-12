use alloc::collections::btree_map::Entry;
use alloc::collections::{BTreeSet, BTreeMap};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::aoc::AocDay;
use crate::aoc::coord::{Coord, Direction};

pub struct AocDay16 {
    graph: BTreeMap<(Coord, Direction), (Coord, usize)>,
    start: Coord,
    end: Coord,
}

impl AocDay for AocDay16 {
    fn new(input: Vec<String>) -> Self {
        let mut walls = BTreeSet::new();
        let mut start = Coord::default();
        let mut end = Coord::default();
        let mut width = 0;
        let mut height = 0;
        for (r, row) in input.iter().filter(|s| !s.is_empty()).enumerate() {
            height += 1;
            width = width.max(row.len() as u8);
            for (c, char) in row.chars().enumerate() {
                match char {
                    'S' => { start = Coord {row: r as u8, col: c as u8} },
                    'E' => { end = Coord {row: r as u8, col: c as u8} },
                    '#' => { walls.insert(Coord {row: r as u8, col: c as u8}); },
                    _ => {}
                }
            }
        }
        let mut graph = BTreeMap::new();
        for r in 0u8..height {
            for c in 0u8..width {
                let from = Coord {row: r, col: c};
                if walls.contains(&from) {
                    continue;
                }
                if !walls.contains(&(from + Direction::Top)) || !walls.contains(&(from + Direction::Bottom)) {
                    let mut next = from + Direction::Right;
                    let mut n = 1;
                    loop {
                        if walls.contains(&next) {
                            break;
                        }
                        if walls.contains(&(next + Direction::Top)) && walls.contains(&(next + Direction::Bottom)) && next != end {
                            next = next + Direction::Right;
                            n += 1;
                        } else {
                            graph.insert((from, Direction::Right), (next, n));
                            graph.insert((next, Direction::Left), (from, n));
                            break;
                        }
                    }
                }
                if !walls.contains(&(from + Direction::Left)) || !walls.contains(&(from + Direction::Right)) {
                    let mut next = from + Direction::Bottom;
                    let mut n = 1;
                    loop {
                        if walls.contains(&next) {
                            break;
                        }
                        if walls.contains(&(next + Direction::Left)) && walls.contains(&(next + Direction::Right)) && next != start {
                            next = next + Direction::Bottom;
                            n += 1;
                        } else {
                            graph.insert((from, Direction::Bottom), (next, n));
                            graph.insert((next, Direction::Top), (from, n));
                            break;
                        }
                    }
                }
            }
        }
        Self {graph, start, end}
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
            for (next, incr) in state.next(&self.graph) {
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
    fn next(&self, graph: &BTreeMap<(Coord, Direction), (Coord, usize)>) -> Vec<(State, usize)> {
        [
            (self.direction, 0),
            (self.direction.rotate_right(), 1000),
            (self.direction.rotate_left(), 1000),
            (self.direction.opposite(), 2000),
        ].into_iter()
            .filter_map(|(dir, score)| graph.get(&(self.position, dir)).copied()
                .map(|(to, dist)| (Self {position: to, direction: dir}, score + dist))
            ).collect()
    }
}
