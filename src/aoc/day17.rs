use alloc::format;
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
        let mut output = Vec::<u32>::new();
        let mut a = self.init_a;
        let mut b = self.init_b;
        let mut c = self.init_c;
        let mut ip = 0;
        while ip < self.program.len() {
            let opcode = self.program[ip];
            let operand = self.program[ip + 1];
            let comboperand = match operand {
                0..=3 => operand as u32,
                4 => a,
                5 => b,
                6 => c,
                _ => unreachable!()
            };
            match opcode {
                0 => {
                    // adv
                    a = a / (2u32.pow(comboperand));
                    ip += 2;
                },
                1 => {
                    // bxl
                    b = b ^ operand as u32;
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
                    b = b ^ c;
                    ip += 2;
                },
                5 => {
                    // out
                    output.push(comboperand % 8);
                    ip += 2;
                },
                6 => {
                    // bdv
                    b = a / (2u32.pow(comboperand));
                    ip += 2;
                },
                7 => {
                    // cdv
                    c = a / (2u32.pow(comboperand));
                    ip += 2;
                },
                _ => unreachable!()
            }
        }
        output.into_iter().map(|i| format!("{i}")).reduce(|s1, s2| s1 + "," + &s2).unwrap_or_default()
    }
}
