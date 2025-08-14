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
            let mut backtrack = vec![(pattern.as_str(), max_len+1)];
            while let Some((rest, prev_len)) = backtrack.pop() {
                for i in (1..prev_len.min(rest.len() + 1)).rev() {
                    if available_set.contains(&rest[0..i]) {
                        backtrack.push((rest, i));
                        let rest = &rest[i..];
                        if rest.is_empty() {
                            return true;
                        }
                        backtrack.push((rest, max_len+1));
                        break;
                    }
                }
            }
            false
        }).count();
        format!("{n}")
    }
}
