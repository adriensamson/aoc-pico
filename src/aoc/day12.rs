use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::format;
use alloc::collections::VecDeque;
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
            let regions = Region::split(plots.clone().into());
            cost += regions.iter().map(|r| r.cost1()).sum::<usize>();
        }
        format!("{cost}")
    }

    fn part2(&self) -> String {
        let mut cost = 0usize;
        for plots in self.plots.values() {
            let regions = Region::split(plots.clone().into());
            cost += regions.iter().map(|r| r.cost2()).sum::<usize>();
        }
        format!("{cost}")
    }
}

struct Region {
    plots: Vec<(u8, u8)>,
}

impl Region {
    fn split(mut plots: VecDeque<(u8, u8)>) -> Vec<Self> {
        let mut regions = Vec::new();
        while let Some(first) = plots.pop_front() {
            let mut region = Region { plots: Vec::from([first]) };
            let mut max_row = first.0;
            loop {
                let mut added = false;
                let mut rest = VecDeque::new();
                for (i, p) in plots.iter().copied().enumerate() {
                    if region.plots.iter().copied().any(|e| are_neighbours(e, p)) {
                        added = true;
                        max_row = max_row.max(p.0);
                        region.plots.push(p);
                    } else if p.0 > max_row + 1 {
                        rest.extend(plots.iter().skip(i));
                        break;
                    } else {
                        rest.push_back(p);
                    }
                }
                plots = rest;
                if !added {
                    break;
                }
            }
            region.plots.sort_unstable();
            regions.push(region);
        }
        regions
    }

    fn cost1(&self) -> usize {
        let area = self.plots.len();
        let mut perimeter = area * 4;
        for i in 0..area-1 {
            for j in i+1..area {
                if are_neighbours(self.plots[i], self.plots[j]) {
                    perimeter -= 2;
                } else if self.plots[j].0 > self.plots[i].0 + 1 {
                    break;
                }
            }
        }
        area * perimeter
    }

    fn cost2(&self) -> usize {
        let area = self.plots.len();
        let mut sides = count_up_sides(self.plots.iter().copied());
        // down sides
        let mut plots = self.plots.clone();
        plots.sort_by(|a, b| a.cmp(b).reverse());
        sides += count_up_sides(plots.iter().copied());
        // left sides
        plots = self.plots.iter().copied().map(|(a, b)| (b, a)).collect();
        plots.sort_unstable();
        sides += count_up_sides(plots.iter().copied());
        // right sides
        plots.sort_by(|a, b| a.cmp(b).reverse());
        sides += count_up_sides(plots.iter().copied());
        area * sides
    }
}

fn are_neighbours(p1: (u8, u8), p2: (u8, u8)) -> bool {
    p1.0.abs_diff(p2.0).saturating_add(p1.1.abs_diff(p2.1)) == 1
}

fn count_up_sides(plots: impl Iterator<Item=(u8, u8)>) -> usize {
    let mut sides = 0;
    let mut prev_row : Vec<(u8, u8)> = Vec::new();
    let mut current_row : Vec<(u8, u8)> = Vec::new();
    let mut current_side : Option<(u8, u8)> = None;
    for p in plots {
        if let Some(c) = current_row.last().copied() {
            if c.0 != p.0 {
                prev_row = current_row;
                current_row = Vec::new();
                current_side = None;
            }
        }
        current_row.push(p);
        if prev_row.iter().copied().any(|u| are_neighbours(u, p)) {
            current_side = None;
        } else {
            if let Some(c) = current_side {
                if p.1.abs_diff(c.1) != 1 {
                    sides += 1;
                }
            } else {
                sides += 1;
            }
            current_side = Some(p)
        }
    }
    sides
}
