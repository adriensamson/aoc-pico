use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::{poll_fn, Future};
use core::ptr::write_volatile;
use core::task::Poll;
use defmt::debug;
use embedded_hal_async::delay::DelayNs;
use rp2040_async::dma::{DmaIrq1, DmaIrqHandler};
use rp2040_async::uart::UartIrqHandler;
use rp_pico::hal::dma::double_buffer::{Config, Transfer, WriteNext};
use rp_pico::hal::dma::{
    Channel, ChannelIndex, EndlessReadTarget, ReadTarget, SingleChannel, WriteTarget,
};
use rp_pico::hal::uart::{Reader, UartDevice, ValidUartPinout};

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

pub struct DoubleChannelReader<
    CH1: ChannelIndex,
    CH2: ChannelIndex,
    ALARM: DelayNs,
    FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
    F: Fn(Vec<u8>),
    const N: usize,
> {
    alarm: ALARM,
    channel1: Channel<CH1>,
    channel2: Channel<CH2>,
    from: FROM,
    on_data: F,
}

impl<
        CH1: ChannelIndex,
        CH2: ChannelIndex,
        ALARM: DelayNs,
        FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
        F: Fn(Vec<u8>),
        const N: usize,
    > DoubleChannelReader<CH1, CH2, ALARM, FROM, F, N>
{
    pub fn new(
        channel1: Channel<CH1>,
        channel2: Channel<CH2>,
        alarm: ALARM,
        from: FROM,
        on_data: F,
    ) -> Self {
        Self {
            alarm,
            channel1,
            channel2,
            from,
            on_data,
        }
    }
}
impl<
        CH1: ChannelIndex,
        CH2: ChannelIndex,
        ALARM: DelayNs,
        U: UartDevice,
        P: ValidUartPinout<U>,
        F: Fn(Vec<u8>),
        const N: usize,
    > DoubleChannelReader<CH1, CH2, ALARM, Reader<U, P>, F, N>
{
    pub async fn run(
        self,
        uart0irq_handler: &'static UartIrqHandler<U, P>,
        dma_irq1handler: &'static DmaIrqHandler<DmaIrq1>,
    ) {
        let Self {
            mut alarm,
            mut channel1,
            mut channel2,
            mut from,
            on_data,
        } = self;
        loop {
            let mut vec = Vec::with_capacity(32);
            let cap = vec.spare_capacity_mut();
            let buf =
                unsafe { core::slice::from_raw_parts_mut(cap.as_mut_ptr().cast(), cap.len()) };
            let len = uart0irq_handler.wait_rx(&mut from, buf).await;
            unsafe { vec.set_len(len) };
            debug!("uart read {=[u8]:X} bytes", vec);
            on_data(vec);
            if len < 16 {
                continue;
            }
            debug!("start dma");
            let mut transfer = Config::new(
                (channel1, channel2),
                from,
                VecCapWriteTarget(Vec::with_capacity(N)),
            )
            .start()
            .write_next(VecCapWriteTarget(Vec::with_capacity(N)));
            let mut alarm_wait = alarm.delay_ms(100);
            (channel1, channel2, from) = 'dma: loop {
                let dma_wait = first_future(
                    unsafe { dma_irq1handler.wait_done(CH1::id()) },
                    unsafe { dma_irq1handler.wait_done(CH2::id()) },
                );
                match first_until(dma_wait, alarm_wait).await {
                    Ok(_) => {
                        debug!("dma irq first");
                        alarm_wait = alarm.delay_ms(100);
                        let (target, transfer2) = transfer.wait();
                        transfer = transfer2.write_next(VecCapWriteTarget(Vec::with_capacity(N)));
                        let mut vec = target.0;
                        unsafe { vec.set_len(N) };
                        debug!("dma read {=[u8]:X} bytes", vec);
                        on_data(vec);
                    }
                    Err(_) => {
                        debug!("alarm irq first");
                        let (ch1, ch2, from, vec) = abort(transfer, N);
                        debug!("dma alarm read {=[u8]:X} bytes", vec);
                        on_data(vec);
                        break 'dma (ch1, ch2, from);
                    }
                }
            }
        }
    }
}

fn abort<
    CH1: ChannelIndex,
    CH2: ChannelIndex,
    FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
>(
    transfer: Transfer<
        Channel<CH1>,
        Channel<CH2>,
        FROM,
        VecCapWriteTarget,
        WriteNext<VecCapWriteTarget>,
    >,
    expected_len: usize,
) -> (Channel<CH1>, Channel<CH2>, FROM, Vec<u8>) {
    let dma = unsafe { rp_pico::pac::DMA::steal() };
    let dma_ch1 = dma.ch(CH1::id() as usize);
    let dma_ch2 = dma.ch(CH2::id() as usize);

    let mask = 1 << CH1::id() | 1 << CH2::id();

    // disable (spurious) interrupts
    let inte0_mask = dma.inte0().read().bits() & mask;
    let inte1_mask = dma.inte1().read().bits() & mask;
    if inte0_mask != 0 {
        unsafe { write_volatile(dma.inte0().as_ptr().byte_add(0x3000), inte0_mask) };
    }
    if inte1_mask != 0 {
        unsafe { write_volatile(dma.inte1().as_ptr().byte_add(0x3000), inte1_mask) };
    }

    // pause
    dma_ch1.ch_ctrl_trig().write(|w| w.en().clear_bit());
    dma_ch2.ch_ctrl_trig().write(|w| w.en().clear_bit());
    // read transcount
    let transcount1 = dma_ch1.ch_trans_count().read().bits() as usize;
    let transcount2 = dma_ch2.ch_trans_count().read().bits() as usize;
    let transcount = if transcount1 > 0 {
        transcount1
    } else {
        transcount2
    };
    // abort
    let chan_abort = dma.chan_abort();
    unsafe { chan_abort.write(|w| w.bits(mask)) };
    while chan_abort.read().bits() != 0 {}

    let (target, transfer) = transfer.wait();
    let (mut ch1, mut ch2, read, _) = transfer.wait();
    ch1.check_irq0();
    ch1.check_irq1();
    ch2.check_irq0();
    ch2.check_irq1();

    if inte0_mask != 0 {
        unsafe { write_volatile(dma.inte0().as_ptr().byte_add(0x2000), inte0_mask) };
    }
    if inte1_mask != 0 {
        unsafe { write_volatile(dma.inte1().as_ptr().byte_add(0x2000), inte1_mask) };
    }

    let mut vec = target.0;
    unsafe { vec.set_len(expected_len - transcount) };
    (ch1, ch2, read, vec)
}

fn first_future<T>(
    f1: impl Future<Output = T> + 'static,
    f2: impl Future<Output = T> + 'static,
) -> impl Future<Output = T> {
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
