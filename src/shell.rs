use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
//use defmt::{debug, Formatter};
use rp_pico::hal::uart::{UartDevice, ValidUartPinout, Writer};

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
                    //debug!("no more bytes");
                    self.buffer.extend(state.as_bytes());
                    return if current_line.is_empty() { None } else { Some(Input::IncompleteLine(current_line)) }
                },
            };
            //debug!("state={:?} b={:X}", state, b);
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
                            //debug!("control: {:X}", b);
                            return Some(Input::Control(b as char))
                        },
                        b'\x20'..=b'\x7e' => {
                            current_line.push(b as char);
                            state = State::Normal;
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
                            //debug!("CSI");
                            state = State::InEscape(v);
                        } else {
                            //debug!("1byte escape");
                            return Some(Input::EscapeSequence(EscapeSequence::from(v)));
                        }
                    } else if matches!(b, b'\x40'..=b'\x7e') {
                        //debug!("end of sequence");
                        // end of sequence
                        return Some(Input::EscapeSequence(EscapeSequence::from(v)));
                    } else {
                        //debug!("continue");
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
    type Output : Iterator<Item = String> + Send;

    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Self::Output;
}

trait DynCommand: Send {
    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Box<dyn Iterator<Item = String> + Send + 'static>;
}

impl<C: Command + Send> DynCommand for C
where C::Output: 'static
{
    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Box<dyn Iterator<Item = String> + Send + 'static> {
        Box::new(self.exec(args, input))
    }
}

#[derive(Default)]
pub struct Commands {
    names: Vec<&'static str>,
    commands: Vec<Box<dyn DynCommand>>
}

impl Commands {
    pub fn new() -> Self { Default::default() }

    pub fn add(&mut self, name: &'static str, command: impl Command + Send + 'static) {
        self.names.push(name);
        self.commands.push(Box::new(command));
    }

    fn get(&mut self, name: &str) -> Option<&mut Box<dyn DynCommand>> {
        let idx = self.names.iter().enumerate().find_map(|(i, &n)| (n == name).then_some(i))?;
        Some(&mut self.commands[idx])
    }
}

pub struct Console {
    parser: InputParser,
    commands: Commands,
    state: ConsoleState,
}

pub trait ConsoleOutput {
    fn output(&mut self, line: &[u8]);
}

enum ConsoleState {
    Prompt(String),
    ParsingInput {cmd_line: String, input: Vec<String>, current_line: String},
    RunCommand {cmd_line: String, input: Vec<String> },
    RunningCommand(Box<dyn Iterator<Item = String> + Send>),
    Error(&'static str),
}

impl Default for ConsoleState {
    fn default() -> Self { Self::Prompt(String::new()) }
}

impl Console {
    pub fn new(commands: Commands) -> Self {
        Self {
            parser: InputParser::new(),
            commands,
            state: Default::default(),
        }
    }

    pub fn push(&mut self, buf: &[u8]) {
        self.parser.push(buf);
    }

    fn eol(&self) -> &str {
        match &self.state {
            ConsoleState::Prompt(_) => "\r\n$ ",
            ConsoleState::ParsingInput {..} => "\r\n< ",
            ConsoleState::RunCommand {..} | ConsoleState::RunningCommand(_) => "\r\n> ",
            ConsoleState::Error(_) => "\r\n! ",
        }
    }
}

impl Iterator for Console {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            ConsoleState::RunCommand {cmd_line, input} => {
                let mut args_iter = cmd_line.trim().split(' ').map(str::trim);
                let name = args_iter.next().unwrap();
                if let Some(command) = self.commands.get(name) {
                    let args = args_iter.map(ToString::to_string).collect();
                    self.state = ConsoleState::RunningCommand(Box::new(command.exec(args, core::mem::take(input)).map(|s| String::from("\r\n> ") + &s)))
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
            },
            ConsoleState::Error(err) => {
                let mut res = err.to_string();
                self.state = ConsoleState::Prompt(String::new());
                res += self.eol();
                Some(res.into_bytes())
            },
            ConsoleState::ParsingInput {cmd_line, input: input_lines, current_line, .. } => {
                match self.parser.next()? {
                    Input::Line(s) => {
                        current_line.push_str(&s);
                        input_lines.push(core::mem::take(current_line));
                        Some((s + self.eol()).into_bytes())
                    },
                    Input::IncompleteLine(s) => {
                        current_line.push_str(&s);
                        Some(s.into_bytes())
                    },
                    Input::Control('\x03') => {
                        input_lines.push(core::mem::take(current_line));
                        self.state = ConsoleState::RunCommand {cmd_line: core::mem::take(cmd_line), input: core::mem::take(input_lines)};
                        Some(self.eol().as_bytes().to_vec())
                    },
                    _ => Some(Vec::new())
                }
            },
            ConsoleState::Prompt(prompt) => {
                match self.parser.next()? {
                    Input::Line(s) => {
                        prompt.push_str(&s);
                        if let Some(prompt) = prompt.strip_suffix('<') {
                            self.state = ConsoleState::ParsingInput {cmd_line: prompt.to_string(), input: Vec::new(), current_line: String::new()};
                        } else {
                            self.state = ConsoleState::RunCommand { cmd_line: core::mem::take(prompt), input: Vec::new() };
                        }
                        Some((s + self.eol()).into_bytes())
                    },
                    Input::IncompleteLine(s) => {
                        prompt.push_str(&s);
                        Some(s.into_bytes())
                    }
                    _ => Some(Vec::new()),
                }
            }
        }
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

#[test]
fn test_input_parser() {
    let mut parser = InputParser::new();
    parser.push(b"abc");
    assert_eq!(parser.next(), Some(Input::IncompleteLine("abc".into())));
    assert_eq!(parser.next(), None);
    parser.push(b"\x1b");
    assert_eq!(parser.next(), None);
    parser.push(b"[m");
    assert_eq!(parser.next(), Some(Input::EscapeSequence(EscapeSequence::Unknown(b"\x1b[m".into()))));
    assert_eq!(parser.next(), None);
    parser.push(b"pppp\r");
    assert_eq!(parser.next(), Some(Input::Line("pppp".into())));
    assert_eq!(parser.next(), None);
    parser.push(b"\x03");
    assert_eq!(parser.next(), Some(Input::Control('\x03')));
    assert_eq!(parser.next(), None);
}

#[test]
fn test_console() {
    let mut console = Console::new(Commands::default());
    console.push(b"\r");
    assert_eq!(console.next(), Some(b"\r\n> ".into()));
    assert_eq!(console.next(), Some(b"unknown command\r\n$ ".into()));
    assert_eq!(console.next(), None);
    console.push(b"abc\r");
    assert_eq!(console.next(), Some(b"abc\r\n> ".into()));
    assert_eq!(console.next(), Some(b"unknown command\r\n$ ".into()));
    assert_eq!(console.next(), None);
    console.push(b"abc <\r");
    assert_eq!(console.next(), Some(b"abc <\r\n< ".into()));
    assert_eq!(console.next(), None);
    console.push(b"plop\r");
    assert_eq!(console.next(), Some(b"plop\r\n< ".into()));
    assert_eq!(console.next(), None);
    console.push(b"\x03");
    assert_eq!(console.next(), Some(b"\r\n> ".into()));
    assert_eq!(console.next(), Some(b"unknown command\r\n$ ".into()));
    assert_eq!(console.next(), None);
}
