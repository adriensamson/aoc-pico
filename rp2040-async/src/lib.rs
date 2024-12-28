#![no_std]
extern crate alloc;

mod waker_slot;
mod dma;

pub use dma::DmaIrq0Listener;
pub use dma::DmaIrq1Listener;
