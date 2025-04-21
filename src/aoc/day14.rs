use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use alloc::collections::BTreeSet;
use defmt::debug;
use crate::aoc::AocDay;

pub struct AocDay14 {
    robots: Vec<Robot>,
}

impl AocDay14 {
    fn after(&self, n: usize) -> impl Iterator<Item=(usize, usize)> + use<'_> {
        self.robots.iter()
            .map(move |r| (r.pos.0 + n as i32 * r.velocity.0, r.pos.1 + n as i32 * r.velocity.1))
            .map(|(x, y)| (x % WIDTH as i32, y % HEIGHT as i32))
            .map(|(x, y)| ((x + WIDTH as i32) % WIDTH as i32, (y + HEIGHT as i32) % HEIGHT as i32))
            .map(|(x, y)| (x as usize, y as usize))
    }
}

struct Robot {
    pos: (i32, i32),
    velocity: (i32, i32),
}

const WIDTH: usize = 101;
const HEIGHT: usize = 103;

impl AocDay for AocDay14 {
    fn new(input: Vec<String>) -> Self {
        let robots = input.iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let (p, v) = s.split_once(' ').unwrap();
                let (px, py) = p.strip_prefix("p=").unwrap().split_once(',').unwrap();
                let (vx, vy) = v.strip_prefix("v=").unwrap().split_once(',').unwrap();
                Robot {
                    pos: (px.parse().unwrap(), py.parse().unwrap()),
                    velocity: (vx.parse().unwrap(), vy.parse().unwrap()),
                }
            }).collect();
        Self { robots }
    }

    fn part1(&self) -> String {
        let positions : Vec<_> = self.after(100).collect();
        let q1 = positions.iter().filter(|(x, y)| *x < WIDTH / 2 && *y < HEIGHT / 2).count();
        let q2 = positions.iter().filter(|(x, y)| *x > WIDTH / 2 && *y < HEIGHT / 2).count();
        let q3 = positions.iter().filter(|(x, y)| *x < WIDTH / 2 && *y > HEIGHT / 2).count();
        let q4 = positions.iter().filter(|(x, y)| *x > WIDTH / 2 && *y > HEIGHT / 2).count();
        format!("{}", q1 * q2 * q3 * q4)
    }

    fn part2(&self) -> String {
        for i in 1..10_000 {
            let mut positions = BTreeSet::new();
            positions.extend(self.after(i));
            if has_tree(&positions) {
                let mut s = format!("After {i} steps\n");
                for r in 0..HEIGHT {
                    for c in 0..WIDTH {
                        s += if positions.contains(&(c, r)) { "#" } else { " " };
                    }
                    s += "\n";
                }
                return s;
            }
            debug!("Not found after {}", i);
        }
        "No tree found :(".into()
    }
}

const PATTERN_W : usize = 5;
const PATTERN_H : usize = 3;
const PATTERN : [(usize, usize, bool); const { PATTERN_W * PATTERN_H }] = [
    (0, 0, false), (1, 0, false), (2, 0, true), (3, 0, false), (4, 0, false),
    (0, 1, false), (1, 1, true ), (2, 1, true), (3, 1, true ), (4, 1, false),
    (0, 2, true ), (1, 2, true ), (2, 2, true), (3, 2, true ), (4, 2, true ),
];

fn has_tree(positions: &BTreeSet<(usize, usize)>) -> bool {
    for (c, r) in positions.iter()
        .copied()
        .filter(|(x, y)| (PATTERN_W/2..WIDTH-PATTERN_W/2-1).contains(x) && (0..HEIGHT-PATTERN_H).contains(y)
    ) {
        if PATTERN.iter().copied().all(|(x, y, on)| positions.contains(&(c - PATTERN_W/2 + x, r + y)) == on) {
            return true;
        }
    }
    false
}
