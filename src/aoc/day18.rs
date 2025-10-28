use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use defmt::debug;
use crate::aoc::AocDay;

pub struct AocDay18 {
    bytes: Vec<(u8, u8)>,
}

impl AocDay for AocDay18 {
    fn new(input: Vec<String>) -> Self {
        let bytes = input.iter()
            .filter_map(|s| {
                let (x, y) = s.split_once(',')?;
                Some((x.parse().ok()?, y.parse().ok()?))
            })
            .collect();
        Self { bytes }
    }

    fn part1(&self) -> String {
        const MAX: u8 = 70; /* 6 in test*/
        const NB: usize = 1024; /* 12 in test */
        let result = find_path::<MAX>(&self.bytes[..NB]);
        format!("{}", result.unwrap())
    }

    fn part2(&self) -> String {
        let mut min = 1024;
        let mut max = self.bytes.len();
        while min != max - 1 {
            let n = min.midpoint(max);
            debug!("testing {}", n);
            if find_path::<70>(&self.bytes[..n]).is_some() {
                min = n;
            } else {
                max = n;
            }
        }
        format!("{},{}", self.bytes[min].0, self.bytes[min].1)
    }
}

fn find_path<const MAX: u8>(corrupted: &[(u8, u8)]) -> Option<u16> {
    let mut visited = BTreeMap::<(u8, u8), u16>::new();
    let mut to_visit = BTreeMap::<(u8, u8), u16>::new();
    to_visit.insert((0, 0), 0);
    let mut result = None;
    loop {
        let Some((&(x, y), &dist)) = to_visit.iter().min_by_key(|(_, d)| **d) else { break; };
        to_visit.remove(&(x, y));
        if corrupted.contains(&(x, y)) {
            continue;
        }
        if x == MAX && y == MAX {
            result = Some(dist);
            break;
        }
        visited.insert((x, y), dist);
        let mut nexts = Vec::with_capacity(4);
        if x > 0 {
            nexts.push((x - 1, y));
        }
        if x < MAX {
            nexts.push((x + 1, y));
        }
        if y > 0 {
            nexts.push((x, y - 1));
        }
        if y < MAX {
            nexts.push((x, y + 1));
        }
        for n in nexts {
            if !visited.contains_key(&n) && !corrupted.contains(&n) {
                to_visit.insert(n, dist + 1);
            }
        }
    }
    result
}