use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use defmt::{debug, Formatter};
use rp_pico::hal::uart::{UartDevice, ValidUartPinout, Writer};

#[allow(dead_code)]
pub enum Input {
    Line(String),
    IncompleteLine(String),
    Control(char),
    EscapeSequence(EscapeSequence),
    InvalidByteSequence(Vec<u8>),
}

#[allow(dead_code)]
pub enum EscapeSequence {
    Unknown(Vec<u8>),
}

impl From<Vec<u8>> for EscapeSequence {
    fn from(value: Vec<u8>) -> Self {
        debug!("escape sequence: {=[u8]:X}", value);
        EscapeSequence::Unknown(value)
    }
}

#[derive(Default)]
pub(crate) struct InputParser {
    buffer: VecDeque<u8>,
}

impl InputParser {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    pub fn push(&mut self, buf: &[u8]) {
        self.buffer.extend(buf);
    }
}

impl Iterator for InputParser {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = State::Normal;
        let mut current_line = String::new();
        loop {
            let b = match self.buffer.pop_front() {
                Some(b) => b,
                None => {
                    debug!("no more bytes");
                    self.buffer.extend(state.as_bytes());
                    return if current_line.is_empty() { None } else { Some(Input::IncompleteLine(current_line)) }
                },
            };
            debug!("state={:?} b={:X}", state, b);
            match state {
                State::Normal => {
                    match b {
                        b'\n' | b'\r' => {
                            return Some(Input::Line(current_line));
                        },
                        b'\x1b' => {
                            state = State::InEscape(Vec::from([b]))
                        },
                        b'\x00'..=b'\x1f' | b'\x7f' => {
                            debug!("control: {:X}", b);
                            return Some(Input::Control(b as char))
                        },
                        b'\x20'..=b'\x7e' => {
                            current_line.push(b as char);
                        },
                        b'\x80'..=b'\xff' => {
                            state = State::InUtf8(Vec::from([b]));
                        }
                    }
                },
                State::InUtf8(mut v) => {
                    v.push(b);
                    if !matches!(b, b'\x80'..=b'\xff') {
                        // invalid sequence
                        return Some(Input::InvalidByteSequence(v));
                    } else if let Ok(s) = core::str::from_utf8(&v) {
                        current_line.push_str(s);
                        state = State::Normal;
                    } else {
                        state = State::InUtf8(v);
                    }
                },
                State::InEscape(mut v) => {
                    v.push(b);
                    if v.len() == 2 {
                        if b == b'[' {
                            debug!("CSI");
                            state = State::InEscape(v);
                        } else {
                            debug!("1byte escape");
                            return Some(Input::EscapeSequence(EscapeSequence::from(v)));
                        }
                    } else if matches!(b, b'\x40'..=b'\x7e') {
                        debug!("end of sequence");
                        // end of sequence
                        return Some(Input::EscapeSequence(EscapeSequence::from(v)));
                    } else {
                        debug!("continue");
                        state = State::InEscape(v);
                    }
                }
            }
        }
    }
}

#[derive(Eq, PartialEq)]
enum State {
    Normal,
    InUtf8(Vec<u8>),
    InEscape(Vec<u8>),
}

impl State {
    fn as_bytes(&self) -> &[u8] {
        match self {
            State::Normal => &[],
            State::InUtf8(v) => v,
            State::InEscape(v) => v,
        }
    }
}

impl defmt::Format for State {
    fn format(&self, fmt: Formatter) {
        match self {
            State::Normal =>  defmt::write!(fmt, "normal"),
            State::InUtf8(v) =>  defmt::write!(fmt, "inutf8: {=[u8]:X}", v),
            State::InEscape(v) =>  defmt::write!(fmt, "inescape: {=[u8]:X}", v),
        }
    }
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
        while let Some(input) = self.parser.next() {
            match input {
                Input::Line(mut line) => {
                    if !self.current_line.is_empty() {
                        self.current_line.push_str(&line);
                        core::mem::swap(&mut self.current_line, &mut line);
                        self.current_line.clear();
                    }
                    self.output.output(line.as_bytes());
                    self.output.output(b"\r\n");

                    for result in self.runner.push_line(line) {
                        self.output.output(result.as_bytes());
                        self.output.output(b"\r\n");
                    }
                }
                Input::IncompleteLine(curr) => {
                    self.output.output(curr.as_bytes());
                    self.current_line.push_str(&curr);
                }
                _ => {}
            }
        }
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
