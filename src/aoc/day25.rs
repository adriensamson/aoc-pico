use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use defmt::debug;
use crate::aoc::AocDay;

pub struct AocDay25 {
    locks: Vec<[u8; 5]>,
    keys: Vec<[u8; 5]>,
}

impl AocDay for AocDay25 {
    fn new(mut input: Vec<String>) -> Self {
        let mut locks = Vec::new();
        let mut keys = Vec::new();
        let mut key_or_lock = None;
        let mut heights = [0; 5];
        input.push(String::new());
        for line in input {
            if line.is_empty() {
                match key_or_lock {
                    None => {},
                    Some(true) => {
                        for h in &mut heights {
                            *h -= 1;
                        }
                        keys.push(heights);
                    },
                    Some(false) => locks.push(heights),
                }
                heights = [0; 5];
                key_or_lock = None;
            } else if key_or_lock.is_some() {
                for (i, c) in line.chars().enumerate() {
                    if c == '#' {
                        heights[i] += 1;
                    }
                }
            } else {
                key_or_lock = Some(line == ".....");
            }
        }

        Self {keys, locks}
    }

    fn part1(&self) -> String {
        let mut count = 0usize;
        for key in &self.keys {
            for lock in &self.locks {
                count += key.iter().zip(lock).all(|(k, l)| k + l <= 5) as usize;
            }
        }
        format!("{count}")
    }
}
