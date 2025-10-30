use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
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
        let mut totals : BTreeMap<[i8; 4], u32> = BTreeMap::new();
        for secret in self.secrets.iter().copied() {
            let mut changes = [0i8; 4];
            let mut prices = BTreeMap::new();
            for (i, (price, change)) in PriceChangeIterator(secret).take(2000).enumerate() {
                changes.rotate_left(1);
                changes[3] = change;
                if i >= 3 && changes.iter().sum::<i8>() >= 0 && total_diff(changes) <= 10 {
                    prices.entry(changes).or_insert(price);
                }
            }
            for (changes, price) in prices {
                *totals.entry(changes).or_default() += price as u32;
            }
        }
        let max = totals.values().max().copied().unwrap();
        format!("{max}")
    }
}

fn next_secret(mut secret: u32) -> u32 {
    secret = ((secret << 6) ^ secret) % 0x100_0000;
    secret = ((secret >> 5) ^ secret) % 0x100_0000;
    secret = ((secret << 11) ^ secret) % 0x100_0000;
    secret
}

struct PriceChangeIterator(u32);
impl Iterator for PriceChangeIterator {
    type Item = (u8, i8);

    fn next(&mut self) -> Option<Self::Item> {
        let prev_price = (self.0 % 10) as u8;
        self.0 = next_secret(self.0);
        let new_price = (self.0 % 10) as u8;
        let change = (new_price as i8) - (prev_price as i8);
        Some((new_price, change))
    }
}

fn total_diff(changes: [i8; 4]) -> i8 {
    let mut pos_diff = 0;
    let mut neg_diff = 0;
    for c in changes {
        if c >= 0 {
            pos_diff += c;
        } else {
            neg_diff += c;
        }
    }
    pos_diff - neg_diff
}