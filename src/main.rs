#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod memory;
mod multicore;

#[allow(unused_imports)]
use defmt_rtt as _;
#[allow(unused_imports)]
use panic_probe as _;

use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::vec::Vec;
use aoc_pico::shell::InputQueue;
use core::cell::RefCell;
use critical_section::Mutex;
use rp_pico::hal::dma::single_buffer::{Config, Transfer};
use rp_pico::hal::dma::{Channel, ChannelIndex, ReadTarget, SingleChannel};
use rp_pico::hal::uart::{Reader, UartDevice, ValidUartPinout, Writer};

fn read_into_vec<D: UartDevice, P: ValidUartPinout<D>>(
    uart: &Reader<D, P>,
    max_len: usize,
) -> Option<Vec<u8>> {
    let mut vec = Vec::with_capacity(max_len);
    let cap = vec.spare_capacity_mut();
    let buf = unsafe { core::slice::from_raw_parts_mut(cap.as_mut_ptr() as *mut u8, cap.len()) };
    let len = uart.read_raw(buf).ok()?;
    unsafe { vec.set_len(vec.len() + len) };
    Some(vec)
}

pub struct OutQueue(VecDeque<Vec<u8>>);

impl OutQueue {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn push(&mut self, data: Vec<u8>) -> bool {
        self.0.push_back(data);
        self.0.len() == 1
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        self.0.pop_front()
    }
}

struct VecReadTarget(Vec<u8>);

unsafe impl ReadTarget for VecReadTarget {
    type ReceivedWord = u8;

    fn rx_treq() -> Option<u8> {
        None
    }

    fn rx_address_count(&self) -> (u32, u32) {
        (self.0.as_ptr() as u32, self.0.len() as u32)
    }

    fn rx_increment(&self) -> bool {
        true
    }
}

enum ConsoleUartDmaWriter<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>> {
    Ready(Writer<U, P>, Channel<D>),
    Transferring(Transfer<Channel<D>, VecReadTarget, Writer<U, P>>),
    Poisoned,
}

impl<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>> ConsoleUartDmaWriter<D, U, P> {
    fn output(&mut self, line: Vec<u8>) {
        match core::mem::replace(self, Self::Poisoned) {
            Self::Ready(writer, ch) => {
                *self = Self::Transferring(Config::new(ch, VecReadTarget(line), writer).start())
            }
            Self::Transferring(transfer) => {
                let (ch, _, writer) = transfer.wait();
                *self = Self::Ready(writer, ch);
                self.output(line);
            }
            Self::Poisoned => unreachable!(),
        }
    }

    fn flush(&mut self) {
        match core::mem::replace(self, Self::Poisoned) {
            Self::Ready(writer, ch) => *self = Self::Ready(writer, ch),
            Self::Transferring(transfer) => {
                let (ch, _, writer) = transfer.wait();
                *self = Self::Ready(writer, ch);
            }
            Self::Poisoned => unreachable!(),
        }
    }

    fn check_irq0(&mut self) -> bool {
        match self {
            Self::Ready(_, ch) => ch.check_irq0(),
            Self::Transferring(transfer) => transfer.check_irq0(),
            Self::Poisoned => false,
        }
    }
}

#[derive(Clone)]
struct MutexInputQueue(Rc<Mutex<RefCell<VecDeque<VecDeque<u8>>>>>);

impl MutexInputQueue {
    fn new() -> Self {
        Self(Rc::new(Mutex::new(RefCell::new(VecDeque::with_capacity(
            1024,
        )))))
    }

    fn push(&self, data: Vec<u8>) {
        critical_section::with(|cs| {
            self.0.borrow_ref_mut(cs).push_back(data.into());
        })
    }
}

impl InputQueue for MutexInputQueue {
    fn pop_byte(&mut self) -> Option<u8> {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).pop_byte())
    }
}
