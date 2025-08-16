extern crate std;

use crate::aoc::AocRunner;
use aoc_pico::shell::{Commands, Console, InputParser, MutexQueue};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    let aoc_runner = AocRunner::new();
    let mut commands = Commands::new();
    commands.add("aoc", aoc_runner);
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
                queue.push(buffer[..len].into());
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
    tokio::join!(input_loop, output_loop);
}

pub fn debug_heap_size() {}

#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {println!($($tt)*)};
}
