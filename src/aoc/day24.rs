use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::aoc::AocDay;

pub struct AocDay24 {
    wires: BTreeMap<String, Wire>,
}

impl AocDay for AocDay24 {
    fn new(input: Vec<String>) -> Self {
        let mut wires = BTreeMap::new();
        for line in input {
            if let Some((wire, value)) = line.split_once(": ") {
                wires.insert(wire.to_string(), Wire::Fixed(value == "1"));
            } else if let Some((left, wire)) = line.split_once(" -> ") {
                let w = if let Some((a, b)) = left.split_once(" AND ") {
                    Wire::And(a.to_string(), b.to_string())
                } else if let Some((a, b)) = left.split_once(" OR ") {
                    Wire::Or(a.to_string(), b.to_string())
                } else if let Some((a, b)) = left.split_once(" XOR ") {
                    Wire::Xor(a.to_string(), b.to_string())
                } else {
                    unreachable!()
                };
                wires.insert(wire.to_string(), w);
            }
        }

        Self {wires}
    }

    fn part1(&self) -> String {
        let mut zkeys : Vec<&str> = self.wires.keys().map(String::as_str).filter(|k| k.starts_with('z')).collect();
        zkeys.sort_unstable();
        let mut solved : BTreeMap<&str, bool> = BTreeMap::new();
        let mut to_solve = zkeys.clone();
        while let Some(k) = to_solve.pop() {
            if solved.contains_key(k) {
                continue;
            }
            match self.wires.get(k).unwrap() {
                Wire::Fixed(b) => {
                    solved.insert(k, *b);
                },
                Wire::And(a, b) if solved.contains_key(a.as_str()) && solved.contains_key(b.as_str()) => {
                    solved.insert(k, *solved.get(a.as_str()).unwrap() && *solved.get(b.as_str()).unwrap());
                }
                Wire::Or(a, b) if solved.contains_key(a.as_str()) && solved.contains_key(b.as_str()) => {
                    solved.insert(k, *solved.get(a.as_str()).unwrap() || *solved.get(b.as_str()).unwrap());
                }
                Wire::Xor(a, b) if solved.contains_key(a.as_str()) && solved.contains_key(b.as_str()) => {
                    solved.insert(k, *solved.get(a.as_str()).unwrap() != *solved.get(b.as_str()).unwrap());
                }
                Wire::And(a, _) | Wire::Or(a, _) | Wire::Xor(a, _) if !solved.contains_key(a.as_str()) => {
                    to_solve.push(k);
                    to_solve.push(a);
                },
                Wire::And(_, b) | Wire::Or(_, b) | Wire::Xor(_, b) => {
                    to_solve.push(k);
                    to_solve.push(b);
                },
            }
        }
        let mut result = 0u64;
        for (i, k) in zkeys.iter().enumerate() {
            if Some(true) == solved.get(k).copied() {
                result += 1 << i;
            }
        }

        format!("{result}")
    }
}

#[derive(Clone)]
enum Wire {
    Fixed(bool),
    And(String, String),
    Or(String, String),
    Xor(String, String),
}
