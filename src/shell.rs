use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

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
    fn pop_byte(&mut self) -> Option<u8>;
}

#[derive(Default)]
pub struct InputParser<Q: InputQueue> {
    queue: Q,
    state: State,
}

impl<Q: InputQueue> InputParser<Q> {
    pub fn new(queue: Q) -> Self {
        Self {
            queue,
            state: State::default(),
        }
    }
}

impl InputQueue for VecDeque<VecDeque<u8>> {
    fn pop_byte(&mut self) -> Option<u8> {
        while let Some(buf) = self.front_mut() {
            if let Some(b) = buf.pop_front() {
                return Some(b);
            } else {
                self.pop_front();
            }
        }
        None
    }
}

impl<Q: InputQueue> Iterator for InputParser<Q> {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_line = String::with_capacity(64);
        loop {
            let b = match self.queue.pop_byte() {
                Some(b) => b,
                None => {
                    //debug!("no more bytes");
                    return if current_line.is_empty() {
                        None
                    } else {
                        Some(Input::IncompleteLine(current_line))
                    };
                }
            };
            //debug!("state={:?} b={:X}", state, b);
            match &mut self.state {
                State::Normal => {
                    match b {
                        b'\n' | b'\r' => {
                            return Some(Input::Line(current_line));
                        }
                        b'\x1b' => self.state = State::InEscape(Vec::from([b])),
                        b'\x00'..=b'\x1f' | b'\x7f' => {
                            //debug!("control: {:X}", b);
                            return Some(Input::Control(b as char));
                        }
                        b'\x20'..=b'\x7e' => {
                            current_line.push(b as char);
                            self.state = State::Normal;
                        }
                        b'\x80'..=b'\xff' => {
                            self.state = State::InUtf8(Vec::from([b]));
                        }
                    }
                }
                State::InUtf8(v) => {
                    v.push(b);
                    if !matches!(b, b'\x80'..=b'\xff') {
                        // invalid sequence
                        let res = Some(Input::InvalidByteSequence(core::mem::take(v)));
                        self.state = State::Normal;
                        return res;
                    } else if let Ok(s) = core::str::from_utf8(&v) {
                        current_line.push_str(s);
                        self.state = State::Normal;
                    } else {
                        self.state = State::InUtf8(core::mem::take(v));
                    }
                }
                State::InEscape(v) => {
                    v.push(b);
                    if v.len() == 2 {
                        if b == b'[' {
                            //debug!("CSI");
                            //self.state = State::InEscape(v);
                        } else {
                            //debug!("1byte escape");
                            let res = Some(Input::EscapeSequence(EscapeSequence::from(
                                core::mem::take(v),
                            )));
                            self.state = State::Normal;
                            return res;
                        }
                    } else if matches!(b, b'\x40'..=b'\x7e') {
                        //debug!("end of sequence");
                        // end of sequence
                        let res = Some(Input::EscapeSequence(EscapeSequence::from(
                            core::mem::take(v),
                        )));
                        self.state = State::Normal;
                        return res;
                    } else {
                        //debug!("continue");
                        //self.state = State::InEscape(v);
                    }
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

pub trait Command {
    type Output: Iterator<Item = String> + Send;

    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Self::Output;
}

trait DynCommand: Send {
    fn exec(
        &mut self,
        args: Vec<String>,
        input: Vec<String>,
    ) -> Box<dyn Iterator<Item = String> + Send + 'static>;
}

impl<C: Command + Send> DynCommand for C
where
    C::Output: 'static,
{
    fn exec(
        &mut self,
        args: Vec<String>,
        input: Vec<String>,
    ) -> Box<dyn Iterator<Item = String> + Send + 'static> {
        Box::new(self.exec(args, input))
    }
}

#[derive(Default)]
pub struct Commands {
    names: Vec<&'static str>,
    commands: Vec<Box<dyn DynCommand>>,
}

impl Commands {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, name: &'static str, command: impl Command + Send + 'static) {
        self.names.push(name);
        self.commands.push(Box::new(command));
    }

    fn get(&mut self, name: &str) -> Option<&mut Box<dyn DynCommand>> {
        let idx = self
            .names
            .iter()
            .enumerate()
            .find_map(|(i, &n)| (n == name).then_some(i))?;
        Some(&mut self.commands[idx])
    }
}

pub struct Console<I: Iterator<Item = Input>> {
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
    RunningCommand(Box<dyn Iterator<Item = String> + Send>),
    Error(&'static str),
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self::Prompt(String::new())
    }
}

impl<I: Iterator<Item = Input>> Console<I> {
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

impl<I: Iterator<Item = Input>> Iterator for Console<I> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            ConsoleState::RunCommand { cmd_line, input } => {
                let mut args_iter = cmd_line.trim().split(' ').map(str::trim);
                let name = args_iter.next().unwrap();
                if let Some(command) = self.commands.get(name) {
                    let args = args_iter.map(ToString::to_string).collect();
                    self.state = ConsoleState::RunningCommand(Box::new(
                        command
                            .exec(args, core::mem::take(input))
                            .map(|s| String::from("\r\n> ") + &s),
                    ))
                } else {
                    self.state = ConsoleState::Error("unknown command")
                }
                self.next()
            }
            ConsoleState::RunningCommand(command) => {
                if let Some(line) = command.next() {
                    return Some(line.into_bytes());
                }
                self.state = ConsoleState::Prompt(String::new());
                Some(self.eol().as_bytes().to_vec())
            }
            ConsoleState::Error(err) => {
                let mut res = err.to_string();
                self.state = ConsoleState::Prompt(String::new());
                res += self.eol();
                Some(res.into_bytes())
            }
            ConsoleState::ParsingInput {
                cmd_line,
                input: input_lines,
                current_line,
                ..
            } => match self.input.next()? {
                Input::Line(s) => {
                    current_line.push_str(&s);
                    input_lines.push(core::mem::take(current_line));
                    Some((s + self.eol()).into_bytes())
                }
                Input::IncompleteLine(s) => {
                    current_line.push_str(&s);
                    Some(s.into_bytes())
                }
                Input::Control('\x04') => {
                    input_lines.push(core::mem::take(current_line));
                    self.state = ConsoleState::RunCommand {
                        cmd_line: core::mem::take(cmd_line),
                        input: core::mem::take(input_lines),
                    };
                    Some(self.eol().as_bytes().to_vec())
                }
                _ => Some(Vec::new()),
            },
            ConsoleState::Prompt(prompt) => match self.input.next()? {
                Input::Line(s) => {
                    prompt.push_str(&s);
                    if let Some(prompt) = prompt.strip_suffix('<') {
                        self.state = ConsoleState::ParsingInput {
                            cmd_line: prompt.to_string(),
                            input: Vec::new(),
                            current_line: String::new(),
                        };
                    } else {
                        self.state = ConsoleState::RunCommand {
                            cmd_line: core::mem::take(prompt),
                            input: Vec::new(),
                        };
                    }
                    Some((s + self.eol()).into_bytes())
                }
                Input::IncompleteLine(s) => {
                    prompt.push_str(&s);
                    Some(s.into_bytes())
                }
                _ => Some(Vec::new()),
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
    struct MutexQueue(Rc<Mutex<VecDeque<VecDeque<u8>>>>);
    impl MutexQueue {
        fn new() -> Self {
            Self(Rc::new(Mutex::new(VecDeque::new())))
        }

        fn push(&self, input: Vec<u8>) {
            self.0.lock().unwrap().push_back(input.into());
        }
    }
    impl InputQueue for MutexQueue {
        fn pop_byte(&mut self) -> Option<u8> {
            self.0.lock().unwrap().pop_byte()
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
