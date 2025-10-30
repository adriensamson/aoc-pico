use alloc::{format, vec};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::Bound;
use crate::aoc::AocDay;

pub struct AocDay19 {
    available_towels: BTreeSet<String>,
    patterns: Vec<String>,
}

impl AocDay for AocDay19 {
    fn new(input: Vec<String>) -> Self {
        let mut iter = input.into_iter();
        let available_towels = iter.next().unwrap()
            .split(", ")
            .map(ToString::to_string)
            .collect();
        iter.next();
        let patterns = iter.filter(|s| !s.is_empty()).collect();
        Self {
            available_towels,
            patterns,
        }
    }

    fn part1(&self) -> String {
        let count = self.patterns.iter().filter(|s| is_doable(s, &self.available_towels)).count();

        format!("{count}")
    }
}

fn is_doable(patt: &str, available_towels: &BTreeSet<String>) -> bool {
    let mut to_check = vec![patt];
    let mut failed_ends = BTreeSet::new();
    while let Some(pattern) = to_check.pop() {
        for towel in available_towels.range::<str, _>((Bound::Included(&pattern[..1]), Bound::Included(pattern))) {
            let mut found = false;
            if let Some(next) = pattern.strip_prefix(towel) {
                found = true;
                if next.is_empty() {
                    return true;
                }
                if !failed_ends.contains(&next) {
                    to_check.push(next);
                }
            }
            if !found {
                failed_ends.insert(pattern);
            }
        }
    }
    false
}