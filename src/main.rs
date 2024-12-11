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
use alloc::vec::Vec;
use aoc_pico::shell::InputQueue;
use core::cell::RefCell;
use critical_section::Mutex;
use defmt::debug;
use rp_pico::hal::dma::single_buffer::{Config, Transfer};
use rp_pico::hal::dma::{Channel, ChannelIndex, ReadTarget, SingleChannel, WriteTarget};
use rp_pico::hal::fugit::ExtU32;
use rp_pico::hal::timer::Alarm;
use rp_pico::hal::uart::{Reader, UartDevice, ValidUartPinout, Writer};
use rp_pico::pac;

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

struct VecCapWriteTarget(Vec<u8>);

unsafe impl WriteTarget for VecCapWriteTarget {
    type TransmittedWord = u8;

    fn tx_treq() -> Option<u8> {
        None
    }

    fn tx_address_count(&mut self) -> (u32, u32) {
        let spare = self.0.spare_capacity_mut();
        (spare.as_ptr() as u32, spare.len() as u32)
    }

    fn tx_increment(&self) -> bool {
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

pub struct ConsoleDmaReader<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>, A: Alarm> {
    queue: &'static MutexInputQueue,
    reader: Option<Reader<U, P>>,
    channel: Option<Channel<D>>,
    transfer: Option<Transfer<Channel<D>, Reader<U, P>, VecCapWriteTarget>>,
    alarm: A,
    next_vec: Vec<u8>,
}

impl<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>, A: Alarm> ConsoleDmaReader<D, U, P, A> {
    const BYTES: usize = 512;

    fn new(
        queue: &'static MutexInputQueue,
        reader: Reader<U, P>,
        channel: Channel<D>,
        alarm: A,
    ) -> Self {
        Self {
            queue,
            reader: Some(reader),
            channel: Some(channel),
            transfer: None,
            alarm,
            next_vec: Vec::with_capacity(Self::BYTES),
        }
    }

    fn read_into(&mut self) -> Result<usize, ()> {
        if let Some(reader) = &mut self.reader {
            self.queue.read_into(reader)
        } else {
            Err(())
        }
    }

    fn start(&mut self) -> Result<(), ()> {
        if let (Some(reader), Some(ch)) = (self.reader.take(), self.channel.take()) {
            self.transfer = Some(
                Config::new(
                    ch,
                    reader,
                    VecCapWriteTarget(core::mem::take(&mut self.next_vec)),
                )
                .start(),
            );
            self.alarm.schedule(100.millis()).unwrap();
            self.alarm.enable_interrupt();
            self.next_vec = Vec::with_capacity(Self::BYTES);
            Ok(())
        } else {
            Err(())
        }
    }

    fn is_done(&self) -> bool {
        match &self.transfer {
            None => true,
            Some(transfer) => transfer.is_done(),
        }
    }

    fn on_finish(&mut self) {
        if !self.is_done() {
            return;
        }
        if let Some(transfer) = self.transfer.take() {
            self.alarm.disable_interrupt();
            let (ch, reader, writer) = transfer.wait();
            self.reader = Some(reader);
            self.channel = Some(ch);
            self.start().unwrap();

            let mut vec = writer.0;
            unsafe {
                vec.set_len(Self::BYTES);
            };
            self.queue.push(vec);
        }
    }

    fn check_irq1(&mut self) -> bool {
        match &mut self.transfer {
            Some(transfer) => transfer.check_irq1(),
            None => match &mut self.channel {
                None => false,
                Some(ch) => ch.check_irq1(),
            },
        }
    }

    fn on_alarm(&mut self) {
        self.alarm.disable_interrupt();
        if let Some(transfer) = self.transfer.take() {
            let trans_count = unsafe { pac::DMA::steal() }
                .ch(D::id() as usize)
                .ch_trans_count()
                .read()
                .bits();
            let (ch, reader, writer) = transfer.abort();
            let mut vec = writer.0;
            unsafe { vec.set_len(Self::BYTES - trans_count as usize) };
            vec.shrink_to_fit();
            debug!("set len from alarm : {}", vec.len());
            self.queue.push(vec);
            self.reader = Some(reader);
            self.channel = Some(ch);
        }
    }
}

pub struct MutexInputQueue(Mutex<RefCell<VecDeque<Vec<u8>>>>);

impl MutexInputQueue {
    fn new() -> Self {
        Self(Mutex::new(RefCell::new(VecDeque::with_capacity(1024))))
    }

    fn push(&self, vec: Vec<u8>) {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).push_back(vec));
    }

    fn read_into<D: UartDevice, P: ValidUartPinout<D>>(
        &self,
        uart: &Reader<D, P>,
    ) -> Result<usize, ()> {
        let mut vec = critical_section::with(|cs| {
            let mut q = self.0.borrow_ref_mut(cs);
            if q.back().is_some_and(|v| v.capacity() - v.len() >= 16) {
                q.pop_back().unwrap()
            } else {
                Vec::with_capacity(64)
            }
        });
        let cap = vec.spare_capacity_mut();
        let buf =
            unsafe { core::slice::from_raw_parts_mut(cap.as_mut_ptr() as *mut u8, cap.len()) };
        let len = uart.read_raw(buf).unwrap_or_default();
        unsafe { vec.set_len(vec.len() + len) };
        self.push(vec);
        if len > 0 {
            Ok(len)
        } else {
            Err(())
        }
    }
}

impl InputQueue for &'static MutexInputQueue {
    fn pop(&mut self) -> Option<Vec<u8>> {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).pop_front())
    }
}
