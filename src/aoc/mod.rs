use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub struct AocRunner {
    day: u8,
    reading_input: bool,
    input: Vec<String>,
}

impl AocRunner {
    pub fn new() -> Self {
        Self {
            day: 0,
            reading_input: false,
            input: Vec::new(),
        }
    }

    pub fn push_line(&mut self, line: String) -> Box<dyn Iterator<Item = String>> {
        if self.reading_input {
            if line.trim() == "end" {
                self.reading_input = false;
                Box::new(
                    Some(String::from(">running...")).into_iter().chain(DAYS[self.day as usize](core::mem::take(&mut self.input))),
                )
            } else {
                self.input.push(line);
                Box::new(None.into_iter())
            }
        } else if let Some(day) = line.trim().strip_prefix("day=").and_then(|s| s.parse::<u8>().ok()) {
            if day < NB_DAYS as u8 {
                self.day = day;
                Box::new(Some(format!(">day={day}")).into_iter())
            } else {
                Box::new(Some(String::from(">bad day")).into_iter())
            }
        } else if line.trim() == "input" {
            self.reading_input = true;
            Box::new(Some(String::from(">start of input")).into_iter())
        } else {
            Box::new(Some(String::from(">unknown command")).into_iter())
        }
    }
}

type AocDay = fn(Vec<String>) -> Box<dyn Iterator<Item = String>>;

const NB_DAYS : usize = 1;
const DAYS: [AocDay; NB_DAYS] = [
    test_aoc
];

fn test_aoc(input: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    Box::new([
        format!("lines={}", input.len()),
        format!("max-cols={}", input.iter().map(|l| l.len()).max().unwrap_or_default()),
    ].into_iter())
}
