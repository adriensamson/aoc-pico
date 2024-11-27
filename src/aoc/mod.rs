use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use aoc_pico::shell::{Command};

pub struct AocRunner;

impl AocRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Command for AocRunner {
    type Output = Box<dyn Iterator<Item = String> + Send>;

    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Self::Output {
        let day = args.first().map(String::as_str).unwrap_or("0").parse::<usize>();
        if day.is_err() {
            return Box::new(Some(String::from("bad day")).into_iter());
        }
        let day = day.unwrap();
        if day > NB_DAYS {
            return Box::new(Some(String::from("bad day")).into_iter());
        }
        Box::new(
            Some(String::from("running...")).into_iter().chain(DAYS[day](input)),
        )
    }
}

type AocDay = fn(Vec<String>) -> Box<dyn Iterator<Item = String> + Send>;

const NB_DAYS : usize = 1;
const DAYS: [AocDay; NB_DAYS] = [
    test_aoc
];

fn test_aoc(input: Vec<String>) -> Box<dyn Iterator<Item = String> + Send> {
    Box::new([
        format!("lines={}", input.len()),
        format!("max-cols={}", input.iter().map(|l| l.len()).max().unwrap_or_default()),
    ].into_iter())
}
