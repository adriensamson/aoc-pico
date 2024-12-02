use alloc::vec::Vec;
use alloc::string::{ToString, String};
use crate::aoc::AocDay;

pub struct AocDay2 {
    reports: Vec<Vec<u8>>,
}

impl AocDay for AocDay2 {
    fn new(input: Vec<String>) -> Self {
        let reports = input.iter()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| line.split_whitespace().map(|level| level.parse().ok()).collect::<Option<Vec<_>>>())
            .collect();
        Self { reports }
    }

    fn part1(&self) -> String {
        self.reports.iter()
            .filter(|levels|
                levels.windows(2).all(|l| (l[0] + 1..=l[0] + 3).contains(&l[1]))
                || levels.windows(2).all(|l| (l[1] + 1..=l[1] + 3).contains(&l[0]))
            )
            .count().to_string()
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;
    use crate::aoc::AocDay;
    use crate::aoc::day2::AocDay2;

    const INPUT: &str = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";

    #[test]
    fn test() {
        let day = AocDay2::new(INPUT.lines().map(ToString::to_string).collect());
        assert_eq!(day.part1(), "2");
    }
}
