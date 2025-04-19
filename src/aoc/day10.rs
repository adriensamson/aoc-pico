use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay10 {
    coords_by_height: [Vec<(i8, i8)>; 10],
}
impl AocDay for AocDay10 {
    fn new(input: Vec<String>) -> Self {
        let mut coords_by_height: [Vec<(i8, i8)>; 10] = Default::default();
        for (r, row) in input.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).enumerate() {
            for (c, h) in row.chars().map(|c| c.to_digit(10).unwrap() as u8).enumerate() {
                coords_by_height[h as usize].push((r as i8, c as i8));
            }
        }
        Self {coords_by_height}
    }

    fn part1(&self) -> String {
        let mut score = 0;
        for start in self.coords_by_height[0].iter() {
            let mut positions = vec![*start];
            for h in 1..=9 {
                positions = self.coords_by_height[h].iter()
                    .copied()
                    .filter(|c| positions.iter().any(|p| (c.0 - p.0).abs() + (c.1 - p.1).abs() == 1))
                    .collect();
            }
            score += positions.len();
        }

        format!("{score}")
    }

    fn part2(&self) -> String {
        let mut score = 0;
        for start in self.coords_by_height[0].iter() {
            let mut positions = vec![*start];
            for h in 1..=9 {
                positions = positions.into_iter().map(|p| {
                    self.coords_by_height[h].iter()
                        .copied()
                        .filter(move |c| (c.0 - p.0).abs() + (c.1 - p.1).abs() == 1)
                }).flatten().collect();
            }
            score += positions.len();
        }

        format!("{score}")
    }
}
