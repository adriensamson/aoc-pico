use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay12 {
    plots: BTreeMap<char, Vec<(u8, u8)>>
}

impl AocDay for AocDay12 {
    fn new(input: Vec<String>) -> Self {
        let mut plots : BTreeMap<char, Vec<(u8, u8)>> = BTreeMap::new();
        for (r, row) in input.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).enumerate() {
            for (c, char) in row.chars().enumerate() {
                plots.entry(char).or_default().push((r as u8, c as u8));
            }
        }
        Self { plots }
    }

    fn part1(&self) -> String {
        let mut cost = 0usize;
        for plots in self.plots.values() {
            let regions = Region::split(plots.clone());
            cost += regions.iter().map(|r| r.cost()).sum::<usize>();
        }
        format!("{cost}")
    }
}

struct Region {
    plots: Vec<(u8, u8)>,
}

impl Region {
    fn split(mut plots: Vec<(u8, u8)>) -> Vec<Self> {
        let mut regions = Vec::new();
        while let Some(last) = plots.pop() {
            let mut region = Region { plots: Vec::from([last]) };
            loop {
                let (neighbours, rest) = plots.into_iter().partition(|p| region.plots.iter().any(|e| are_neighbours(*e, *p)));
                plots = rest;
                if neighbours.is_empty() {
                    break;
                }
                region.plots.extend(neighbours);
            }
            regions.push(region);
        }
        regions
    }

    fn cost(&self) -> usize {
        let area = self.plots.len();
        let mut perimeter = area * 4;
        for i in 0..area-1 {
            for j in i+1..area {
                if are_neighbours(self.plots[i], self.plots[j]) {
                    perimeter -= 2;
                }
            }
        }
        area * perimeter
    }
}

fn are_neighbours(p1: (u8, u8), p2: (u8, u8)) -> bool {
    p1.0.abs_diff(p2.0).saturating_add(p1.1.abs_diff(p2.1)) == 1
}
