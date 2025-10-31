use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::aoc::AocDay;

pub struct AocDay23 {
    links: BTreeMap<Computer, BTreeSet<Computer>>,
}

type Computer = [char; 2];

impl AocDay for AocDay23 {
    fn new(input: Vec<String>) -> Self {
        let mut this = Self {links: BTreeMap::new()};
        for line in input {
            let chars : Vec<_> = line.chars().collect();
            if chars.len() == 5 && chars[2] == '-' {
                let left = [chars[0], chars[1]];
                let right = [chars[3], chars[4]];
                this.links.entry(left).or_default().insert(right);
                this.links.entry(right).or_default().insert(left);
            }
        }
        this
    }

    fn part1(&self) -> String {
        let mut sets : BTreeSet<ThreeComputersSet> = BTreeSet::new();
        for (computer, links) in self.links.range(['t', 'a']..=['t', 'z']) {
            for (i, link1) in links.iter().enumerate() {
                for link2 in links.iter().skip(i+1) {
                    if self.links.get(link1).unwrap().contains(link2) {
                        sets.insert(ThreeComputersSet::new(*computer, *link1, *link2));
                    }
                }
            }
        }

        format!("{}", sets.len())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct ThreeComputersSet([Computer; 3]);

impl ThreeComputersSet {
    fn new(a: Computer, b: Computer, c: Computer) -> Self {
        let mut this = Self([a, b, c]);
        this.0.sort_unstable();
        this
    }
}