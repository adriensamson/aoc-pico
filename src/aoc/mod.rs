use aoc_pico::shell::{SyncCommand, SyncRunningCommand};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::aoc::day1::AocDay1;
use crate::aoc::day2::AocDay2;
use crate::aoc::day3::AocDay3;
use crate::aoc::day4::AocDay4;
use crate::aoc::day5::AocDay5;
use crate::aoc::day6::AocDay6;
use crate::aoc::day7::AocDay7;
use crate::aoc::day8::AocDay8;
use crate::aoc::day9::AocDay9;
use crate::aoc::day10::AocDay10;
use crate::aoc::day11::AocDay11;
use crate::aoc::day12::AocDay12;
use crate::aoc::day13::AocDay13;
use crate::aoc::day14::AocDay14;
use crate::aoc::day15::AocDay15;
use crate::aoc::day16::AocDay16;
use crate::aoc::day17::AocDay17;
use crate::aoc::day18::AocDay18;
use crate::aoc::day19::AocDay19;
use crate::aoc::day20::AocDay20;
use crate::aoc::day21::AocDay21;
use crate::aoc::day22::AocDay22;
use crate::aoc::day23::AocDay23;
use crate::aoc::day24::AocDay24;
use crate::aoc::day25::AocDay25;

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
mod day17;
mod day18;
mod day19;
mod day20;
mod day21;
mod day22;
mod day23;
mod day24;
mod day25;

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
impl SyncRunningCommand for ErrRunningCommand {
    fn next_sync(&mut self) -> Option<String> {
        self.0.take()
    }
}

impl SyncCommand for AocRunner {
    type RunningCommand = Box<dyn SyncRunningCommand>;
    fn exec_sync(&self, args: Vec<String>, input: Vec<String>) -> Self::RunningCommand {
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
        Box::new(DAYS[day](input))
    }
}

type AocDayFn = fn(Vec<String>) -> Box<dyn SyncRunningCommand + 'static>;

const NB_DAYS: usize = 1 + 25;
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
    AocDay17::run,
    AocDay18::run,
    AocDay19::run,
    AocDay20::run,
    AocDay21::run,
    AocDay22::run,
    AocDay23::run,
    AocDay24::run,
    AocDay25::run,
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

    fn run(input: Vec<String>) -> Box<dyn SyncRunningCommand> {
        Box::new(RunningAoc(Self::new(input), 0))
    }
}

struct RunningAoc<D: AocDay>(D, u8);

impl<D: AocDay> SyncRunningCommand for RunningAoc<D> {
    fn next_sync(&mut self) -> Option<String> {
        match self.1 {
            0 => {
                self.1 = 1;
                Some(String::from("running..."))
            }
            1 => {
                self.1 = 2;
                Some(format!("Part1: {}", self.0.part1()))
            }
            2 => {
                self.1 = 3;
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
