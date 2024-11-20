use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

pub(crate) struct Console {
    incomplete_seq: Vec<u8>,
    current_line: String,
    lines: VecDeque<String>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            incomplete_seq: Vec::new(),
            current_line: String::new(),
            lines: VecDeque::new(),
        }
    }

    pub fn push(&mut self, buf: &[u8]) {
        let mut state = State::Normal;
        let mut incomplete_seq = Vec::with_capacity(4);
        for &b in self.incomplete_seq.iter().chain(buf) {
            match state {
                State::Normal => {
                    match b {
                        b'\n' | b'\r' => {
                            self.lines.push_back(core::mem::take(&mut self.current_line))
                        },
                        b'\x1b' => {
                            state = State::InEscape
                        },
                        b'\x00'..=b'\x1f' => {
                            // ignore
                        },
                        b'\x20'..=b'\x7e' => {
                            self.current_line.push(b as char);
                        },
                        b'\x7f' => {
                            // ignore
                        },
                        b'\x80'..=b'\xff' => {
                            state = State::InUtf8;
                            incomplete_seq.push(b);
                        }
                    }
                },
                State::InUtf8 => {
                    if !matches!(b, b'\x80'..=b'\xff') {
                        // invalid sequence
                        incomplete_seq.clear();
                        state = State::Normal;
                    } else {
                        incomplete_seq.push(b);
                        if let Ok(s) = core::str::from_utf8(&incomplete_seq) {
                            self.current_line.push_str(s);
                            incomplete_seq.clear();
                            state = State::Normal;
                        }
                    }
                },
                State::InEscape => {
                    if incomplete_seq.is_empty() {
                        if b != b'[' {
                            // ignore and treat as 1byte seq
                            state = State::Normal
                        } else {
                            incomplete_seq.push(b);
                        }
                    } else if incomplete_seq.is_empty() {
                        incomplete_seq.push(b);
                    } else {
                        if matches!(b, b'\x40'..=b'\x7e') {
                            // end of sequence, ignore for now
                            incomplete_seq.clear();
                            state = State::Normal
                        } else {
                            incomplete_seq.push(b);
                        }
                    }
                }
            }
        }
    }

    pub fn pop_line(&mut self) -> Option<String> {
        self.lines.pop_front()
    }

    pub fn pop_current_line(&mut self) -> String {
        core::mem::take(&mut self.current_line)
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Eq, PartialEq)]
enum State {
    Normal,
    InUtf8,
    InEscape,
}
