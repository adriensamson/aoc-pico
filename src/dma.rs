use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::{Future, poll_fn};
use core::ptr::write_volatile;
use core::task::Poll;
use defmt::debug;
use embedded_hal_async::delay::DelayNs;
use embedded_io_async::Read;
use rp2040_async::dma::{AsyncTransfer, WaitDone};
use rp2040_async::uart::AsyncReader;
use rp2040_hal::dma::single_buffer::{Config, Transfer};
use rp2040_hal::dma::{
    Channel, ChannelIndex, EndlessReadTarget, ReadTarget, SingleChannel, WriteTarget,
};
use rp2040_hal::pac::UART0;
use rp2040_hal::uart::{Reader, ValidUartPinout};

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

pub struct TimeoutDmaReader<
    CH: ChannelIndex,
    ALARM: DelayNs,
    FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
    F: Fn(Vec<u8>),
    const N: usize,
> {
    alarm: ALARM,
    channel: Channel<CH>,
    from: FROM,
    on_data: F,
}

impl<
    CH: ChannelIndex,
    ALARM: DelayNs,
    FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
    F: Fn(Vec<u8>),
    const N: usize,
> TimeoutDmaReader<CH, ALARM, FROM, F, N>
{
    pub fn new(channel: Channel<CH>, alarm: ALARM, from: FROM, on_data: F) -> Self {
        Self {
            alarm,
            channel,
            from,
            on_data,
        }
    }
}
impl<CH: ChannelIndex, ALARM: DelayNs, P: ValidUartPinout<UART0>, F: Fn(Vec<u8>), const N: usize>
    TimeoutDmaReader<CH, ALARM, Reader<UART0, P>, F, N>
{
    pub async fn run(self) {
        let Self {
            mut alarm,
            mut channel,
            mut from,
            on_data,
        } = self;
        loop {
            let mut vec = Vec::with_capacity(32);
            let cap = vec.spare_capacity_mut();
            let buf =
                unsafe { core::slice::from_raw_parts_mut(cap.as_mut_ptr().cast(), cap.len()) };
            let mut async_reader = AsyncReader::new(from);
            let len = async_reader.read(buf).await.unwrap();
            unsafe { vec.set_len(len) };
            //debug!("uart read {=[u8]:X} bytes", vec);
            on_data(vec);
            from = async_reader.into_inner();
            if len < 16 {
                continue;
            }
            debug!("start dma");
            let mut transfer = AsyncTransfer::new_single_buffer_irq1(
                Config::new(channel, from, VecCapWriteTarget(Vec::with_capacity(N))).start(),
            );
            let mut alarm_wait = alarm.delay_ms(100);
            (channel, from) = 'dma: loop {
                let dma_wait = transfer.wait_done();
                match first_until(dma_wait, alarm_wait).await {
                    Ok(_) => {
                        debug!("dma irq first");
                        alarm_wait = alarm.delay_ms(100);
                        let (channel, from, target) = transfer.into_inner().wait();
                        transfer = AsyncTransfer::new_single_buffer_irq1(
                            Config::new(channel, from, VecCapWriteTarget(Vec::with_capacity(N)))
                                .start(),
                        );
                        let mut vec = target.0;
                        unsafe { vec.set_len(N) };
                        //debug!("dma read {=[u8]:X} bytes", vec);
                        on_data(vec);
                    }
                    Err(_) => {
                        debug!("alarm irq first");
                        let (ch, from, vec) = abort(transfer.into_inner());
                        //debug!("dma alarm read {=[u8]:X} bytes", vec);
                        on_data(vec);
                        break 'dma (ch, from);
                    }
                }
            }
        }
    }
}

fn abort<CH: ChannelIndex, FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget>(
    transfer: Transfer<Channel<CH>, FROM, VecCapWriteTarget>,
) -> (Channel<CH>, FROM, Vec<u8>) {
    let (ch, read, target) = transfer.abort();
    let next_addr = ch.ch().ch_al1_write_addr().read().bits() as usize;
    let mut vec = target.0;
    unsafe { vec.set_len(next_addr - vec.as_ptr() as usize) };
    (ch, read, vec)
}

fn first_until<T, U>(
    f1: impl Future<Output = T>,
    f2: impl Future<Output = U>,
) -> impl Future<Output = Result<T, U>> {
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
