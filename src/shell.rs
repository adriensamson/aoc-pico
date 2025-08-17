use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::future::ready;

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
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>>;
}

pub trait Command {
    fn exec(&self, args: Vec<String>, input: Vec<String>) -> Box<dyn RunningCommand>;
}

pub trait SyncCommand {
    type RunningCommand: SyncRunningCommand + 'static;
    fn exec_sync(&self, args: Vec<String>, input: Vec<String>) -> Self::RunningCommand;
}

impl<S: SyncCommand> Command for S {
    fn exec(&self, args: Vec<String>, input: Vec<String>) -> Box<dyn RunningCommand> {
        Box::new(self.exec_sync(args, input))
    }
}

pub trait SyncRunningCommand: Send {
    fn next_sync(&mut self) -> Option<String>;
}

impl SyncRunningCommand for Box<dyn SyncRunningCommand> {
    fn next_sync(&mut self) -> Option<String> {
        self.as_mut().next_sync()
    }
}

impl<S: SyncRunningCommand> RunningCommand for S {
    fn next(&mut self) -> Pin<Box<dyn Future<Output=Option<String>> + Send + '_>> {
        Box::pin(ready(self.next_sync()))
    }
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
    Poisoned,
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
}

const EOL_NONE : &[u8] = b"";
const EOL_PROMPT : &[u8] = b"\r\n$ ";
const EOL_INPUT : &[u8] = b"\r\n< ";
const EOL_RUN : &[u8] = b"\r\n> ";

const COLS : usize = 128;
const COLS_SHRINK : usize = 32;
const ROWS : usize = 256;

impl<I: AsyncInputIterator> Console<I> {
    pub async fn next_wait(&mut self) -> (Cow<'_, [u8]>, Cow<'_, [u8]>) {
        match core::mem::replace(&mut self.state, ConsoleState::Poisoned) {
            ConsoleState::RunCommand { cmd_line, input } => {
                let mut args_iter = cmd_line.trim().split(' ').map(str::trim);
                let name = args_iter.next().unwrap();
                if let Some(command) = self.commands.get(name) {
                    let args = args_iter.map(ToString::to_string).collect();
                    self.state =
                        ConsoleState::RunningCommand(command.exec(args, input));
                } else {
                    self.state = ConsoleState::Error("unknown command")
                }
                Box::pin(self.next_wait()).await
            }
            ConsoleState::RunningCommand(mut command) => {
                if let Some(line) = command.next().await {
                    self.state = ConsoleState::RunningCommand(command);
                    return (b"\r\n> ".into(), line.into_bytes().into());
                }
                self.state = ConsoleState::Prompt(String::with_capacity(COLS));
                (EOL_PROMPT.into(), EOL_NONE.into())
            }
            ConsoleState::Error(err) => {
                let res = err.to_string();
                self.state = ConsoleState::Prompt(String::with_capacity(COLS));
                (res.into_bytes().into(), EOL_PROMPT.into())
            }
            ConsoleState::ParsingInput {
                cmd_line,
                input: mut input_lines,
                mut current_line,
                ..
            } => match self.input.next_wait().await {
                Input::Line(mut s) => {
                    let start = if current_line.is_empty() {
                        if s.len() < COLS_SHRINK {
                            s.shrink_to_fit();
                        }
                        input_lines.push(s);
                        0
                    } else {
                        let start = current_line.len();
                        current_line.push_str(&s);
                        if current_line.len() < COLS_SHRINK {
                            current_line.shrink_to_fit();
                        }
                        input_lines.push(core::mem::replace(&mut current_line, String::with_capacity(COLS)));
                        start
                    };
                    self.state = ConsoleState::ParsingInput {cmd_line, input: input_lines, current_line};
                    let ConsoleState::ParsingInput {input, .. } = &self.state else { unreachable!() };
                    (input.last().unwrap().as_bytes()[start..].into(), EOL_INPUT.into())
                }
                Input::IncompleteLine(s) => {
                    let start = if current_line.is_empty() {
                        current_line = s;
                        0
                    }  else {
                        let start = current_line.len();
                        current_line.push_str(&s);
                        start
                    };
                    self.state = ConsoleState::ParsingInput {cmd_line, input: input_lines, current_line};
                    let ConsoleState::ParsingInput {current_line, .. } = &self.state else { unreachable!() };
                    (current_line.as_bytes()[start..].into(), EOL_NONE.into())
                }
                Input::Control('\x04') => {
                    input_lines.push(current_line);
                    self.state = ConsoleState::RunCommand {
                        cmd_line,
                        input: input_lines,
                    };
                    (EOL_RUN.into(), EOL_NONE.into())
                }
                _ => {
                    self.state = ConsoleState::ParsingInput {cmd_line, input: input_lines, current_line};
                    (EOL_NONE.into(), EOL_NONE.into())
                }
            },
            ConsoleState::Prompt(mut prompt) => match self.input.next_wait().await {
                Input::Line(s) => {
                    prompt.push_str(&s);
                    let eol = if let Some(prompt) = prompt.strip_suffix('<') {
                        self.state = ConsoleState::ParsingInput {
                            cmd_line: prompt.to_string(),
                            input: Vec::with_capacity(ROWS),
                            current_line: String::with_capacity(COLS),
                        };
                        EOL_INPUT
                    } else {
                        self.state = ConsoleState::RunCommand {
                            cmd_line: prompt,
                            input: Vec::new(),
                        };
                        EOL_RUN
                    };
                    (s.into_bytes().into(), eol.into())
                }
                Input::IncompleteLine(s) => {
                    prompt.push_str(&s);
                    self.state = ConsoleState::Prompt(prompt);
                    (s.into_bytes().into(), EOL_NONE.into())
                }
                _ => {
                    self.state = ConsoleState::Prompt(prompt);
                    (EOL_NONE.into(), EOL_NONE.into())
                },
            },
            ConsoleState::Poisoned => unreachable!(),
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    extern crate std;
    use alloc::rc::Rc;
    use std::collections::VecDeque;
    use std::future::poll_fn;
    use std::prelude::rust_2015::Vec;
    use std::sync::Mutex;
    use std::task::Poll;
    use super::{AsyncInputQueue, InputQueue};

    #[derive(Clone)]
    pub struct MutexQueue(Rc<Mutex<(VecDeque<Vec<u8>>, Option<std::task::Waker>)>>);
    impl MutexQueue {
        pub fn new() -> Self {
            Self(Rc::new(Mutex::new((VecDeque::new(), None))))
        }

        pub fn push(&self, input: Vec<u8>) {
            let mut inner = self.0.lock().unwrap();
            inner.0.push_back(input.into());
            inner.1.take().map(|waker| waker.wake());
        }
    }
    impl InputQueue for MutexQueue {
        fn pop(&mut self) -> Option<Vec<u8>> {
            self.0.lock().unwrap().0.pop_front()
        }
    }
    impl AsyncInputQueue for MutexQueue {
        async fn pop_wait(&mut self) -> Vec<u8> {
            poll_fn(|ctx| {
                match self.pop() {
                    Some(buf) => Poll::Ready(buf),
                    None => {
                        self.0.lock().unwrap().1 = Some(ctx.waker().clone());
                        Poll::Pending
                    },
                }
            }).await
        }
    }
}
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(all(target_os = "linux", test))]
mod tests {
    use super::*;

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
