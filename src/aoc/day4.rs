use crate::aoc::AocDay;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct AocDay4 {
    letters: Vec<Vec<char>>,
}

impl AocDay4 {
    fn width(&self) -> usize {
        self.letters[0].len()
    }

    fn height(&self) -> usize {
        self.letters.len()
    }
}

impl AocDay for AocDay4 {
    fn new(input: Vec<String>) -> Self {
        let letters = input
            .into_iter()
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(line.chars().collect())
                }
            })
            .collect();
        Self { letters }
    }

    fn part1(&self) -> String {
        let mut count = 0u32;

        for x in 0..self.width() - 3 {
            // horizontal to right
            for y in 0..self.height() {
                if self.letters[y][x] == 'X'
                    && self.letters[y][x + 1] == 'M'
                    && self.letters[y][x + 2] == 'A'
                    && self.letters[y][x + 3] == 'S'
                {
                    count += 1;
                }
            }
            // diagonal to bottom right
            for y in 0..self.height() - 3 {
                if self.letters[y][x] == 'X'
                    && self.letters[y + 1][x + 1] == 'M'
                    && self.letters[y + 2][x + 2] == 'A'
                    && self.letters[y + 3][x + 3] == 'S'
                {
                    count += 1;
                }
            }
            // diagonal to top right
            for y in 3..self.height() {
                if self.letters[y][x] == 'X'
                    && self.letters[y - 1][x + 1] == 'M'
                    && self.letters[y - 2][x + 2] == 'A'
                    && self.letters[y - 3][x + 3] == 'S'
                {
                    count += 1;
                }
            }
        }
        for x in 3..self.width() {
            // horizontal to left
            for y in 0..self.height() {
                if self.letters[y][x] == 'X'
                    && self.letters[y][x - 1] == 'M'
                    && self.letters[y][x - 2] == 'A'
                    && self.letters[y][x - 3] == 'S'
                {
                    count += 1;
                }
            }
            // diagonal to bottom left
            for y in 0..self.height() - 3 {
                if self.letters[y][x] == 'X'
                    && self.letters[y + 1][x - 1] == 'M'
                    && self.letters[y + 2][x - 2] == 'A'
                    && self.letters[y + 3][x - 3] == 'S'
                {
                    count += 1;
                }
            }
            // diagonal to top left
            for y in 3..self.height() {
                if self.letters[y][x] == 'X'
                    && self.letters[y - 1][x - 1] == 'M'
                    && self.letters[y - 2][x - 2] == 'A'
                    && self.letters[y - 3][x - 3] == 'S'
                {
                    count += 1;
                }
            }
        }
        for x in 0..self.width() {
            for y in 0..self.height() - 3 {
                if self.letters[y][x] == 'X'
                    && self.letters[y + 1][x] == 'M'
                    && self.letters[y + 2][x] == 'A'
                    && self.letters[y + 3][x] == 'S'
                {
                    count += 1;
                }
            }
            for y in 3..self.height() {
                if self.letters[y][x] == 'X'
                    && self.letters[y - 1][x] == 'M'
                    && self.letters[y - 2][x] == 'A'
                    && self.letters[y - 3][x] == 'S'
                {
                    count += 1;
                }
            }
        }

        count.to_string()
    }

    fn part2(&self) -> String {
        let mut count = 0u32;

        for x in 1..self.width() - 1 {
            for y in 1..self.height() - 1 {
                if self.letters[y][x] != 'A' {
                    continue;
                }
                if (self.letters[y - 1][x - 1] == 'M' && self.letters[y + 1][x + 1] == 'S'
                    || self.letters[y - 1][x - 1] == 'S' && self.letters[y + 1][x + 1] == 'M')
                    && (self.letters[y - 1][x + 1] == 'M' && self.letters[y + 1][x - 1] == 'S'
                        || self.letters[y - 1][x + 1] == 'S' && self.letters[y + 1][x - 1] == 'M')
                {
                    count += 1;
                }
            }
        }

        count.to_string()
    }
}

#[cfg(all(target_os = "linux", test))]
mod test {
    use crate::aoc::AocDay;
    use crate::aoc::day4::AocDay4;
    use alloc::string::ToString;

    const INPUT: &'static str = "MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";

    #[test]
    fn test() {
        let day = AocDay4::new(INPUT.lines().map(ToString::to_string).collect());
        assert_eq!(day.part1(), "18");
        assert_eq!(day.part2(), "9");
    }
}
