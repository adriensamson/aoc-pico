use alloc::{format, vec};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
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

    fn part2(&self) -> String {
        let count : u64 = self.patterns.iter().map(|s| num_ways(s, &self.available_towels)).sum();
        format!("{count}")
    }
}

struct StartsWith<'a>(&'a str);

impl core::ops::RangeBounds<str> for StartsWith<'_> {
    fn start_bound(&self) -> Bound<&str> {
        if self.0.is_empty() {
            Bound::Included(self.0)
        } else {
            Bound::Included(&self.0[..1])
        }
    }

    fn end_bound(&self) -> Bound<&str> {
        Bound::Included(self.0)
    }
}

fn is_doable(patt: &str, available_towels: &BTreeSet<String>) -> bool {
    let mut to_check = vec![patt];
    let mut failed_ends = BTreeSet::new();
    while let Some(pattern) = to_check.pop() {
        for towel in available_towels.range(StartsWith(pattern)) {
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

#[derive(Eq, PartialEq)]
struct BySize<'a>(&'a str);

impl Ord for BySize<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.len().cmp(&other.0.len()).then(self.0.cmp(other.0))
    }
}
impl PartialOrd for BySize<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn num_ways(patt: &str, available_towels: &BTreeSet<String>) -> u64 {
    let mut to_check = BTreeSet::new();
    to_check.insert(BySize(patt));
    let mut end_ways : BTreeMap<&str, u64> = BTreeMap::new();
    end_ways.insert("", 1);
    while let Some(BySize(pattern)) = to_check.pop_first() {
        let towels : Vec<_> = available_towels.range(StartsWith(pattern))
            .filter_map(|t| pattern.strip_prefix(t.as_str()))
            .collect();
        let missing : Vec<_> = towels.iter().copied().filter(|next| !end_ways.contains_key(next)).collect();
        if !missing.is_empty() {
            to_check.insert(BySize(pattern));
            to_check.extend(missing.into_iter().map(BySize));
        } else {
            let count : u64 = towels.iter().map(|next| end_ways.get(next).unwrap()).sum();
            end_ways.insert(pattern, count);
        }
    }
    end_ways.get(patt).copied().unwrap()
}