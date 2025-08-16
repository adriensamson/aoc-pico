use alloc::{format, vec};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::aoc::AocDay;

pub struct AocDay19 {
    available_towels: Vec<String>,
    patterns: Vec<String>,
}

impl AocDay for AocDay19 {
    fn new(input: Vec<String>) -> Self {
        let mut iter = input.into_iter();
        let available_towels = iter.next().unwrap().split(", ").map(ToString::to_string).collect();
        iter.next();
        let patterns = iter.collect();
        Self {
            available_towels,
            patterns,
        }
    }

    fn part1(&self) -> String {
        let available_set : BTreeSet<&str> = self.available_towels.iter().map(String::as_str).collect();
        let max_len = available_set.iter().map(|s| s.len()).max().unwrap();
        let n = self.patterns.iter().filter(|pattern| {
            let pattern = pattern.as_str();
            let mut backtrack = vec![(0, max_len+1)];
            let mut impossible_ends : BTreeSet<usize> = BTreeSet::new();
            while let Some((start, prev_len)) = backtrack.pop() {
                let rest = &pattern[start..];
                if impossible_ends.contains(&rest.len()) {
                    continue;
                }
                for i in (1..prev_len.min(rest.len() + 1)).rev() {
                    if available_set.contains(&rest[0..i]) {
                        backtrack.push((start, i));
                        if rest[i..].is_empty() {
                            return true;
                        }
                        backtrack.push((start + i, max_len+1));
                        break;
                    }
                }
                impossible_ends.insert(rest.len());
            }
            false
        }).count();
        format!("{n}")
    }
}
