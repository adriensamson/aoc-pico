use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use defmt::debug;
use crate::aoc::AocDay;

pub struct AocDay14 {
    robots: Vec<Robot>,
}

struct Robot {
    pos: (i32, i32),
    velocity: (i32, i32),
}

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
        const WIDTH: usize = 101;
        const HEIGHT: usize = 103;
        let positions : Vec<(usize, usize)> = self.robots.iter()
            .map(|r| (r.pos.0 + 100 * r.velocity.0, r.pos.1 + 100 * r.velocity.1))
            .map(|(x, y)| (x % WIDTH as i32, y % HEIGHT as i32))
            .map(|(x, y)| ((x + WIDTH as i32) % WIDTH as i32, (y + HEIGHT as i32) % HEIGHT as i32))
            .map(|(x, y)| (x as usize, y as usize))
            .collect();
        let q1 = positions.iter().filter(|(x, y)| *x < WIDTH / 2 && *y < HEIGHT / 2).count();
        let q2 = positions.iter().filter(|(x, y)| *x > WIDTH / 2 && *y < HEIGHT / 2).count();
        let q3 = positions.iter().filter(|(x, y)| *x < WIDTH / 2 && *y > HEIGHT / 2).count();
        let q4 = positions.iter().filter(|(x, y)| *x > WIDTH / 2 && *y > HEIGHT / 2).count();
        format!("{}", q1 * q2 * q3 * q4)
    }
}
