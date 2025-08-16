#![no_std]
extern crate alloc;

#[cfg(all(target_os = "linux", test))]
extern crate std;

pub mod shell;
