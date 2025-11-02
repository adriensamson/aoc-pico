use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::aoc::AocDay;
use crate::debug;

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
                    Wire::BinaryOp(Op::And, a.to_string(), b.to_string())
                } else if let Some((a, b)) = left.split_once(" OR ") {
                    Wire::BinaryOp(Op::Or, a.to_string(), b.to_string())
                } else if let Some((a, b)) = left.split_once(" XOR ") {
                    Wire::BinaryOp(Op::Xor, a.to_string(), b.to_string())
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
                Wire::BinaryOp(op, a, b) if solved.contains_key(a.as_str()) && solved.contains_key(b.as_str()) => {
                    let a = *solved.get(a.as_str()).unwrap();
                    let b  = *solved.get(b.as_str()).unwrap();
                    let res = match op {
                        Op::And => a & b,
                        Op::Or => a | b,
                        Op::Xor => a ^ b,
                    };
                    solved.insert(k, res);
                }
                Wire::BinaryOp(_, a, _) if !solved.contains_key(a.as_str()) => {
                    to_solve.push(k);
                    to_solve.push(a);
                },
                Wire::BinaryOp(_, _, b) => {
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

    fn part2(&self) -> String {
        let mut rewired = Rewired::new(&self.wires);

        let z00 = rewired.find(Op::Xor, "x00", "y00").unwrap();
        if z00.as_str() != "z00" {
            rewired.swap("z00", &z00);
        }
        let mut carry = rewired.find(Op::And, "x00", "y00").unwrap();

        // z = carry ^ (x ^ y)
        // carry2 = (x & y) | (carry & (x ^ y))
        // w = x ^ y
        // a = x & y
        // z = c ^ w
        // k = c & w
        // c+ = a | k

        for bit in 1..=43 {

            let x = format!("x{bit:02}");
            let y = format!("y{bit:02}");
            let z_str = format!("z{bit:02}");

            let w = rewired.find_or_swap(Op::Xor, &x, &y, None);
            let _ = rewired.find_or_swap(Op::Xor, &carry, &w, Some(z_str));

            let a = rewired.find_or_swap(Op::And, &x, &y, None);
            let k = rewired.find_or_swap(Op::And, &carry, &w, None);
            carry = rewired.find_or_swap(Op::Or, &a, &k, None);
        }
        // don't need to check z44, it should have been already swapped if needed

        rewired.swaps.sort_unstable();
        rewired.swaps.dedup();
        rewired.swaps.join(",")
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Op {
    And,
    Or,
    Xor,
}

#[derive(Clone)]
enum Wire {
    Fixed(bool),
    BinaryOp(Op, String, String),
}

struct Rewired {
    wires: BTreeMap<String, Wire>,
    swaps: Vec<String>,
}

impl Rewired {
    fn new(wires: &BTreeMap<String, Wire>) -> Self {
        Self {
            wires: wires.clone(),
            swaps: Vec::new(),
        }
    }

    fn swap(&mut self, a: &str, b: &str) {
        debug!("swap {} with {}", a, b);
        self.swaps.push(a.into());
        self.swaps.push(b.into());
        let wire_a = self.wires.remove(a).unwrap();
        let wire_b = self.wires.remove(b).unwrap();
        self.wires.insert(a.to_string(), wire_b);
        self.wires.insert(b.to_string(), wire_a);
    }

    fn find(&self, op: Op, left: &str, right: &str) -> Option<String> {
        self.wires.iter()
            .find_map(|(name, w)| matches!(w, Wire::BinaryOp(op2, a, b) | Wire::BinaryOp(op2, b, a) if op == *op2 && a.as_str() == left && b.as_str() == right).then_some(name))
            .cloned()
    }

    fn find_partial(&self, op: Op, one: &str) -> Option<(String, String)> {
        self.wires.iter()
            .find_map(|(name, w)| match w {
                Wire::BinaryOp(op2, a, b) if op == *op2 && a.as_str() == one => Some((name.clone(), b.clone())),
                Wire::BinaryOp(op2, a, b) if op == *op2 && b.as_str() == one => Some((name.clone(), a.clone())),
                _ => None,
            })
    }

    fn find_or_swap(&mut self, op: Op, left: &str, right: &str, zname: Option<String>) -> String {
        let found = self.find(op, left, right);
        match found {
            Some(name) => {
                if zname.is_none() && name.starts_with('z') {
                    panic!("{name} should not start with z");
                } else if let Some(z) = zname && z != name {
                    self.swap(&z, &name);
                    z
                } else {
                    name
                }
            },
            None => {
                match self.find_partial(op, left) {
                    None => match self.find_partial(op, right) {
                        None => panic!("no partial match"),
                        Some((res, intermediate)) => {
                            self.swap(&intermediate, left);
                            res
                        },
                    },
                    Some((res, intermediate)) => {
                        self.swap(&intermediate, right);
                        res
                    }

                }
            }
        }
    }
}
