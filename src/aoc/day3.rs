use alloc::string::{ToString, String};
use alloc::vec::Vec;
use crate::aoc::AocDay;

pub struct AocDay3 {
    code: String,
}

impl AocDay for AocDay3 {
    fn new(input: Vec<String>) -> Self {
        Self {
            code: input.join("\n"),
        }
    }

    fn part1(&self) -> String {
        let sum: u32 = (0..self.code.len()-8)
            .filter_map(|i| {
                let s = self.code[i..].strip_prefix("mul(")?;
                let comma = s[..s.len().min(4)].find(',')?;
                let x : u32 = s[..comma].parse().ok()?;
                let s = &s[comma+1..];
                let paren = s[..s.len().min(4)].find(')')?;
                let y : u32 = s[..paren].parse().ok()?;
                Some(x * y)
            }).sum();
        sum.to_string()
    }

    fn part2(&self) -> String {
        let mut enabled = true;
        let sum: u32 = (0..self.code.len()-8)
            .filter_map(|i| {
                if self.code[i..].starts_with("do()") {
                    enabled = true;
                    return None;
                }
                if self.code[i..].starts_with("don't()") {
                    enabled = false;
                    return None;
                }
                if !enabled {
                    return None;
                }
                let s = self.code[i..].strip_prefix("mul(")?;
                let comma = s[..s.len().min(4)].find(',')?;
                let x : u32 = s[..comma].parse().ok()?;
                let s = &s[comma+1..];
                let paren = s[..s.len().min(4)].find(')')?;
                let y : u32 = s[..paren].parse().ok()?;
                Some(x * y)
            }).sum();
        sum.to_string()
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;
    use crate::aoc::day3::AocDay3;
    use crate::aoc::AocDay;

    const INPUT : &'static str = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
    const INPUT2 : &'static str = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

    #[test]
    fn test() {
        let part1 = AocDay3::new(vec![INPUT.into()]);
        assert_eq!(part1.part1(), "161");
        let part2 = AocDay3::new(vec![INPUT2.into()]);
        assert_eq!(part2.part2(), "48");
    }
}
