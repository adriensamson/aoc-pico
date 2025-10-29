use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use defmt::debug;
use crate::aoc::AocDay;

pub struct AocDay22 {
    secrets: Vec<u32>,
}

impl AocDay for AocDay22 {
    fn new(input: Vec<String>) -> Self {
        Self {
            secrets: input.iter().filter_map(|s| s.parse().ok()).collect(),
        }
    }

    fn part1(&self) -> String {
        let sum: u64 = self.secrets.iter().copied().map(|mut secret| {
            for _ in 0..2000 {
                secret = next_secret(secret);
            }
            secret as u64
        }).sum();
        format!("{sum}")
    }
}

fn next_secret(mut secret: u32) -> u32 {
    secret = ((secret << 6) ^ secret) % 0x100_0000;
    secret = ((secret >> 5) ^ secret) % 0x100_0000;
    secret = ((secret << 11) ^ secret) % 0x100_0000;
    secret
}
