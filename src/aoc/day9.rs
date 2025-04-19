use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay9 {
    layout: Vec<u8>,
}

impl AocDay for AocDay9 {
    fn new(input: Vec<String>) -> Self {
        let layout = input.iter().map(|s| s.trim().chars()).flatten()
            .map(|c| format!("{c}").parse().unwrap())
            .collect();
        Self {layout}
    }

    fn part1(&self) -> String {
        let mut checksum = 0u64;
        let mut i = 0u64;
        let mut head = 0usize;
        let mut tail = self.layout.len() - 1;
        let mut to_move = self.layout[tail];

        loop {
            // original file
            for _ in 0..self.layout[head] {
                checksum += i * (head / 2) as u64;
                i += 1;
            }
            head += 1;
            if head >= tail {
                break;
            }
            // free space
            for _ in 0..self.layout[head] {
                if to_move == 0 {
                    tail -= 2;
                    if head >= tail {
                        break;
                    }
                    to_move = self.layout[tail];
                }
                checksum += i * (tail / 2) as u64;
                i += 1;
                to_move -= 1;
            }
            head += 1;
            if head >= tail {
                break;
            }
        }
        while to_move > 0 {
            checksum += i * (tail / 2) as u64;
            i += 1;
            to_move -= 1;
        }

        format!("{checksum}")
    }
}
