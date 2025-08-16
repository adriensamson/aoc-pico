extern crate std;

use std::pin::Pin;
use crate::aoc::AocRunner;
use aoc_pico::shell::{Command, Commands, Console, InputParser, MutexQueue, RunningCommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
pub async fn main() {
    let aoc_runner = AocRunner::new();
    let mut commands = Commands::new();
    commands.add("aoc", SpawnerCommand::new(aoc_runner));
    let queue = MutexQueue::new();
    let mut console = Console::new(InputParser::new(queue.clone()), commands);
    crossterm::terminal::enable_raw_mode().unwrap();
    std::panic::set_hook(Box::new(|_| {
        crossterm::terminal::disable_raw_mode().unwrap();
    }));
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let input_loop = async {
        let mut buffer = vec![0u8; 1024];
        loop {
            if let Ok(len) = stdin.read(buffer.as_mut()).await {
                let str = &buffer[..len];
                if str.contains(&b'\x03') {
                    break;
                }
                queue.push(str.into());
            }
        }
    };
    let output_loop = async {
        loop {
            let (buf1, buf2) = console.next_wait().await;
            stdout.write_all(&buf1[..]).await.unwrap();
            stdout.write_all(&buf2[..]).await.unwrap();
        }
    };
    tokio::select!(
        _ = input_loop => (),
        _ = output_loop => ()
    );
    crossterm::terminal::disable_raw_mode().unwrap();
}

struct SpawnerCommand<C: Command> {
    inner: C,
}

impl<C: Command> SpawnerCommand<C> {
    fn new(inner: C) -> Self {
        Self { inner }
    }
}

impl<C: Command> Command for SpawnerCommand<C> {
    fn exec(&self, args: Vec<String>, input: Vec<String>) -> Box<dyn RunningCommand> {
        let inner = &self.inner;
        let (sender, receiver) = tokio::sync::mpsc::channel(3);
        let mut running = inner.exec(args, input);
        tokio::spawn(async move {
            while let Some(s) = running.next().await {
                sender.send(s).await.unwrap();
            }
        });
        Box::new(SpawnedCommand {
            receiver
        })
    }
}

struct SpawnedCommand {
    receiver: tokio::sync::mpsc::Receiver<String>,
}

impl RunningCommand for SpawnedCommand {
    fn next(&mut self) -> Pin<Box<dyn Future<Output=Option<String>> + Send + '_>> {
        Box::pin(self.receiver.recv())
    }
}

pub fn debug_heap_size() {}

#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {println!($($tt)*)};
}
