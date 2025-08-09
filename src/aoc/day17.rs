use alloc::{format, vec};
use alloc::string::String;
use alloc::vec::Vec;
use defmt::debug;
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
                    a /= 2u32.pow(comboperand);
                    ip += 2;
                },
                1 => {
                    // bxl
                    b ^= operand as u32;
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

    fn part2(&self) -> String {
        let mut solutions = Solutions::new();
        let mut output_idx = 0;
        let mut a = Exprs::new(Expr::a());
        let mut b = Exprs::new(Expr::zero());
        let mut c = Exprs::new(Expr::zero());
        let mut ip = 0;
        while ip < self.program.len() {
            let opcode = self.program[ip];
            let operand = self.program[ip + 1];
            let comboperand = || match operand {
                0..=3 => Exprs::new(Expr::literal3(operand)),
                4 => a.clone(),
                5 => b.clone(),
                6 => c.clone(),
                _ => unreachable!()
            };
            debug!("opcode: {}", opcode);
            crate::memory::debug_heap_size("opcode");
            match opcode {
                0 => {
                    // adv
                    a.shift_right(comboperand());
                    ip += 2;
                },
                1 => {
                    // bxl
                    b.xor(comboperand());
                    ip += 2;
                },
                2 => {
                    // bst
                    let mut val = comboperand();
                    val.mod8();
                    b = val;
                    ip += 2;
                },
                3 => {
                    if output_idx == self.program.len() {
                        solutions.apply(a, 0);
                        break;
                    }
                    ip = operand as usize;
                },
                4 => {
                    // bxc
                    b.xor(c.clone());
                    ip += 2;
                },
                5 => {
                    // out
                    let mut val = comboperand();
                    val.mod8();
                    solutions.apply(val, self.program[output_idx]);
                    a.apply_solutions(&solutions);
                    b.apply_solutions(&solutions);
                    c.apply_solutions(&solutions);
                    output_idx += 1;
                    ip += 2;
                },
                6 => {
                    // bdv
                    let val = a.clone();
                    a.shift_right(comboperand());
                    b = val;
                    ip += 2;
                },
                7 => {
                    // cdv
                    let val = a.clone();
                    a.shift_right(comboperand());
                    c = val;
                    ip += 2;
                },
                _ => unreachable!()
            }
        }
        format!("{}", solutions.min().unwrap())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Bit {
    Zero,
    One,
    A(u8),
    NotA(u8),
}

impl core::ops::BitXor for Bit {
    type Output = Result<Self, ()>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Zero, other) | (other, Bit::Zero) => Ok(other),
            (Bit::One, Bit::One) => Ok(Bit::Zero),
            (Bit::A(a), Bit::One) | (Bit::One, Bit::A(a)) => Ok(Bit::NotA(a)),
            (Bit::NotA(a), Bit::One) | (Bit::One, Bit::NotA(a)) => Ok(Bit::A(a)),
            (Bit::A(a), Bit::A(b)) if a == b => Ok(Bit::Zero),
            (Bit::NotA(a), Bit::NotA(b)) if a == b => Ok(Bit::Zero),
            (Bit::A(a), Bit::NotA(b)) if a == b => Ok(Bit::One),
            (Bit::NotA(a), Bit::A(b)) if a == b => Ok(Bit::One),
            _ => Err(())
        }
    }
}

const NBITS : usize = 48;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Expr([Bit; NBITS]);

impl Expr {
    fn a() -> Self {
        Self(core::array::from_fn(|i| Bit::A(i as u8)))
    }

    fn zero() -> Self {
        Self([Bit::Zero; NBITS])
    }

    fn literal3(i: u8) -> Self {
        let mut lit = Self::zero();
        if i & 1 == 1 {
            lit.0[0] = Bit::One;
        }
        if i & 2 == 2 {
            lit.0[1] = Bit::One;
        }
        if i & 4 == 4 {
            lit.0[2] = Bit::One;
        }
        lit
    }

    fn mod8(&mut self) {
        for b in 3..NBITS {
            self.0[b] = Bit::Zero;
        }
    }

    fn shift_right(&mut self, s: usize) {
        self.0.rotate_left(s);
        for b in NBITS-s..NBITS {
            self.0[b] = Bit::Zero;
        }
    }

    fn xor(&self, rhs: Expr) -> Result<Expr, Vec<(Constraints, Expr)>> {
        crate::memory::debug_heap_size("start inner xor");
        let mut results = vec![(Constraints::none(), Expr::zero())];
        for b in 0..NBITS {
            match self.0[b] ^ rhs.0[b] {
                Ok(bit) => results.iter_mut().for_each(|res| res.1.0[b] = bit),
                Err(()) => {
                    let s = format!("{:?} XOR {:?}", self.0, rhs.0);
                    debug!("{}", s.as_str());
                    let (b2, inv) = match self.0[b] {
                        Bit::A(b2) => (b2, false),
                        Bit::NotA(b2) => (b2, true),
                        _ => unreachable!(),
                    };
                    let mut opposite = Vec::new();
                    for res in &mut results {
                        match res.0.get_bit(b2) {
                            Constraint::None => {
                                res.0.set_bit(b2, if inv { Constraint::One } else { Constraint::Zero });
                                res.1.0[b2 as usize] = rhs.0[b];
                                let mut opp = res.clone();
                                opp.0.set_bit(b2, if inv { Constraint::Zero } else { Constraint::One });
                                opp.1.0[b2 as usize] = (Bit::One ^ rhs.0[b]).unwrap();
                                opposite.push(opp);
                            },
                            Constraint::Zero => {
                                res.1.0[b2 as usize] = if inv { (Bit::One ^ rhs.0[b]).unwrap() } else { rhs.0[b] };
                            }
                            Constraint::One => {
                                res.1.0[b2 as usize] = if !inv { (Bit::One ^ rhs.0[b]).unwrap() } else { rhs.0[b] };
                            }
                        }
                    }
                    results.extend(opposite);
                }
            }
        }
        crate::memory::debug_heap_size("end inner xor");
        if results.len() == 1 {
            Ok(results.into_iter().next().unwrap().1)
        } else {
            Err(results)
        }
    }

    fn apply_constraints(&mut self, constraints: &Constraints) {
        for b in &mut self.0 {
            if let Bit::A(i) = b {
                match constraints.get_bit(*i) {
                    Constraint::None => {},
                    Constraint::Zero => *b = Bit::Zero,
                    Constraint::One => *b = Bit::One,
                }
            } else if let Bit::NotA(i) = b {
                match constraints.get_bit(*i) {
                    Constraint::None => {},
                    Constraint::Zero => *b = Bit::One,
                    Constraint::One => *b = Bit::Zero,
                }
            }
        }
    }

    fn solve(&self, i: u8) -> Option<Constraints> {
        let mut res = Constraints::none();
        for b in 0..NBITS {
            let bit = if b < 3 { (i >> b) & 1 } else { 0 };
            match self.0[b] {
                Bit::A(a) => res.set_bit(a, if bit == 1 { Constraint::One } else { Constraint::Zero }),
                Bit::NotA(a) => res.set_bit(a, if bit == 0 { Constraint::One } else { Constraint::Zero }),
                Bit::One if bit == 0 => return None,
                Bit::Zero if bit == 1 => return None,
                _ => {}
            }
        }
        Some(res)
    }
}

impl TryFrom<Expr> for usize {
    type Error = Vec<(Constraints, usize)>;

    fn try_from(value: Expr) -> Result<Self, Self::Error> {
        let mut cases = vec![(Constraints::none(), 0)];
        for (i, bit) in value.0.iter().enumerate() {
            match bit {
                Bit::One => cases.iter_mut().for_each(|c| c.1 |= 1 << i),
                Bit::Zero => {},
                Bit::A(a) => {
                    cases.iter_mut().for_each(|c| c.0.set_bit(*a, Constraint::Zero));
                    let mut ones = cases.clone();
                    ones.iter_mut().for_each(|c| {c.0.set_bit(*a, Constraint::One); c.1 |= 1 << i});
                    cases.extend(ones)
                },
                Bit::NotA(a) => {
                    cases.iter_mut().for_each(|c| c.0.set_bit(*a, Constraint::One));
                    let mut ones = cases.clone();
                    ones.iter_mut().for_each(|c| {c.0.set_bit(*a, Constraint::Zero); c.1 |= 1 << i});
                    cases.extend(ones)
                },
            }
        }
        if cases.len() == 1 {
            Ok(cases[0].1)
        } else {
            Err(cases)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Constraint {
    None,
    One,
    Zero,
}

#[derive(Copy, Clone, Debug, Default)]
struct Bitmask(u64);
impl Bitmask {
    fn get_bit(&self, i: u8) -> bool {
        self.0 >> i & 1 == 1
    }
    fn set_bit(&mut self, i: u8) {
        self.0 |= 1 << i;
    }
    fn clear_bit(&mut self, i: u8) {
        self.0 &= !(1 << i);
    }
}

#[derive(Clone, Debug, Default)]
struct Constraints {
    ones: Bitmask,
    zeros: Bitmask,
}

impl Constraints {
    fn none() -> Self {
        Default::default()
    }

    fn get_bit(&self, b: u8) -> Constraint {
        if self.ones.get_bit(b) {
            Constraint::One
        } else if self.zeros.get_bit(b) {
            Constraint::Zero
        } else {
            Constraint::None
        }
    }

    fn set_bit(&mut self, b: u8, c: Constraint) {
        match c {
            Constraint::One => {
                self.ones.set_bit(b);
                self.zeros.clear_bit(b);
            },
            Constraint::Zero => {
                self.zeros.set_bit(b);
                self.ones.clear_bit(b);
            },
            _ => {
                self.ones.clear_bit(b);
                self.zeros.clear_bit(b);
            }
        }
    }

    fn combine(&self, other: &Self) -> Option<Self> {
        let mut res = Self::none();
        for i in 0..NBITS as u8 {
            match (self.get_bit(i), other.get_bit(i)) {
                (Constraint::One, Constraint::Zero) | (Constraint::Zero, Constraint::One) => return None,
                (Constraint::None, other) => res.set_bit(i, other),
                (first, _) => res.set_bit(i, first),
            }
        }
        Some(res)
    }

    fn exactly_1bit_diff(&self, other: &Self) -> Option<u8> {
        let mut diff = None;
        for b in 0..NBITS as u8 {
            if self.get_bit(b) != other.get_bit(b) {
                if diff.is_none() {
                    diff = Some(b);
                } else {
                    return None;
                }
            }
        }
        diff
    }

    fn min(&self) -> u64 {
        self.ones.0
    }
}

#[derive(Clone, Debug)]
struct Exprs(Vec<(Constraints, Expr)>);

impl Exprs {
    fn new(expr: Expr) -> Self {
        Self(vec![(Constraints::none(), expr)])
    }

    fn mod8(&mut self) {
        for (_, e) in &mut self.0 {
            e.mod8();
        }
    }

    fn xor(&mut self, rhs: Exprs) {
        crate::memory::debug_heap_size("start_xor");
        let mut result = Vec::new();
        for (left_cons, left_expr) in &self.0 {
            for (right_cons, right_expr) in &rhs.0 {
                if let Some(cons) = left_cons.combine(right_cons) {
                    let mut left = left_expr.clone();
                    let mut right = right_expr.clone();
                    left.apply_constraints(&cons);
                    right.apply_constraints(&cons);
                    match left.xor(right) {
                        Ok(expr) => result.push((cons, expr)),
                        Err(cases) => {
                            for (right2_cons, e) in cases {
                                result.push((right2_cons, e));
                            }
                        }
                    }
                }
            }
        }
        self.0 = result;
        crate::memory::debug_heap_size("before_recombine");
        self.recombine();
    }

    fn shift_right(&mut self, rhs: Exprs) {
        let mut result = Vec::new();
        for (left_cons, left_expr) in &self.0 {
            for (right_cons, right_expr) in &rhs.0 {
                if let Some(cons) = left_cons.combine(right_cons) {
                    let mut left = left_expr.clone();
                    let mut right = right_expr.clone();
                    left.apply_constraints(&cons);
                    right.apply_constraints(&cons);
                    match usize::try_from(right) {
                        Ok(s) => {
                            left.shift_right(s);
                            result.push((cons, left));
                        },
                        Err(cases) => {
                            for (right2_cons, n) in cases {
                                let mut left2 = left.clone();
                                left2.apply_constraints(&right2_cons);
                                left2.shift_right(n);
                                result.push((right2_cons, left2));
                            }
                        }
                    }
                }
            }
        }
        self.0 = result;
        self.recombine();
    }

    fn apply_solutions(&mut self, solutions: &Solutions) {
        let mut res = Vec::new();
        for (cons, expr) in &self.0 {
            for sol in &solutions.0 {
                if let Some(cons2) = cons.combine(sol) {
                    let mut expr2 = expr.clone();
                    expr2.apply_constraints(&cons2);
                    res.push((cons2, expr2));
                }
            }
        }
        self.0 = res;
        self.recombine();
    }

    fn recombine(&mut self) {
        while let Some(idx) = 'rm: loop {
            for i in 0..self.0.len()-1 {
                for j in i+1..self.0.len() {
                    if let Some(b) = self.0[i].0.exactly_1bit_diff(&self.0[j].0).filter(|_| self.0[i].1 == self.0[j].1) {
                        self.0[i].0.set_bit(b, Constraint::None);
                        break 'rm Some(j);
                    }
                }
            }
            break 'rm None;
        } {
            self.0.remove(idx);
        }
    }
}

struct Solutions(Vec<Constraints>);

impl Solutions {
    fn new() -> Self {
        Self(vec![Constraints::none()])
    }

    fn apply(&mut self, exprs: Exprs, n: u8) {
        let mut res = Vec::new();
        for cons1 in &self.0 {
            for (cons2, expr) in &exprs.0 {
                if let Some(cons) = expr.solve(n).and_then(|cons3| cons3.combine(cons2)).and_then(|c| c.combine(cons1)) {
                    res.push(cons);
                }
            }
        }
        self.0 = res
    }

    fn min(&self) -> Option<u64> {
        self.0.iter().map(|cons| cons.min()).min()
    }
}
