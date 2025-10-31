use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::{format, vec};
use alloc::vec::Vec;
use crate::aoc::AocDay;

pub struct AocDay20 {
    start: Coord,
    end: Coord,
    walls: BTreeSet<Coord>,
}

type Coord = [u8; 2];

impl AocDay for AocDay20 {
    fn new(input: Vec<String>) -> Self {
        let mut start = None;
        let mut end = None;
        let mut walls = BTreeSet::new();
        for (y, line) in input.into_iter().enumerate() {
            for (x, char) in line.chars().enumerate() {
                match char {
                    'S' => start = Some([x as u8, y as u8]),
                    'E' => end = Some([x as u8, y as u8]),
                    '#' => { walls.insert([x as u8, y as u8]); },
                    _ => {}
                }
            }
        }
        Self {
            start: start.unwrap(),
            end: end.unwrap(),
            walls,
        }
    }

    fn part1(&self) -> String {
        let path = find_path(self.start, self.end, &self.walls);

        let mut cheats = 0usize;
        for (i, &point) in path.iter().enumerate() {
            for cheat_start in around(point).into_iter().filter(|cs| self.walls.contains(cs)) {
                for cheat_end in around(cheat_start).into_iter().filter(|cs| !self.walls.contains(cs)) {
                    let j = path.iter().enumerate().find_map(|(j, p)| (cheat_end == *p).then_some(j)).unwrap_or_default();
                    let gain = j.saturating_sub(i + 2);
                    if gain >= 100 {
                        cheats += 1;
                    }
                }
            }
        }

        format!("{cheats}")
    }

    fn part2(&self) -> String {
        let path = find_path(self.start, self.end, &self.walls);

        let mut cheats = 0usize;
        for (i, &point) in path.iter().enumerate() {
            for cheat_start in around(point).into_iter().filter(|cs| self.walls.contains(cs)) {
                let mut current = BTreeSet::new();
                current.extend(around(cheat_start).into_iter().filter(|cs| self.walls.contains(cs)));
                let mut visited = BTreeSet::new();
                visited.extend(current.iter().copied());
                for duration in 1..20 {
                    let mut next = BTreeSet::new();
                    for cheat in current {
                        for c in around(cheat).into_iter().filter(|c| !visited.contains(c)) {
                            if self.walls.contains(&c) {
                                next.insert(c);
                            } else {
                                let j = path.iter().enumerate().find_map(|(j, p)| (c == *p).then_some(j)).unwrap_or_default();
                                let gain = j.saturating_sub(i + duration + 1);
                                if gain >= 100 {
                                    cheats += 1;
                                }
                            }
                        }
                    }
                    current = next;
                }
            }
        }

        format!("{cheats}")
    }
}

fn around(coord: Coord) -> Vec<Coord> {
    let mut around = Vec::with_capacity(4);
    if coord[0] > 0 {
        around.push([coord[0] - 1, coord[1]]);
    }
    around.push([coord[0] + 1, coord[1]]);
    if coord[1] > 0 {
        around.push([coord[0], coord[1] - 1]);
    }
    around.push([coord[0], coord[1] + 1]);
    around
}

fn find_path(start: Coord, end: Coord, walls: &BTreeSet<Coord>) -> Vec<Coord> {
    let mut path = vec![start];
    let mut previous = start;
    let mut current = start;
    while current != end {
        let next = around(current).into_iter().find(|c| !walls.contains(c) && *c != previous).unwrap();
        path.push(next);
        previous = current;
        current = next;
    }
    path
}
