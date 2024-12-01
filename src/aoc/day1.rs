use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::aoc::AocDay;

pub struct AocDay1 {
    left: Vec<u32>,
    right: Vec<u32>,
}

impl AocDay for AocDay1 {
    fn new(input: Vec<String>) -> Self {
        let mut left = Vec::with_capacity(input.len());
        let mut right = Vec::with_capacity(input.len());
        for line in &input {
            let nums : Vec<u32> = line.split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if nums.len() == 2 {
                left.push(nums[0]);
                right.push(nums[1]);
            }
        }
        Self { left, right }
    }

    fn part1(&self) -> String {
        let mut left_sorted = self.left.clone();
        left_sorted.sort_unstable();
        let mut right_sorted = self.right.clone();
        right_sorted.sort_unstable();
        let sum : u32= left_sorted.into_iter().zip(right_sorted)
            .map(|(left, right)| left.abs_diff(right))
            .sum();
        sum.to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::aoc::AocDay;
    use crate::aoc::day1::AocDay1;
    use alloc::string::ToString;

    const DATA : &str = "3   4
4   3
2   5
1   3
3   9
3   3";

    #[test]
    fn test_part1() {
        let day = AocDay1::new(DATA.lines().map(ToString::to_string).collect());
        assert_eq!(day.part1(), "11");
    }
}
