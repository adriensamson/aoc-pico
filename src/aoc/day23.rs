use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;
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

    fn part2(&self) -> String {
        let mut max = Vec::new();
        let mut by_size : Vec<Computer> = self.links.keys().copied().collect();
        by_size.sort_unstable_by_key(|c| Reverse(self.links.get(c).unwrap().len()));
        for base in by_size {
            if max.contains(&base) {
                continue;
            }
            let mut base_set : Vec<Computer> = self.links.get(&base).unwrap().iter().copied().collect();
            base_set.push(base);
            if base_set.len() <= max.len() {
                break;
            }

            let mut refined : BTreeMap<Computer, BTreeSet<Computer>> = base_set.iter().copied().map(|c| {
                let links = self.links.get(&c).unwrap().iter().copied().filter(|l| base_set.contains(l)).collect();
                (c, links)
            }).collect();
            base_set.sort_unstable_by_key(|c| Reverse(refined.get(c).unwrap().len()));
            while refined.get(base_set.last().unwrap()).unwrap().len() < base_set.len() - 1 {
                let rm = base_set.pop().unwrap();
                refined.remove(&rm);
                for links in refined.values_mut() {
                    links.remove(&rm);
                }
                base_set.sort_unstable_by_key(|c| Reverse(refined.get(c).unwrap().len()));
            }
            if base_set.len() > max.len() {
                max = base_set;
            }
        }

        max.sort_unstable();
        let mut password = String::new();
        for m in max {
            if !password.is_empty() {
                password.push(',');
            }
            password.push(m[0]);
            password.push(m[1]);
        }
        password
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