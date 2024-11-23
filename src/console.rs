use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use rp_pico::hal::uart::{UartDevice, ValidUartPinout, Writer};

pub(crate) struct InputParser {
    incomplete_seq: Vec<u8>,
    current_line: String,
    lines: VecDeque<String>,
}

impl InputParser {
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
                    } else if matches!(b, b'\x40'..=b'\x7e') {
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

    pub fn pop_line(&mut self) -> Option<String> {
        self.lines.pop_front()
    }

    pub fn pop_current_line(&mut self) -> String {
        core::mem::take(&mut self.current_line)
    }
}

impl Default for InputParser {
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

pub struct Console<Output, Runner> {
    parser: InputParser,
    current_line: String,
    output: Output,
    runner: Runner,
}

pub trait ConsoleRunner {
    type Output<'a>: Iterator<Item = String> where Self: 'a;

    fn push_line(&mut self, line: String) -> Self::Output<'_>;
}

pub trait ConsoleOutput {
    fn output(&mut self, line: &[u8]);
}

impl<Output: ConsoleOutput, Runner: ConsoleRunner> Console<Output, Runner> {
    pub fn new(output: Output, runner: Runner) -> Self {
        Self {
            parser: InputParser::new(),
            current_line: String::new(),
            output,
            runner,
        }
    }

    pub fn push(&mut self, buf: &[u8]) {
        self.parser.push(buf);
        while let Some(mut line) = self.parser.pop_line() {
            self.output.output(line.as_bytes());
            self.output.output(b"\r\n");

            if !self.current_line.is_empty() {
                self.current_line.push_str(&line);
                core::mem::swap(&mut self.current_line, &mut line);
                self.current_line.clear();
            }
            for result in self.runner.push_line(line) {
                self.output.output(result.as_bytes());
                self.output.output(b"\r\n");
            }
        }
        let curr = self.parser.pop_current_line();
        self.output.output(curr.as_bytes());
        self.current_line.push_str(&curr);
    }

    pub fn writeln(&mut self, line: &str) {
        self.output.output(line.as_bytes());
        self.output.output(b"\r\n");
    }
}

pub struct ConsoleUartWriter<U: UartDevice, P: ValidUartPinout<U>>(pub Writer<U, P>);

impl<U: UartDevice, P: ValidUartPinout<U>> ConsoleOutput for ConsoleUartWriter<U, P> {
    fn output(&mut self, mut line: &[u8]) {
        loop {
            match self.0.write_raw(line) {
                Ok([]) => break,
                Ok(rem) => line = rem,
                Err(_) => {},
            }
        }
    }
}
