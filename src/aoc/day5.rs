use crate::aoc::AocDay;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use defmt::debug;

pub struct AocDay5 {
    rules: Vec<(u8, u8)>,
    updates: Vec<Vec<u8>>,
}

impl AocDay5 {
    fn is_correct(&self, updates: &[u8]) -> bool {
        for i in 0..updates.len() - 1 {
            for j in i + 1..updates.len() {
                if self.rules.iter().any(|&r| r == (updates[j], updates[i])) {
                    return false;
                }
            }
        }
        true
    }

    fn reorder(&self, updates: &[u8]) -> Vec<u8> {
        let mut reordered = updates.to_vec();
        let mut i = 0;
        while i < reordered.len() - 1 {
            let mut j = i + 1;
            while j < reordered.len() {
                if self
                    .rules
                    .iter()
                    .any(|&r| r == (reordered[j], reordered[i]))
                {
                    reordered.swap(i, j);
                    j = i + 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        reordered
    }
}

impl AocDay for AocDay5 {
    fn new(input: Vec<String>) -> Self {
        let mut lines = input.into_iter().peekable();
        while lines.peek().is_some_and(|s| s.is_empty()) {
            lines.next();
        }
        let mut rules = Vec::new();
        loop {
            let line = lines.next().unwrap();
            if line.is_empty() {
                break;
            }
            let (left, right) = line.split_once('|').unwrap();
            rules.push((left.parse().unwrap(), right.parse().unwrap()));
        }
        let updates = lines
            .filter(|s| !s.is_empty())
            .map(|line| line.split(',').map(|n| n.parse().unwrap()).collect())
            .collect();
        Self { rules, updates }
    }

    fn part1(&self) -> String {
        let sum: u32 = self
            .updates
            .iter()
            .filter(|updates| self.is_correct(updates))
            .map(|updates| updates[updates.len() / 2] as u32)
            .sum();

        sum.to_string()
    }

    fn part2(&self) -> String {
        let sum: u32 = self
            .updates
            .iter()
            .filter(|updates| !self.is_correct(updates))
            .map(|updates| self.reorder(updates))
            .inspect(|u| debug!("{=[u8]}", u))
            .map(|updates| updates[updates.len() / 2] as u32)
            .sum();

        sum.to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::aoc::day5::AocDay5;
    use crate::aoc::AocDay;
    use alloc::string::ToString;

    const INPUT: &'static str = "47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";

    #[test]
    fn test() {
        let day = AocDay5::new(INPUT.lines().map(ToString::to_string).collect());
        assert_eq!(day.part1(), "143");
        assert_eq!(day.part2(), "123");
    }
}
