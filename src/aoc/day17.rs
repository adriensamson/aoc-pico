use alloc::{format, vec};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
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
            debug!("comboperand");
            let comboperand = match operand {
                0..=3 => Exprs::new(Expr::literal3(operand)),
                4 => a.clone(),
                5 => b.clone(),
                6 => c.clone(),
                _ => unreachable!()
            };
            debug!("opcode: {}", opcode);
            crate::debug_heap_size("opcode");
            match opcode {
                0 => {
                    // adv
                    a.shift_right(comboperand);
                    ip += 2;
                },
                1 => {
                    // bxl
                    b.xor(comboperand);
                    ip += 2;
                },
                2 => {
                    // bst
                    let mut val = comboperand;
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

                    let mut val = comboperand;
                    val.mod8();
                    debug!("apply");
                    solutions.apply(val, self.program[output_idx]);
                    debug!("solutions len: {}", solutions.0.len());
                    a.apply_solutions(&solutions);
                    b.apply_solutions(&solutions);
                    c.apply_solutions(&solutions);
                    output_idx += 1;
                    ip += 2;
                },
                6 => {
                    // bdv
                    let val = a.clone();
                    a.shift_right(comboperand);
                    b = val;
                    ip += 2;
                },
                7 => {
                    // cdv
                    let val = a.clone();
                    a.shift_right(comboperand);
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
#[repr(u8)]
enum Bit {
    Zero,
    One,
}

impl core::ops::Not for Bit {
    type Output = Bit;
    fn not(self) -> Self::Output {
        match self {
            Bit::Zero => Bit::One,
            Bit::One => Bit::Zero,
        }
    }
}

impl core::ops::BitXor for Bit {
    type Output = Bit;
    fn bitxor(self, rhs: Self) -> Self::Output {
        if self == rhs { Self::Zero } else { Self::One }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
struct XoredA {
    a: Bitmask,
    not_a: Bitmask,
}

impl XoredA {
    fn new(i: u8) -> Self {
        let mut res = Self::default();
        res.a.set_bit(i);
        res
    }

    fn apply_constraints(&mut self, constraints: &Constraints) {
        let mut should_inv = false;
        for b in 0..NBITS as u8 {
            if let Constraint::Fixed(bit) = constraints.get_bit(b) {
                match bit {
                    Bit::Zero => {
                        self.a.clear_bit(b);
                        if self.not_a.get_bit(b) {
                            self.not_a.clear_bit(b);
                            should_inv = !should_inv;
                        }
                    },
                    Bit::One => {
                        self.not_a.clear_bit(b);
                        if self.a.get_bit(b) {
                            self.a.clear_bit(b);
                            should_inv = !should_inv;
                        }
                    }
                }
            }
        }
        if should_inv {
            *self = !*self
        }
    }

    fn iter_bits(&self) -> impl Iterator<Item=(u8, Bit)> {
        (0..NBITS as u8).filter_map(|bit| {
            if self.a.get_bit(bit) && !self.not_a.get_bit(bit) {
                Some((bit, Bit::Zero))
            } else if self.not_a.get_bit(bit) && !self.a.get_bit(bit) {
                Some((bit, Bit::One))
            } else {
                None
            }
        })
    }
}

impl core::ops::Not for XoredA {
    type Output = XoredA;
    fn not(self) -> Self::Output {
        Self {
            a: self.not_a,
            not_a: self.a,
        }
    }
}

impl core::ops::BitXor for XoredA {
    type Output = XoredA;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            a: self.a ^ rhs.a,
            not_a: self.not_a ^ rhs.not_a,
        }
    }
}

impl From<XoredA> for ExprBit {
    fn from(value: XoredA) -> Self {
        if value.a == value.not_a {
            ExprBit::Known(if value.a.0.count_ones() % 2 == 0 {
                Bit::Zero
            } else {
                Bit::One
            })
        } else {
            ExprBit::XoredA(Box::new(value))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ExprBit {
    Known(Bit),
    XoredA(Box<XoredA>),
}

impl ExprBit {
    const ZERO : Self = Self::Known(Bit::Zero);
    const ONE : Self = Self::Known(Bit::One);
}

impl core::ops::BitXor<Bit> for ExprBit {
    type Output = Self;
    fn bitxor(self, other: Bit) -> Self::Output {
        if other == Bit::Zero {
            return self;
        }
        match self {
            Self::Known(bit) => Self::Known(!bit),
            Self::XoredA(xoreda) => Self::XoredA(Box::new(!(*xoreda))),
        }
    }
}

impl core::ops::BitXor for ExprBit {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ExprBit::Known(bit), other) | (other, ExprBit::Known(bit)) => other ^ bit,
            (ExprBit::XoredA(a), ExprBit::XoredA(b)) => (*a ^ *b).into(),
        }
    }
}

const NBITS : usize = 48;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Expr(Box<[ExprBit; NBITS]>);

impl Expr {
    fn a() -> Self {
        Self(Box::new(core::array::from_fn(|i| ExprBit::XoredA(Box::new(XoredA::new(i as u8))))))
    }

    fn zero() -> Self {
        Self(Box::new([ExprBit::ZERO; NBITS]))
    }

    fn literal3(i: u8) -> Self {
        let mut lit = Self::zero();
        if i & 1 == 1 {
            lit.0[0] = ExprBit::ONE;
        }
        if i & 2 == 2 {
            lit.0[1] = ExprBit::ONE;
        }
        if i & 4 == 4 {
            lit.0[2] = ExprBit::ONE;
        }
        lit
    }

    fn mod8(&mut self) {
        for b in 3..NBITS {
            self.0[b] = ExprBit::ZERO;
        }
    }

    fn shift_right(&mut self, s: usize) {
        self.0.rotate_left(s);
        for b in NBITS-s..NBITS {
            self.0[b] = ExprBit::ZERO;
        }
    }

    fn xor(&self, rhs: Expr) -> Self {
        let mut res = Expr::zero();
        for b in 0..NBITS {
            res.0[b] = self.0[b].clone() ^ rhs.0[b].clone();
        }
        res
    }

    fn apply_constraints(&mut self, constraints: &Constraints) {
        for b in &mut *self.0 {
            if let ExprBit::XoredA(x) = b {
                x.apply_constraints(constraints);
                *b = (**x).into();
            }
        }
    }

    fn solve(&self, i: u8) -> Option<Constraints> {
        let mut res = Constraints::none();
        for b in 0..NBITS {
            let bit = if b < 3 { (i >> b) & 1 } else { 0 };
            let bit = if bit == 1 { Bit::One } else { Bit::Zero };
            match &self.0[b] {
                ExprBit::XoredA(x) => {
                    for (i, bit) in x.iter_bits() {
                        res.set_bit(i, Constraint::Fixed(bit));
                    }
                },
                ExprBit::Known(bit2) if *bit2 != bit => return None,
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
                ExprBit::Known(Bit::One) => cases.iter_mut().for_each(|c| c.1 |= 1 << i),
                ExprBit::Known(Bit::Zero) => {},
                ExprBit::XoredA(a) => {
                    for (ab, fb) in a.iter_bits() {
                        cases.iter_mut().for_each(|c| c.0.set_bit(ab, Constraint::Fixed(fb)));
                        let mut ones = cases.clone();
                        ones.iter_mut().for_each(|c| {
                            c.0.set_bit(ab, Constraint::Fixed(!fb));
                            c.1 |= 1 << i
                        });
                        cases.extend(ones)
                    }
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
#[repr(u8)]
enum Constraint {
    None,
    Fixed(Bit),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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

impl core::ops::BitXor for Bitmask {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

#[derive(Clone, Default)]
struct Constraints {
    ones: Bitmask,
    zeros: Bitmask,
}

impl Debug for Constraints {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for b in (0..NBITS).rev() {
            if self.ones.get_bit(b as u8) {
                write!(f, "1")?;
            } else if self.zeros.get_bit(b as u8) {
                write!(f, "0")?;
            } else {
                write!(f, ".")?;
            }
        }
        Ok(())
    }
}

impl Constraints {
    fn none() -> Self {
        Default::default()
    }

    fn get_bit(&self, b: u8) -> Constraint {
        if self.ones.get_bit(b) {
            Constraint::Fixed(Bit::One)
        } else if self.zeros.get_bit(b) {
            Constraint::Fixed(Bit::Zero)
        } else {
            Constraint::None
        }
    }

    fn set_bit(&mut self, b: u8, c: Constraint) {
        match c {
            Constraint::Fixed(Bit::One) => {
                self.ones.set_bit(b);
                self.zeros.clear_bit(b);
            },
            Constraint::Fixed(Bit::Zero) => {
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
                (Constraint::Fixed(a), Constraint::Fixed(b)) if a != b => return None,
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
        crate::debug_heap_size("start_xor");
        let mut result = Vec::new();
        for (left_cons, left_expr) in &self.0 {
            for (right_cons, right_expr) in &rhs.0 {
                if let Some(cons) = left_cons.combine(right_cons) {
                    let mut left = left_expr.clone();
                    let mut right = right_expr.clone();
                    left.apply_constraints(&cons);
                    right.apply_constraints(&cons);
                    result.push((cons, left.xor(right)));
                }
            }
        }
        self.0 = result;
        crate::debug_heap_size("before_recombine");
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
        debug!("solutions len: {}", res.len());
        res.sort_unstable_by_key(|sol| sol.ones.0);
        for r in &res {
            debug!("{}", format!("{r:?}").as_str());
        }
        if res.len() > 3 {
            debug!("truncate!");
            res.truncate(3);
        }
        self.0 = res
    }

    fn min(&self) -> Option<u64> {
        self.0.iter().map(|cons| cons.min()).min()
    }
}
