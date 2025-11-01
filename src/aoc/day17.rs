use alloc::{format};
use alloc::string::String;
use alloc::vec::Vec;
use crate::aoc::AocDay;

#[derive(Default)]
pub struct AocDay17 {
    init_a: u32,
    init_b: u32,
    init_c: u32,
    program: Vec<u8>,
}

impl AocDay for AocDay17 {
    fn new(input: Vec<String>) -> Self {
        let mut day = AocDay17::default();
        for line in input {
            if let Some(a) = line.strip_prefix("Register A: ") {
                day.init_a = a.parse().unwrap();
            } else if let Some(b) = line.strip_prefix("Register B: ") {
                day.init_b = b.parse().unwrap();
            } else if let Some(c) = line.strip_prefix("Register C: ") {
                day.init_c = c.parse().unwrap();
            } else if let Some(p) = line.strip_prefix("Program: ") {
                day.program = p.split(',').map(|i| i.parse().unwrap()).collect();
            }
        }
        day
    }

    fn part1(&self) -> String {
        run_program(self.init_a as u64, self.init_b, self.init_c, &self.program).into_iter()
            .map(|i| format!("{i}")).reduce(|s1, s2| s1 + "," + &s2).unwrap_or_default()
    }

    fn part2(&self) -> String {
        let mut result = 0u64;
        let len = self.program.len();
        for pos in (3..len).rev() {
            for i in 0..8 {
                let out = run_program(result | i << (pos * 3), self.init_b, self.init_c, &self.program);
                if out.len() == len && self.program[pos..] == out[pos..] {
                    result |= i << (pos * 3);
                    break;
                }
            }
        }
        for n in 0..2<<9 {
            let out = run_program(result | n, self.init_b, self.init_c, &self.program);
            if out.len() == len && self.program == out {
                result |= n;
                break
            }
        }
        format!("{result}")
    }
}

fn run_program(init_a: u64, init_b: u32, init_c: u32, program: &[u8]) -> Vec<u8> {
    let mut output = Vec::<u8>::new();
    let mut a = init_a;
    let mut b = init_b as u64;
    let mut c = init_c as u64;
    let mut ip = 0;
    while ip < program.len() {
        let opcode = program[ip];
        let operand = program[ip + 1];
        let comboperand = match operand {
            0..=3 => operand as u64,
            4 => a,
            5 => b,
            6 => c,
            _ => unreachable!()
        };
        match opcode {
            0 => {
                // adv
                a >>= comboperand;
                ip += 2;
            },
            1 => {
                // bxl
                b ^= operand as u64;
                ip += 2;
            },
            2 => {
                // bst
                b = comboperand & 0x7;
                ip += 2;
            },
            3 => {
                // jnz
                if a != 0 {
                    ip = operand as usize;
                } else {
                    ip += 2;
                }
            },
            4 => {
                // bxc
                b ^= c;
                ip += 2;
            },
            5 => {
                // out
                output.push((comboperand % 8) as u8);
                ip += 2;
            },
            6 => {
                // bdv
                b = a >> comboperand;
                ip += 2;
            },
            7 => {
                // cdv
                c = a >> comboperand;
                ip += 2;
            },
            _ => unreachable!()
        }
    }
    output
}
