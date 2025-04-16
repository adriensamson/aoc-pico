use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay7 {
    equations: Vec<Equation>,
}

impl AocDay for AocDay7 {
    fn new(input: Vec<String>) -> Self {
        let equations = input.into_iter().filter_map(|s| Equation::from_str(&s)).collect();
        Self { equations }
    }

    fn part1(&self) -> String {
        let sum : u64 = self.equations.iter().filter_map(|eq| if eq.is_valid() { Some(eq.result )} else { None }).sum();
        format!("{}", sum)
    }
}

struct Equation {
    result: u64,
    operands: Vec<u64>
}

impl Equation {
    pub fn from_str(s: &str) -> Option<Self> {
        let (r, ops) = s.split_once(": ")?;
        let result = r.parse().unwrap();
        let operands = ops.split(' ').filter_map(|o| o.parse::<u64>().ok()).collect();
        Some(Self {result, operands})
    }

    pub fn is_valid(&self) -> bool {
        is_valid(self.result, &self.operands)
    }
}

fn is_valid(result: u64, operands: &[u64]) -> bool {
    match operands {
        [] => false,
        [op] => result == *op,
        [op1, op2] => result == op1 * op2 || result == op1 + op2,
        [head @ .., tail] =>
            (result >= *tail && is_valid(result - tail, head))
            || (result % tail == 0 && is_valid(result / tail, head))
    }
}
