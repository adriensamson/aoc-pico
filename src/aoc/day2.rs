use crate::aoc::AocDay;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct AocDay2 {
    reports: Vec<Vec<u8>>,
}

impl AocDay for AocDay2 {
    fn new(input: Vec<String>) -> Self {
        let reports = input
            .iter()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| {
                line.split_whitespace()
                    .map(|level| level.parse().ok())
                    .collect::<Option<Vec<_>>>()
            })
            .collect();
        Self { reports }
    }

    fn part1(&self) -> String {
        self.reports
            .iter()
            .filter(|levels| is_safe(levels.iter().copied()))
            .count()
            .to_string()
    }

    fn part2(&self) -> String {
        self.reports
            .iter()
            .filter(|levels| tolerate_one_bad(levels).any(is_safe))
            .count()
            .to_string()
    }
}

fn is_safe(iter: impl Iterator<Item = u8> + Clone) -> bool {
    iter.clone()
        .pairs()
        .all(|(l, r)| (l + 1..=l + 3).contains(&r))
        || iter.pairs().all(|(l, r)| (r + 1..=r + 3).contains(&l))
}

trait Pairs: Iterator<Item = u8> + Sized {
    fn pairs(self) -> PairsIter<Self>;
}

impl<I: Iterator<Item = u8>> Pairs for I {
    fn pairs(self) -> PairsIter<Self> {
        PairsIter {
            inner: self,
            prev: None,
        }
    }
}

struct PairsIter<I: Iterator<Item = u8>> {
    inner: I,
    prev: Option<u8>,
}

impl<I: Iterator<Item = u8>> Iterator for PairsIter<I> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.inner.next()?;
        if let Some(prev) = self.prev {
            self.prev = Some(val);
            Some((prev, val))
        } else {
            self.prev = Some(val);
            self.next()
        }
    }
}

fn tolerate_one_bad(
    levels: &[u8],
) -> impl Iterator<Item = impl Iterator<Item = u8> + Clone + use<'_>> + use<'_> {
    (0..levels.len()).map(|skip| {
        levels[0..skip]
            .iter()
            .copied()
            .chain(levels[skip + 1..].iter().copied())
    })
}

#[cfg(all(target_os = "linux", test))]
mod test {
    use crate::aoc::AocDay;
    use crate::aoc::day2::AocDay2;
    use alloc::string::ToString;

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
        assert_eq!(day.part2(), "4");
    }
}
