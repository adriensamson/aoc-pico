#![cfg_attr(not(target_os = "linux"), no_std)]
#![cfg_attr(not(target_os = "linux"), no_main)]

extern crate alloc;

pub mod aoc;

#[cfg(target_os = "none")]
mod pico;
#[cfg(target_os = "none")]
pub use pico::memory::debug_heap_size;
#[cfg(target_os = "none")]
pub use pico::debug;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{main, debug_heap_size};
