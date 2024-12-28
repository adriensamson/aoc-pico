#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod dma;
mod memory;
mod multicore;

use alloc::boxed::Box;
#[allow(unused_imports)]
use defmt_rtt as _;
#[allow(unused_imports)]
use panic_probe as _;

use crate::dma::DoubleChannelReader;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use aoc_pico::shell::{AsyncInputQueue, InputQueue};
use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::pin::{Pin};
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;
use defmt::debug;
use rp2040_async::{DmaIrq1Handler, TimerIrq0Handler, UartIrqHandler};
use rp_pico::hal::dma::single_buffer::{Config, Transfer};
use rp_pico::hal::dma::{Channel, ChannelIndex, ReadTarget};
use rp_pico::hal::timer::Alarm;
use rp_pico::hal::uart::{Reader, UartDevice, ValidUartPinout, Writer};

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

    async fn run(&mut self, uart0irq_handler: &'static UartIrqHandler<U, P>, timer_irq0handler: &'static TimerIrq0Handler, dma_irq1handler: &'static DmaIrq1Handler) -> ! {
        loop {
            let mut vec = Vec::with_capacity(32);
            let cap = vec.spare_capacity_mut();
            let buf = unsafe { core::slice::from_raw_parts_mut(cap.as_mut_ptr().cast(), cap.len()) };
            let len = uart0irq_handler.wait_rx(self.dma.reader().unwrap(), buf).await;
            unsafe { vec.set_len(len) };
            self.queue.push(vec);
            if len >= 16 {
                'dma: loop {
                    self.dma.start().unwrap();
                    let dma_wait = first_future(dma_irq1handler.wait_done(CH1::id() as usize), dma_irq1handler.wait_done(CH2::id() as usize));
                    let alarm_wait = timer_irq0handler.wait_alarm();
                    match first_until(dma_wait, alarm_wait).await {
                        Ok(_) => {
                            let vec = self.dma.on_dma_irq().unwrap();
                            self.queue.push(vec);
                        },
                        Err(_) => {
                            let vec = self.dma.on_alarm_irq().unwrap();
                            self.queue.push(vec);
                            break 'dma;
                        }
                    }
                }
            }
        }
    }
}

fn first_future<T>(f1: impl Future<Output=T> + 'static, f2: impl Future<Output=T> + 'static) -> impl Future<Output = T> {
    let (mut p1, mut p2) = (Box::pin(f1), Box::pin(f2));
    poll_fn(move |cx| {
        if let Poll::Ready(t) = p1.as_mut().poll(cx) {
            Poll::Ready(t)
        } else if let Poll::Ready(t) = p2.as_mut().poll(cx) {
            Poll::Ready(t)
        } else {
            Poll::Pending
        }
    })
}

fn first_until<T, U>(f1: impl Future<Output=T>, f2: impl Future<Output=U>) -> impl Future<Output = Result<T, U>> {
    let (mut p1, mut p2) = (Box::pin(f1), Box::pin(f2));
    poll_fn(move |cx| {
        if let Poll::Ready(t) = p1.as_mut().poll(cx) {
            Poll::Ready(Ok(t))
        } else if let Poll::Ready(t) = p2.as_mut().poll(cx) {
            Poll::Ready(Err(t))
        } else {
            Poll::Pending
        }
    })
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
        if let Some(v) = critical_section::with(|cs| self.0 .0.borrow_ref_mut(cs).0.pop_front()) {
            return Poll::Ready(v);
        }
        critical_section::with(|cs| {
            self.0 .0.borrow_ref_mut(cs).1.replace(cx.waker().clone());
        });
        Poll::Pending
    }
}
