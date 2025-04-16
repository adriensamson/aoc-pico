use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
pub enum Input {
    Line(String),
    IncompleteLine(String),
    Control(char),
    EscapeSequence(EscapeSequence),
    InvalidByteSequence(Vec<u8>),
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
pub enum EscapeSequence {
    Unknown(Vec<u8>),
}

impl From<Vec<u8>> for EscapeSequence {
    fn from(value: Vec<u8>) -> Self {
        //debug!("escape sequence: {=[u8]:X}", value);
        EscapeSequence::Unknown(value)
    }
}

pub trait InputQueue {
    fn pop(&mut self) -> Option<Vec<u8>>;
}

impl InputQueue for VecDeque<Vec<u8>> {
    fn pop(&mut self) -> Option<Vec<u8>> {
        self.pop_front()
    }
}

#[allow(async_fn_in_trait)]
pub trait AsyncInputQueue: InputQueue {
    async fn pop_wait(&mut self) -> Vec<u8>;
}

#[derive(Default)]
pub struct InputParser<Q: InputQueue> {
    queue: Q,
    current: VecDeque<u8>,
}

pub trait AsyncInputIterator {
    #[allow(async_fn_in_trait)]
    async fn next_wait(&mut self) -> Input;
}

impl<Q: InputQueue> InputParser<Q> {
    pub fn new(queue: Q) -> Self {
        Self {
            queue,
            current: VecDeque::new(),
        }
    }

    fn pop_byte(&mut self) -> Option<u8> {
        if let Some(byte) = self.current.pop_front() {
            return Some(byte);
        }
        self.current = self.queue.pop()?.into();
        self.pop_byte()
    }
}

impl<Q: AsyncInputQueue> InputParser<Q> {
    async fn pop_byte_wait(&mut self) -> u8 {
        if let Some(byte) = self.pop_byte() {
            return byte;
        }
        self.current = self.queue.pop_wait().await.into();
        Box::pin(self.pop_byte_wait()).await
    }
}

impl<Q: AsyncInputQueue> AsyncInputIterator for InputParser<Q> {
    async fn next_wait(&mut self) -> Input {
        let mut acc = ParserAccumulator::new();
        let mut b = self.pop_byte_wait().await;
        loop {
            match acc.advance(b) {
                Ok(input) => {
                    return input;
                }
                Err(acc2) => {
                    acc = acc2;
                }
            }
            match self.pop_byte() {
                Some(byte) => b = byte,
                None => {
                    self.current = acc.state.into_bytes().into();
                    return Input::IncompleteLine(acc.current_line);
                }
            }
        }
    }
}

struct ParserAccumulator {
    state: State,
    current_line: String,
}

impl ParserAccumulator {
    fn with(current_line: String, state: State) -> Self {
        Self {
            state,
            current_line,
        }
    }

    fn new() -> Self {
        Self {
            state: State::Normal,
            current_line: String::with_capacity(64),
        }
    }

    fn advance(self, b: u8) -> Result<Input, Self> {
        let Self {
            state,
            mut current_line,
        } = self;
        match state {
            State::Normal => {
                match b {
                    b'\n' | b'\r' => Ok(Input::Line(current_line)),
                    b'\x1b' => Err(Self::with(current_line, State::InEscape(Vec::from([b])))),
                    b'\x00'..=b'\x1f' | b'\x7f' => {
                        //debug!("control: {:X}", b);
                        Ok(Input::Control(b as char))
                    }
                    b'\x20'..=b'\x7e' => {
                        current_line.push(b as char);
                        Err(Self::with(current_line, State::Normal))
                    }
                    b'\x80'..=b'\xff' => {
                        Err(Self::with(current_line, State::InUtf8(Vec::from([b]))))
                    }
                }
            }
            State::InUtf8(mut v) => {
                v.push(b);
                if !matches!(b, b'\x80'..=b'\xff') {
                    // invalid sequence
                    Ok(Input::InvalidByteSequence(v))
                } else if let Ok(s) = core::str::from_utf8(&v) {
                    current_line.push_str(s);
                    Err(Self::with(current_line, State::Normal))
                } else {
                    Err(Self::with(current_line, State::InUtf8(v)))
                }
            }
            State::InEscape(mut v) => {
                v.push(b);
                if v.len() == 2 {
                    if b == b'[' {
                        //debug!("CSI");
                        Err(Self::with(current_line, State::InEscape(v)))
                    } else {
                        //debug!("1byte escape");
                        Ok(Input::EscapeSequence(EscapeSequence::from(v)))
                    }
                } else if matches!(b, b'\x40'..=b'\x7e') {
                    //debug!("end of sequence");
                    // end of sequence
                    Ok(Input::EscapeSequence(EscapeSequence::from(v)))
                } else {
                    //debug!("continue");
                    Err(Self::with(current_line, State::InEscape(v)))
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Default)]
enum State {
    #[default]
    Normal,
    InUtf8(Vec<u8>),
    InEscape(Vec<u8>),
}

impl State {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            State::Normal => Vec::new(),
            State::InUtf8(v) => v,
            State::InEscape(v) => v,
        }
    }
}

/*impl defmt::Format for State {
    fn format(&self, fmt: Formatter) {
        match self {
            State::Normal =>  defmt::write!(fmt, "normal"),
            State::InUtf8(v) =>  defmt::write!(fmt, "inutf8: {=[u8]:X}", v),
            State::InEscape(v) =>  defmt::write!(fmt, "inescape: {=[u8]:X}", v),
        }
    }
}*/

pub trait RunningCommand: Send {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<String>>>>;
}

pub trait Command {
    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Box<dyn RunningCommand>;
}

#[derive(Default)]
pub struct Commands {
    names: Vec<&'static str>,
    commands: Vec<Box<dyn Command>>,
}

impl Commands {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, name: &'static str, command: impl Command + Send + 'static) {
        self.names.push(name);
        self.commands.push(Box::new(command));
    }

    fn get(&mut self, name: &str) -> Option<&mut Box<dyn Command>> {
        let idx = self
            .names
            .iter()
            .enumerate()
            .find_map(|(i, &n)| (n == name).then_some(i))?;
        Some(&mut self.commands[idx])
    }
}

pub struct Console<I> {
    input: I,
    commands: Commands,
    state: ConsoleState,
}

enum ConsoleState {
    Prompt(String),
    ParsingInput {
        cmd_line: String,
        input: Vec<String>,
        current_line: String,
    },
    RunCommand {
        cmd_line: String,
        input: Vec<String>,
    },
    RunningCommand(Box<dyn RunningCommand>),
    Error(&'static str),
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self::Prompt(String::with_capacity(COLS))
    }
}

impl<I> Console<I> {
    pub fn new(input: I, commands: Commands) -> Self {
        Self {
            input,
            commands,
            state: Default::default(),
        }
    }

    fn eol(&self) -> &str {
        match &self.state {
            ConsoleState::Prompt(_) => "\r\n$ ",
            ConsoleState::ParsingInput { .. } => "\r\n< ",
            ConsoleState::RunCommand { .. } | ConsoleState::RunningCommand(_) => "\r\n> ",
            ConsoleState::Error(_) => "\r\n! ",
        }
    }
}

const COLS : usize = 128;
const ROWS : usize = 256;

impl<I: AsyncInputIterator> Console<I> {
    pub async fn next_wait(&mut self) -> Vec<u8> {
        match &mut self.state {
            ConsoleState::RunCommand { cmd_line, input } => {
                let mut args_iter = cmd_line.trim().split(' ').map(str::trim);
                let name = args_iter.next().unwrap();
                if let Some(command) = self.commands.get(name) {
                    let args = args_iter.map(ToString::to_string).collect();
                    self.state =
                        ConsoleState::RunningCommand(command.exec(args, core::mem::take(input)));
                } else {
                    self.state = ConsoleState::Error("unknown command")
                }
                Box::pin(self.next_wait()).await
            }
            ConsoleState::RunningCommand(command) => {
                if let Some(line) = command.next().await {
                    return (String::from("\r\n> ") + &line).into_bytes();
                }
                self.state = ConsoleState::Prompt(String::with_capacity(COLS));
                self.eol().as_bytes().to_vec()
            }
            ConsoleState::Error(err) => {
                let mut res = err.to_string();
                self.state = ConsoleState::Prompt(String::with_capacity(COLS));
                res += self.eol();
                res.into_bytes()
            }
            ConsoleState::ParsingInput {
                cmd_line,
                input: input_lines,
                current_line,
                ..
            } => match self.input.next_wait().await {
                Input::Line(s) => {
                    current_line.push_str(&s);
                    input_lines.push(core::mem::replace(current_line, String::with_capacity(COLS)));
                    (s + self.eol()).into_bytes()
                }
                Input::IncompleteLine(s) => {
                    current_line.push_str(&s);
                    s.into_bytes()
                }
                Input::Control('\x04') => {
                    input_lines.push(core::mem::replace(current_line, String::with_capacity(COLS)));
                    self.state = ConsoleState::RunCommand {
                        cmd_line: core::mem::take(cmd_line),
                        input: core::mem::take(input_lines),
                    };
                    self.eol().as_bytes().to_vec()
                }
                _ => Vec::new(),
            },
            ConsoleState::Prompt(prompt) => match self.input.next_wait().await {
                Input::Line(s) => {
                    prompt.push_str(&s);
                    if let Some(prompt) = prompt.strip_suffix('<') {
                        self.state = ConsoleState::ParsingInput {
                            cmd_line: prompt.to_string(),
                            input: Vec::with_capacity(ROWS),
                            current_line: String::with_capacity(COLS),
                        };
                    } else {
                        self.state = ConsoleState::RunCommand {
                            cmd_line: core::mem::take(prompt),
                            input: Vec::new(),
                        };
                    }
                    (s + self.eol()).into_bytes()
                }
                Input::IncompleteLine(s) => {
                    prompt.push_str(&s);
                    s.into_bytes()
                }
                _ => Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use std::collections::VecDeque;
    use std::prelude::rust_2015::Vec;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct MutexQueue(Rc<Mutex<VecDeque<Vec<u8>>>>);
    impl MutexQueue {
        fn new() -> Self {
            Self(Rc::new(Mutex::new(VecDeque::new())))
        }

        fn push(&self, input: Vec<u8>) {
            self.0.lock().unwrap().push_back(input.into());
        }
    }
    impl InputQueue for MutexQueue {
        fn pop(&mut self) -> Option<Vec<u8>> {
            self.0.lock().unwrap().pop_front()
        }
    }

    #[test]
    fn test_input_parser() {
        let queue = MutexQueue::new();
        let mut parser = InputParser::new(queue.clone());
        queue.push(b"abc".to_vec());
        assert_eq!(parser.next(), Some(Input::IncompleteLine("abc".into())));
        assert_eq!(parser.next(), None);
        queue.push(b"\x1b".to_vec());
        assert_eq!(parser.next(), None);
        queue.push(b"[m".to_vec());
        assert_eq!(
            parser.next(),
            Some(Input::EscapeSequence(EscapeSequence::Unknown(
                b"\x1b[m".into()
            )))
        );
        assert_eq!(parser.next(), None);
        queue.push(b"pppp\r".to_vec());
        assert_eq!(parser.next(), Some(Input::Line("pppp".into())));
        assert_eq!(parser.next(), None);
        queue.push(b"\x04".to_vec());
        assert_eq!(parser.next(), Some(Input::Control('\x04')));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_console() {
        let queue = MutexQueue::new();

        let mut console = Console::new(InputParser::new(queue.clone()), Commands::default());
        queue.push(b"\r".to_vec());
        assert_eq!(console.next(), Some(b"\r\n> ".into()));
        assert_eq!(console.next(), Some(b"unknown command\r\n$ ".into()));
        assert_eq!(console.next(), None);
        queue.push(b"abc\r".to_vec());
        assert_eq!(console.next(), Some(b"abc\r\n> ".into()));
        assert_eq!(console.next(), Some(b"unknown command\r\n$ ".into()));
        assert_eq!(console.next(), None);
        queue.push(b"abc <\r".to_vec());
        assert_eq!(console.next(), Some(b"abc <\r\n< ".into()));
        assert_eq!(console.next(), None);
        queue.push(b"plop\r".to_vec());
        assert_eq!(console.next(), Some(b"plop\r\n< ".into()));
        assert_eq!(console.next(), None);
        queue.push(b"\x04".to_vec());
        assert_eq!(console.next(), Some(b"\r\n> ".into()));
        assert_eq!(console.next(), Some(b"unknown command\r\n$ ".into()));
        assert_eq!(console.next(), None);
    }
}
