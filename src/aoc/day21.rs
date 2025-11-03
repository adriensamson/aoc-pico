use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::debug;
use crate::aoc::AocDay;

pub struct AocDay21 {
    codes: Vec<String>,
}

impl AocDay for AocDay21 {
    fn new(input: Vec<String>) -> Self {
        Self { codes: input.into_iter().filter(|s| !s.is_empty()).collect() }
    }

    fn part1(&self) -> String {
        let sum : usize = self.codes.iter().map(|code| {
            let code1 = apply(code, num_path);
            debug!("{}", code1.as_str());
            let code2 = apply(&code1, dir_path);
            debug!("{}", code2.as_str());
            let code3 = apply(&code2, dir_path);
            debug!("{}", code3.as_str());
            let n = code[0..3].parse::<usize>().unwrap();
            n * code3.len()
        })
            .sum();
        format!("{}", sum)
    }

    fn part2(&self) -> String {
        let mut dir_table : BTreeMap<&str, BTreeMap<&str, usize>> = BTreeMap::new();
        let mut sum = 0u64;
        for code in self.codes.iter() {
            let mut seqs : BTreeMap<&str, u64> = BTreeMap::new();
            let mut from = 'A';
            for c in code.chars() {
                *seqs.entry(num_path(from, c)).or_default() += 1;
                from = c;
            }
            for _ in 0..25 {
                let mut seqs2 = BTreeMap::new();
                for (s, n) in seqs {
                    let dir_seq = dir_table.entry(s).or_insert_with(|| {
                        let mut s2 = BTreeMap::new();
                        let mut from = 'A';
                        for c in s.chars() {
                            *s2.entry(dir_path(from, c)).or_default() += 1;
                            from = c;
                        }
                        *s2.entry(dir_path(from, 'A')).or_default() += 1;
                        s2
                    });
                    for (s2, n2) in &*dir_seq {
                        *seqs2.entry(*s2).or_default() += *n2 as u64 * n;
                    }
                }
                seqs = seqs2;
            }
            let len = seqs.iter().map(|(s, n)| (s.len() + 1) as u64 * n).sum::<u64>();
            debug!("{}", len);
            let n = code[0..3].parse::<usize>().unwrap() as u64;
            sum += n * len;
        }

        format!("{sum}")
    }
}

const fn num_path(from: char, to: char) -> &'static str {
    if from == to {
        return "";
    }
    match from {
        'A' => match to {
            '0' => "<",
            '1' => "^<<",
            '2' => "<^",
            '3' => "^",
            '4' => "^^<<",
            '5' => "<^^",
            '6' => "^^",
            '7' => "^^^<<",
            '8' => "<^^^",
            '9' => "^^^",
            _ => unreachable!(),
        },
        '0' => match to {
            'A' => ">",
            '1' => "^<",
            '2' => "^",
            '3' => "^>",
            '4' => "^^<",
            '5' => "^^",
            '6' => "^^>",
            '7' => "^^^<",
            '8' => "^^^",
            '9' => "^^^>",
            _ => unreachable!(),
        },
        '1' => match to {
            '0' => "v>",
            'A' => "v>>",
            '2' => ">",
            '3' => ">>",
            '4' => "^",
            '5' => "^>",
            '6' => "^>>",
            '7' => "^^",
            '8' => "^^>",
            '9' => "^^>>",
            _ => unreachable!(),
        },
        '2' => match to {
            '0' => "v",
            'A' => "v>",
            '1' => "<",
            '3' => ">",
            '4' => "<^",
            '5' => "^",
            '6' => "^>",
            '7' => "<^^",
            '8' => "^^",
            '9' => "^^>",
            _ => unreachable!(),
        },
        '3' => match to {
            '0' => "<v",
            'A' => "v",
            '1' => "<<",
            '2' => "<",
            '4' => "<<^",
            '5' => "<^",
            '6' => "^",
            '7' => "<<^^",
            '8' => "<^^",
            '9' => "^^",
            _ => unreachable!(),
        },
        '4' => match to {
            '0' => ">vv",
            'A' => ">>vv",
            '1' => "v",
            '2' => "v>",
            '3' => "v>>",
            '5' => ">",
            '6' => ">>",
            '7' => "^",
            '8' => "^>",
            '9' => "^>>",
            _ => unreachable!(),
        },
        '5' => match to {
            '0' => "vv",
            'A' => "vv>",
            '1' => "<v",
            '2' => "v",
            '3' => "v>",
            '4' => "<",
            '6' => ">",
            '7' => "<^",
            '8' => "^",
            '9' => "^>",
            _ => unreachable!(),
        },
        '6' => match to {
            '0' => "<vv",
            'A' => "vv",
            '1' => "<<v",
            '2' => "<v",
            '3' => "v",
            '4' => "<<",
            '5' => "<",
            '7' => "<<^",
            '8' => "<^",
            '9' => "^",
            _ => unreachable!(),
        },
        '7' => match to {
            '0' => ">vvv",
            'A' => ">>vvv",
            '1' => "vv",
            '2' => "vv>",
            '3' => "vv>>",
            '4' => "v",
            '5' => "v>",
            '6' => "v>>",
            '8' => ">",
            '9' => ">>",
            _ => unreachable!(),
        },
        '8' => match to {
            '0' => "vvv",
            'A' => "vvv>",
            '1' => "<vv",
            '2' => "vv",
            '3' => "vv>",
            '4' => "<v",
            '5' => "v",
            '6' => "v>",
            '7' => "<",
            '9' => ">",
            _ => unreachable!(),
        },
        '9' => match to {
            '0' => "<vvv",
            'A' => "vvv",
            '1' => "<<vv",
            '2' => "<vv",
            '3' => "vv",
            '4' => "<<v",
            '5' => "<v",
            '6' => "v",
            '7' => "<<",
            '8' => "<",
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

const fn dir_path(from: char, to: char) -> &'static str {
    if from == to {
        return "";
    }
    match from {
        'A' => match to {
            '^' => "<",
            '<' => "v<<",
            'v' => "<v",
            '>' => "v",
            _ => unreachable!(),
        },
        '^' => match to {
            'A' => ">",
            '<' => "v<",
            'v' => "v",
            '>' => "v>",
            _ => unreachable!(),
        },
        '<' => match to {
            '^' => ">^",
            'A' => ">>^",
            'v' => ">",
            '>' => ">>",
            _ => unreachable!(),
        },
        'v' => match to {
            '^' => "^",
            'A' => "^>",
            '<' => "<",
            '>' => ">",
            _ => unreachable!(),
        },
        '>' => match to {
            '^' => "<^",
            'A' => "^",
            '<' => "<<",
            'v' => "<",
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn apply(s: &str, f: fn(char, char) -> &'static str) -> String {
    let mut result = String::new();
    let mut from = 'A';
    for to in s.chars() {
        result.push_str(f(from, to));
        result.push('A');
        from = to;
    }
    result
}