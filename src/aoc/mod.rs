use crate::aoc::day1::AocDay1;
use crate::aoc::day2::AocDay2;
use crate::aoc::day3::AocDay3;
use crate::aoc::day4::AocDay4;
use crate::aoc::day5::AocDay5;
use crate::aoc::day6::AocDay6;
use crate::aoc::day7::AocDay7;
use aoc_pico::shell::{Command, RunningCommand};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::future::{Future, ready};
use core::pin::Pin;
use crate::aoc::day10::AocDay10;
use crate::aoc::day11::AocDay11;
use crate::aoc::day12::AocDay12;
use crate::aoc::day13::AocDay13;
use crate::aoc::day14::AocDay14;
use crate::aoc::day15::AocDay15;
use crate::aoc::day16::AocDay16;
use crate::aoc::day8::AocDay8;
use crate::aoc::day9::AocDay9;

pub mod coord;

mod day1;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;

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

struct ErrRunningCommand(Option<String>);
impl RunningCommand for ErrRunningCommand {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<String>>>> {
        Box::pin(ready(self.0.take()))
    }
}

impl Command for AocRunner {
    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Box<dyn RunningCommand> {
        let day = args
            .first()
            .map(String::as_str)
            .unwrap_or("0")
            .parse::<usize>();
        if day.is_err() {
            return Box::new(ErrRunningCommand(Some(String::from("bad day"))));
        }
        let day = day.unwrap();
        if day >= NB_DAYS {
            return Box::new(ErrRunningCommand(Some(String::from("bad day"))));
        }
        DAYS[day](input)
    }
}

type AocDayFn = fn(Vec<String>) -> Box<dyn RunningCommand>;

const NB_DAYS: usize = 1 + 16;
const DAYS: [AocDayFn; NB_DAYS] = [
    TestDay0::run,
    AocDay1::run,
    AocDay2::run,
    AocDay3::run,
    AocDay4::run,
    AocDay5::run,
    AocDay6::run,
    AocDay7::run,
    AocDay8::run,
    AocDay9::run,
    AocDay10::run,
    AocDay11::run,
    AocDay12::run,
    AocDay13::run,
    AocDay14::run,
    AocDay15::run,
    AocDay16::run,
];

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

    fn run(input: Vec<String>) -> Box<dyn RunningCommand> {
        Box::new(RunningAoc(Self::new(input), 0))
    }
}

struct RunningAoc<D: AocDay>(D, u8);

impl<D: AocDay> RunningCommand for RunningAoc<D> {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<String>>>> {
        match self.1 {
            0 => {
                self.1 = 1;
                Box::pin(ready(Some(String::from("running..."))))
            }
            1 => {
                self.1 = 2;
                Box::pin(ready(Some(format!("Part1: {}", self.0.part1()))))
            }
            2 => {
                self.1 = 3;
                Box::pin(ready(Some(format!("Part2: {}", self.0.part2()))))
            }
            _ => Box::pin(ready(None)),
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
