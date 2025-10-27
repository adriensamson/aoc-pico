use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Entry;
use alloc::string::String;
use alloc::{format, vec};
use alloc::vec::Vec;
use core::cmp::Reverse;
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
        let corrupted = &self.bytes[..NB];
        let mut visited = BTreeMap::<(u8, u8), usize>::new();
        let mut currents = vec![((0, 0), 0)];
        let mut result = None;
        while let Some(((x, y), dist)) = currents.pop() {
            if corrupted.contains(&(x, y)) {
                continue;
            }
            if x == MAX && y == MAX {
                result = Some(dist);
                break;
            }
            match visited.entry((x, y)) {
                Entry::Occupied(mut occ) => {
                    if *occ.get() < dist {
                        continue;
                    }
                    occ.insert(dist);
                },
                Entry::Vacant(vac) => {
                    vac.insert(dist);
                }
            }
            if x > 0 {
                currents.push(((x - 1, y), dist + 1));
            }
            if x < MAX {
                currents.push(((x + 1, y), dist + 1));
            }
            if y > 0 {
                currents.push(((x, y - 1), dist + 1));
            }
            if y < MAX {
                currents.push(((x, y + 1), dist + 1));
            }
            currents.sort_unstable_by_key(|(_, d)| Reverse(*d));
        }
        format!("{}", result.unwrap())
    }
}