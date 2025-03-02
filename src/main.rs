#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod dma;
mod memory;
mod multicore;

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[allow(unused_imports)]
use defmt_rtt as _;
#[allow(unused_imports)]
use panic_probe as _;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use aoc_pico::shell::{AsyncInputQueue, Console, InputParser, InputQueue};
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;
use defmt::debug;
use rp2040_async::dma::{AsyncTransfer, WaitDone};
use rp2040_hal::dma::single_buffer::Config;
use rp2040_hal::dma::{Channel, ChannelIndex, ReadTarget};
use rp2040_hal::uart::{UartDevice, ValidUartPinout, Writer};

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

pub struct MutexInputQueue(Mutex<RefCell<(VecDeque<Vec<u8>>, Option<Waker>)>>);

impl MutexInputQueue {
    fn new() -> Self {
        Self(Mutex::new(RefCell::new((
            VecDeque::with_capacity(1024),
            None,
        ))))
    }

    fn push(&self, vec: Vec<u8>) {
        critical_section::with(|cs| {
            let (vd, w) = &mut *self.0.borrow_ref_mut(cs);
            vd.push_back(vec);
            debug!("wake");
            if let Some(w) = w.take() {
                w.wake();
            }
        });
    }
}

impl InputQueue for &'_ MutexInputQueue {
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

async fn run_console<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>>(
    mut console: Console<InputParser<&'static MutexInputQueue>>,
    mut writer: Writer<U, P>,
    mut ch: Channel<D>,
) {
    loop {
        let out = console.next_wait().await;
        let mut transfer = AsyncTransfer::new_single_buffer_irq0(
            Config::new(ch, VecReadTarget(out), writer).start(),
        );
        transfer.wait_done().await;
        (ch, _, writer) = transfer.into_inner().wait();
    }
}
