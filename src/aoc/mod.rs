use crate::aoc::day1::AocDay1;
use crate::aoc::day2::AocDay2;
use crate::shell::Command;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

mod day1;
mod day2;

pub struct AocRunner;

impl AocRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AocRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for AocRunner {
    type Output = Box<dyn Iterator<Item = String> + Send>;

    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Self::Output {
        let day = args
            .first()
            .map(String::as_str)
            .unwrap_or("0")
            .parse::<usize>();
        if day.is_err() {
            return Box::new(Some(String::from("bad day")).into_iter());
        }
        let day = day.unwrap();
        if day >= NB_DAYS {
            return Box::new(Some(String::from("bad day")).into_iter());
        }
        Box::new(
            Some(String::from("running..."))
                .into_iter()
                .chain(DAYS[day](input)),
        )
    }
}

type AocDayFn = fn(Vec<String>) -> Box<dyn Iterator<Item = String> + Send>;

const NB_DAYS: usize = 3;
const DAYS: [AocDayFn; NB_DAYS] = [TestDay0::run, AocDay1::run, AocDay2::run];

trait AocDay: Send + Sized
where
    Self: 'static,
{
    fn new(input: Vec<String>) -> Self;

    fn part1(&self) -> String {
        String::new()
    }
    fn part2(&self) -> String {
        String::new()
    }

    fn run(input: Vec<String>) -> Box<dyn Iterator<Item = String> + Send> {
        Box::new(AocIter(Self::new(input), 0))
    }
}

struct AocIter<D: AocDay>(D, u8);

impl<D: AocDay> Iterator for AocIter<D> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1 {
            0 => {
                self.1 = 1;
                Some(format!("Part1: {}", self.0.part1()))
            }
            1 => {
                self.1 = 2;
                Some(format!("Part2: {}", self.0.part2()))
            }
            _ => None,
        }
    }
}

struct TestDay0 {
    input: Vec<String>,
}

impl AocDay for TestDay0 {
    fn new(input: Vec<String>) -> Self {
        Self { input }
    }

    fn part1(&self) -> String {
        format!("lines={}", self.input.len())
    }

    fn part2(&self) -> String {
        format!(
            "max-cols={}",
            self.input.iter().map(|l| l.len()).max().unwrap_or_default()
        )
    }
}
