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

    fn part2(&self) -> String {
        let mut max = 0;
        for i in -3..=3 {
            for j in (-6-i).max(-3)..=(6-i).min(3) {
                for k in (-6-i-j).max(-3)..=(6-i-j).min(3) {
                    for l in (-6-i-j-k).max(-3)..=(6-i-j-k).min(3) {
                        let sum = self.secrets.iter().copied().filter_map(|secret| price_for_seq(secret, [i, j, k, l])).sum();
                        if sum > max {
                            debug!("{}, {}, {}, {} => {}", i, j, k, l, sum);
                            max = sum;
                        }
                    }
                }
            }
        }
        format!("{max}")
    }
}

fn next_secret(mut secret: u32) -> u32 {
    secret = ((secret << 6) ^ secret) % 0x100_0000;
    secret = ((secret >> 5) ^ secret) % 0x100_0000;
    secret = ((secret << 11) ^ secret) % 0x100_0000;
    secret
}

struct PriceIterator(u32);
impl Iterator for PriceIterator {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        let price = (self.0 % 10) as i8;
        self.0 = next_secret(self.0);
        Some(price)
    }
}

fn price_for_seq(secret: u32, seq: [i8; 4]) -> Option<i32> {
    let mut changes = [0i8; 4];
    let mut previous = 0;
    for (i, price) in PriceIterator(secret).take(2001).enumerate() {
        changes.rotate_left(1);
        changes[3] = price - previous;
        previous = price;
        if i >= 4 && changes == seq {
            return Some(price as i32);
        }
    }
    None
}