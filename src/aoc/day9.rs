use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::aoc::AocDay;

pub struct AocDay9 {
    layout: Vec<u8>,
}

impl AocDay for AocDay9 {
    fn new(input: Vec<String>) -> Self {
        let layout = input.iter().flat_map(|s| s.trim().chars())
            .map(|c| format!("{c}").parse().unwrap())
            .collect();
        Self {layout}
    }

    fn part1(&self) -> String {
        let mut checksum = 0u64;
        let mut i = 0u64;
        let mut head = 0usize;
        let mut tail = self.layout.len() - 1;
        let mut to_move = self.layout[tail];

        loop {
            // original file
            for _ in 0..self.layout[head] {
                checksum += i * (head / 2) as u64;
                i += 1;
            }
            head += 1;
            if head >= tail {
                break;
            }
            // free space
            for _ in 0..self.layout[head] {
                if to_move == 0 {
                    tail -= 2;
                    if head >= tail {
                        break;
                    }
                    to_move = self.layout[tail];
                }
                checksum += i * (tail / 2) as u64;
                i += 1;
                to_move -= 1;
            }
            head += 1;
            if head >= tail {
                break;
            }
        }
        while to_move > 0 {
            checksum += i * (tail / 2) as u64;
            i += 1;
            to_move -= 1;
        }

        format!("{checksum}")
    }

    fn part2(&self) -> String {
        let mut files : Vec<(usize, u8)> = Vec::with_capacity(self.layout.len() / 2 + 1);
        let mut frees : Vec<(usize, u8)> = Vec::with_capacity(self.layout.len() / 2);
        let mut pos = 0usize;
        for (i, len) in self.layout.iter().copied().enumerate() {
            if i % 2 == 0 {
                files.push((pos, len));
                pos += len as usize;
            } else {
                frees.push((pos, len));
                pos += len as usize;
            }
        }
        for (pos, len) in files.iter_mut().rev() {
            if let Some(free) = frees.iter_mut().find(|(fpos, flen)| fpos < pos && flen >= len) {
                *pos = free.0;
                free.0 += *len as usize;
                free.1 -= *len;
            }
        }

        let checksum : u64 = files.into_iter()
            .enumerate()
            .map(|(id, (pos, len))| (0..len).map(|i| id as u64 * (pos + i as usize) as u64).sum::<u64>())
            .sum();
        format!("{checksum}")
    }
}
