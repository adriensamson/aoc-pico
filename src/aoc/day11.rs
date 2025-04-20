use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use alloc::collections::BTreeMap;
use crate::aoc::AocDay;

pub struct AocDay11 {
    stones: Vec<u32>
}

impl AocDay11 {
    fn count_after_blinks(&self, n: usize) -> u64 {
        let mut counts : BTreeMap<u64, u64> = BTreeMap::new();
        for s in self.stones.iter().copied() {
            *counts.entry(s as u64).or_default() += 1
        }
        for _ in 0..n {
            let mut new_counts = BTreeMap::new();
            for (s, c) in counts.into_iter() {
                let str = format!("{s}");
                if s == 0 {
                    *new_counts.entry(1).or_default() += c;
                } else if str.len() % 2 == 0 {
                    let mid = str.len() / 2;
                    for s in [str[..mid].parse().unwrap(), str[mid..].parse().unwrap()] {
                        *new_counts.entry(s).or_default() += c;
                    }
                } else {
                    *new_counts.entry(s * 2024).or_default() += c;
                }
            }
            counts = new_counts;
        }

        counts.values().sum()
    }
}

impl AocDay for AocDay11 {
    fn new(input: Vec<String>) -> Self {
        let stones = input[0].split(' ').map(|s| s.parse::<u32>().unwrap()).collect();
        Self { stones }
    }

    fn part1(&self) -> String {
        format!("{}", self.count_after_blinks(25))
    }

    fn part2(&self) -> String {
        format!("{}", self.count_after_blinks(75))
    }
}
