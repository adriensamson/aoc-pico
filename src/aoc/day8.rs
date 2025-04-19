use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay8 {
    width: usize,
    height: usize,
    antennas: BTreeMap<char, Vec<(usize, usize)>>,
}

impl AocDay for AocDay8 {
    fn new(input: Vec<String>) -> Self {
        let mut antennas : BTreeMap<char, Vec<(usize, usize)>> = BTreeMap::new();
        let mut width = 0;
        let mut height = 0;
        for (i, row) in input.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).enumerate() {
            if i == 0 {
                width = row.len();
            }
            height = i + 1;
            for (j, c) in row.chars().enumerate() {
                if c != '.' {
                    antennas.entry(c).or_default().push((i, j));
                }
            }
        }
        Self {width, height, antennas}
    }

    fn part1(&self) -> String {
        let mut antinodes = BTreeSet::new();
        for ants in self.antennas.values() {
            for i in 0..ants.len()-1 {
                for j in i+1..ants.len() {
                    let (a_y, a_x) = ants[i];
                    let (b_y, b_x) = ants[j];
                    let r_x = 2 * a_x as isize - b_x as isize;
                    let r_y = 2 * a_y as isize - b_y as isize;
                    if 0 <= r_x && r_x < self.width as isize && 0 <= r_y && r_y < self.height as isize {
                        antinodes.insert((r_y as usize, r_x as usize));
                    }
                    let r_x = 2 * b_x as isize - a_x as isize;
                    let r_y = 2 * b_y as isize - a_y as isize;
                    if 0 <= r_x && r_x < self.width as isize && 0 <= r_y && r_y < self.height as isize {
                        antinodes.insert((r_y as usize, r_x as usize));
                    }
                }
            }
        }

        format!("{}", antinodes.len())
    }

    fn part2(&self) -> String {
        let mut antinodes = BTreeSet::new();
        for ants in self.antennas.values() {
            for i in 0..ants.len()-1 {
                for j in i+1..ants.len() {
                    let (a_y, a_x) = ants[i];
                    let (b_y, b_x) = ants[j];
                    let (inc_x, inc_y) = normalize(a_x as isize - b_x as isize, a_y as isize - b_y as isize);
                    let mut k = 0;
                    loop {
                        let x = a_x as isize + k * inc_x;
                        let y = a_y as isize + k * inc_y;
                        if 0 <= x && x < self.width as isize && 0 <= y && y < self.height as isize {
                            antinodes.insert((y as usize, x as usize));
                            k += 1;
                        } else {
                            break
                        }
                    }
                    k = -1;
                    loop {
                        let x = a_x as isize + k * inc_x;
                        let y = a_y as isize + k * inc_y;
                        if 0 <= x && x < self.width as isize && 0 <= y && y < self.height as isize {
                            antinodes.insert((y as usize, x as usize));
                            k -= 1;
                        } else {
                            break
                        }
                    }
                }
            }
        }

        format!("{}", antinodes.len())
    }
}

fn normalize(x: isize, y: isize) -> (isize, isize) {
    for i in 2..x.abs().max(y.abs()) {
        if x % i == 0 && y % i == 0 {
            return normalize(x / i, y / i);
        }
    }
    (x, y)
}
