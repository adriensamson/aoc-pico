use alloc::collections::btree_map::Entry;
use alloc::collections::{BTreeSet, BTreeMap};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format};
use core::ops::{RangeBounds};
use defmt::debug;
use crate::aoc::AocDay;
use crate::aoc::coord::{Coord, Direction};

pub struct AocDay16 {
    graph: BTreeMap<(Coord, Direction), (Coord, u8)>,
    start: Coord,
    end: Coord,
}

impl AocDay for AocDay16 {
    fn new(input: Vec<String>) -> Self {
        crate::memory::debug_heap_size();
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
        debug!("graph size: {}", graph.len());
        Self {graph, start, end}
    }

    fn part1(&self) -> String {
        crate::memory::debug_heap_size();
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

    fn part2(&self) -> String {
        crate::memory::debug_heap_size();
        let start = State {
            position: self.start,
            direction: Direction::Right,
        };
        let mut states : BTreeMap<State, usize> = BTreeMap::new();
        let mut done : BTreeMap<State, usize> = BTreeMap::new();
        states.insert(start, 0);
        let mut min_score = None;
        loop {
            let state = states.iter().min_by_key(|(_, score)| *score).map(|(state, _)| *state).unwrap();
            let score = states.remove(&state).unwrap();
            if let Some(min) = min_score {
                if min < score {
                    break;
                }
            }
            done.insert(state, score);
            if state.position == self.end {
                min_score = Some(score);
                continue;
            }
            for (next, incr) in state.next(&self.graph) {
                if done.contains_key(&next) {
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
        }
        debug!("found!");
        let mut steps : Vec<_> = [Direction::Right, Direction::Top].iter()
            .map(|dir| State { position: self.end, direction: *dir })
            .filter_map(|state| done.get(&state).map(|score| (state, *score)))
            .collect();
        let mut seats = BTreeSet::new();
        while let Some((to, score)) = steps.pop() {
            if let Some((from_coord, _)) = self.graph.get(&(to.position, to.direction.opposite())) {
                let froms = done.range(State::range_coord(*from_coord))
                    .filter(|(from, from_score)| from.next(&self.graph).contains(&(to, score.saturating_sub(**from_score))));
                for (&from, &s) in froms {
                    let mut pos = from.position;
                    seats.insert(pos);
                    while pos != to.position {
                        pos = pos + to.direction;
                        seats.insert(pos);
                    }
                    steps.push((from, s));
                }
            }
        }
        format!("{}", seats.len())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct State {
    position: Coord,
    direction: Direction,
}

impl State {
    fn next(&self, graph: &BTreeMap<(Coord, Direction), (Coord, u8)>) -> Vec<(State, usize)> {
        [
            (self.direction, 0),
            (self.direction.rotate_right(), 1000),
            (self.direction.rotate_left(), 1000),
            (self.direction.opposite(), 2000),
        ].into_iter()
            .filter_map(|(dir, score)| graph.get(&(self.position, dir)).copied()
                .map(|(to, dist)| (Self {position: to, direction: dir}, score + dist as usize))
            ).collect()
    }

    fn range_coord(c: Coord) -> impl RangeBounds<Self> {
        Self {position: c, direction: Direction::Top}..=Self {position: c, direction: Direction::Left}
    }
}
