#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod dma;
mod memory;
mod multicore;

#[allow(unused_imports)]
use defmt_rtt as _;
#[allow(unused_imports)]
use panic_probe as _;

use crate::dma::DoubleChannelReader;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use aoc_pico::shell::{AsyncInputQueue, InputQueue};
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;
use defmt::debug;
use rp_pico::hal::dma::single_buffer::{Config, Transfer};
use rp_pico::hal::dma::{Channel, ChannelIndex, ReadTarget, SingleChannel};
use rp_pico::hal::timer::Alarm;
use rp_pico::hal::uart::{Reader, UartDevice, ValidUartPinout, Writer};

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

pub struct ConsoleDmaReader<
    CH1: ChannelIndex,
    CH2: ChannelIndex,
    A: Alarm,
    U: UartDevice,
    P: ValidUartPinout<U>,
> {
    queue: &'static MutexInputQueue,
    dma: DoubleChannelReader<CH1, CH2, A, Reader<U, P>, 512>,
}

impl<CH1: ChannelIndex, CH2: ChannelIndex, A: Alarm, U: UartDevice, P: ValidUartPinout<U>>
    ConsoleDmaReader<CH1, CH2, A, U, P>
{
    fn new(
        queue: &'static MutexInputQueue,
        dma: DoubleChannelReader<CH1, CH2, A, Reader<U, P>, 512>,
    ) -> Self {
        Self { queue, dma }
    }

    fn read_into(&mut self) -> Result<usize, ()> {
        let reader = self.dma.reader().ok_or(())?;
        self.queue.read_into(reader)
    }

    fn start(&mut self) -> Result<(), ()> {
        self.dma.start()
    }

    fn on_dma_irq(&mut self) -> Result<(), ()> {
        let vec = self.dma.on_dma_irq()?;
        self.queue.push(vec);
        Ok(())
    }

    fn on_alarm(&mut self) -> Result<(), ()> {
        let vec = self.dma.on_alarm_irq()?;
        self.queue.push(vec);
        Ok(())
    }
}

pub struct MutexInputQueue(Mutex<RefCell<(VecDeque<Vec<u8>>, Option<Waker>)>>);

impl MutexInputQueue {
    fn new() -> Self {
        Self(Mutex::new(RefCell::new((VecDeque::with_capacity(1024), None))))
    }

    fn push(&self, vec: Vec<u8>) {
        critical_section::with(|cs| {
            let (vd, w) = &mut *self.0.borrow_ref_mut(cs);
            vd.push_back(vec);
            debug!("wake");
            w.take().map(Waker::wake);
        });
    }

    fn read_into<D: UartDevice, P: ValidUartPinout<D>>(
        &self,
        uart: &Reader<D, P>,
    ) -> Result<usize, ()> {
        let mut vec = critical_section::with(|cs| {
            let q = &mut self.0.borrow_ref_mut(cs).0;
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

impl<'a> InputQueue for &'a MutexInputQueue {
    fn pop(&mut self) -> Option<Vec<u8>> {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).0.pop_front())
    }
}

impl<'a> AsyncInputQueue for &'a MutexInputQueue {
    async fn pop_wait(&mut self) -> Vec<u8> {
        MutexInputQueueWaiter(self).await
    }
}

struct MutexInputQueueWaiter<'a>(&'a MutexInputQueue);

impl Future for MutexInputQueueWaiter<'_> {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(v) = critical_section::with(|cs| self.0.0.borrow_ref_mut(cs).0.pop_front()) {
            return Poll::Ready(v);
        }
        critical_section::with(|cs| {
            self.0.0.borrow_ref_mut(cs).1.replace(cx.waker().clone());
        });
        Poll::Pending
    }
}
