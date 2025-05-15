use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay13 {
    machines: Vec<Machine>,
}

struct Machine {
    button_a: (i32, i32),
    button_b: (i32, i32),
    prize: (i32, i32),
}

impl Machine {
    fn part1(&self) -> u32 {
        // p_x = t_a * a_x + t_b * b_x
        // p_y = t_a * a_y + t_b * b_y
        // b_y * p_x - b_x * p_y = t_a * (b_y * a_x - b_x * a_y)
        // a_y * p_x - a_x * p_y = t_b * (a_y * b_x - a_x * b_y)
        let div = self.button_b.1 * self.button_a.0 - self.button_b.0 * self.button_a.1;
        if div == 0 {
            return 0;
        }
        let t_a = ((self.button_b.1 * self.prize.0 - self.button_b.0 * self.prize.1) as f64 / div as f64) as i32;
        let t_b = -((self.button_a.1 * self.prize.0 - self.button_a.0 * self.prize.1) as f64 / div as f64) as i32;
        if (0..=100).contains(&t_a) && (0..=100).contains(&t_b) && self.prize.0 == t_a * self.button_a.0 + t_b * self.button_b.0 {
            return (t_a * 3 + t_b) as u32;
        }
        0
    }

    fn part2(&self) -> u64 {
        const BUMP : f64 = 10_000_000_000_000f64;
        let div = self.button_b.1 * self.button_a.0 - self.button_b.0 * self.button_a.1;
        if div == 0 {
            return 0;
        }
        let t_a = ((self.button_b.1 as f64 * (BUMP + self.prize.0 as f64) - self.button_b.0 as f64 * (BUMP + self.prize.1 as f64)) / div as f64) as i64;
        let t_b = -((self.button_a.1 as f64 * (BUMP + self.prize.0 as f64) - self.button_a.0 as f64 * (BUMP + self.prize.1 as f64)) / div as f64) as i64;
        if t_a >= 0 && t_b >= 0 && (BUMP as i64 + self.prize.0 as i64) == t_a * self.button_a.0 as i64 + t_b * self.button_b.0 as i64 {
            return (t_a * 3 + t_b) as u64;
        }
        0
    }
}

impl AocDay for AocDay13 {
    fn new(input: Vec<String>) -> Self {
        let mut button_a = None;
        let mut button_b = None;
        let mut prize = None;
        let mut machines = Vec::new();
        for line in input.iter().map(|s| s.trim()) {
            if line.is_empty() {
                continue;
            }
            if let Some(xy) = line.strip_prefix("Button A: ") {
                let (x, y) = xy.split_once(", ").unwrap();
                let x = x.strip_prefix("X+").and_then(|s| s.parse().ok()).unwrap();
                let y = y.strip_prefix("Y+").and_then(|s| s.parse().ok()).unwrap();
                button_a = Some((x, y));
            } else if let Some(xy) = line.strip_prefix("Button B: ") {
                let (x, y) = xy.split_once(", ").unwrap();
                let x = x.strip_prefix("X+").and_then(|s| s.parse().ok()).unwrap();
                let y = y.strip_prefix("Y+").and_then(|s| s.parse().ok()).unwrap();
                button_b = Some((x, y));
            } else if let Some(xy) = line.strip_prefix("Prize: ") {
                let (x, y) = xy.split_once(", ").unwrap();
                let x = x.strip_prefix("X=").and_then(|s| s.parse().ok()).unwrap();
                let y = y.strip_prefix("Y=").and_then(|s| s.parse().ok()).unwrap();
                prize = Some((x, y));
            }
            if let (Some(a), Some(b), Some(p)) = (button_a, button_b, prize) {
                machines.push(Machine {
                    button_a: a,
                    button_b: b,
                    prize: p,
                });
            }
        }

        Self {machines}
    }

    fn part1(&self) -> String {
        let count : u32 = self.machines.iter().map(|m| m.part1()).sum();
        format!("{count}")
    }

    fn part2(&self) -> String {
        let count : u64 = self.machines.iter().map(|m| m.part2()).sum();
        format!("{count}")
    }
}
