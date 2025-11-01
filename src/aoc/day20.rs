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

        let cheats = count_cheats(&path, 2);

        format!("{cheats}")
    }

    fn part2(&self) -> String {
        let path = find_path(self.start, self.end, &self.walls);

        let cheats = count_cheats(&path, 20);

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

fn count_cheats(path: &[Coord], allowed_cheats: u8) -> usize {
    let mut count = 0;
    for i in 0..path.len() - 100 {
        let before = path[i];
        count += (i+100..path.len())
            .map(|j| (j, path[j]))
            .filter(|(j, end)| {
                let cheat_len = before[0].abs_diff(end[0]).saturating_add(before[1].abs_diff(end[1]));
                cheat_len <= allowed_cheats && j.saturating_sub(i + cheat_len as usize) >= 100
            })
            .count();
    }
    count
}
