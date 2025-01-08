use crate::trigger::{TriggerCell, TriggerFuture};
use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use rp2040_hal::dma::double_buffer::Transfer as DoubleBufferTransfer;
use rp2040_hal::dma::single_buffer::Transfer as SingleBufferTransfer;
use rp2040_hal::dma::{Channel, ChannelIndex, ReadTarget, WriteTarget};

pub trait DmaIrq {
    fn read_status_mask() -> u16;
    fn write_status_mask(mask: u16);
}
pub struct DmaIrq0;
pub struct DmaIrq1;

impl DmaIrq for DmaIrq0 {
    fn read_status_mask() -> u16 {
        unsafe {
            rp2040_hal::pac::DMA::ptr()
                .as_ref()
                .unwrap()
                .ints0()
                .read()
                .ints0()
                .bits()
        }
    }
    fn write_status_mask(mask: u16) {
        unsafe {
            rp2040_hal::pac::DMA::ptr()
                .as_ref()
                .unwrap()
                .ints0()
                .write(|w| w.bits(mask as u32))
        };
    }
}

impl DmaIrq for DmaIrq1 {
    fn read_status_mask() -> u16 {
        unsafe {
            rp2040_hal::pac::DMA::ptr()
                .as_ref()
                .unwrap()
                .ints1()
                .read()
                .ints1()
                .bits()
        }
    }
    fn write_status_mask(mask: u16) {
        unsafe {
            rp2040_hal::pac::DMA::ptr()
                .as_ref()
                .unwrap()
                .ints1()
                .write(|w| w.bits(mask as u32))
        };
    }
}

pub struct DmaIrqHandler<I: DmaIrq>([TriggerCell; 12], PhantomData<I>);

impl<I: DmaIrq> DmaIrqHandler<I> {
    pub const fn new() -> Self {
        Self([const { TriggerCell::new() }; 12], PhantomData)
    }

    pub unsafe fn wait_done(&self, channel: u8) -> TriggerFuture {
        self.0[channel as usize].as_future()
    }

    pub fn wait_single_buffer_transfer<'t, CH: ChannelIndex, FROM: ReadTarget, TO: WriteTarget>(
        &self,
        transfer: &'t mut SingleBufferTransfer<Channel<CH>, FROM, TO>,
    ) -> TransferFuture<
        't,
        impl Future<Output = ()> + '_,
        SingleBufferTransfer<Channel<CH>, FROM, TO>,
    > {
        TransferFuture {
            inner: self.0[CH::id() as usize].as_future(),
            transfer,
        }
    }

    pub fn wait_double_buffer_transfer<
        't,
        CH1: ChannelIndex,
        CH2: ChannelIndex,
        FROM: ReadTarget,
        TO: WriteTarget,
        STATE,
    >(
        &self,
        transfer: &'t mut DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
    ) -> TransferFuture<
        't,
        impl Future<Output = ()> + '_,
        DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
    > {
        TransferFuture {
            inner: first_future(
                self.0[CH1::id() as usize].as_future(),
                self.0[CH2::id() as usize].as_future(),
            ),
            transfer,
        }
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            let status = I::read_status_mask();
            for ch in 0..12 {
                if status & (1 << ch) != 0 {
                    self.0[ch].0.borrow_ref_mut(cs).wake();
                }
            }
            I::write_status_mask(status);
        })
    }
}

pub struct TransferFuture<'t, F, T> {
    inner: F,
    #[allow(dead_code)]
    transfer: &'t mut T,
}

impl<'t, F: Future<Output = ()>, T> Future for TransferFuture<'t, F, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.poll(cx)
    }
}

pub trait WaitDone {
    #[allow(async_fn_in_trait)]
    async fn wait_done(&mut self);
}

pub struct AsyncTransfer<T, I: DmaIrq>(T, PhantomData<I>);

impl<T, I: DmaIrq> AsyncTransfer<T, I> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

fn first_future<T>(
    f1: impl Future<Output = T>,
    f2: impl Future<Output = T>,
) -> impl Future<Output = T> {
    async {
        let (mut p1, mut p2) = (core::pin::pin!(f1), core::pin::pin!(f2));
        poll_fn(move |cx| {
            if let Poll::Ready(t) = p1.as_mut().poll(cx) {
                Poll::Ready(t)
            } else if let Poll::Ready(t) = p2.as_mut().poll(cx) {
                Poll::Ready(t)
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

#[cfg(feature = "dma0")]
pub mod dma0 {
    use crate::dma::{AsyncTransfer, DmaIrq0, DmaIrqHandler};
    use core::marker::PhantomData;
    use rp2040_hal::dma::{Channel, ChannelIndex, ReadTarget, WriteTarget};
    use rp2040_hal::pac::interrupt;

    pub static DMA_IRQ_0_HANDLER: DmaIrqHandler<DmaIrq0> = DmaIrqHandler::new();

    #[interrupt]
    fn DMA_IRQ_0() {
        DMA_IRQ_0_HANDLER.on_irq();
    }

    impl<CH: ChannelIndex, FROM: ReadTarget, TO: WriteTarget>
        AsyncTransfer<super::SingleBufferTransfer<Channel<CH>, FROM, TO>, DmaIrq0>
    {
        pub fn new_single_buffer_irq0(
            transfer: super::SingleBufferTransfer<Channel<CH>, FROM, TO>,
        ) -> Self {
            Self(transfer, PhantomData)
        }
    }
    impl<CH: ChannelIndex, FROM: ReadTarget, TO: WriteTarget> super::WaitDone
        for AsyncTransfer<super::SingleBufferTransfer<Channel<CH>, FROM, TO>, DmaIrq0>
    {
        async fn wait_done(&mut self) {
            DMA_IRQ_0_HANDLER
                .wait_single_buffer_transfer(&mut self.0)
                .await
        }
    }

    impl<CH1: ChannelIndex, CH2: ChannelIndex, FROM: ReadTarget, TO: WriteTarget, STATE>
        AsyncTransfer<
            super::DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
            DmaIrq0,
        >
    {
        pub fn new_double_buffer_irq0(
            transfer: super::DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
        ) -> Self {
            Self(transfer, PhantomData)
        }
    }
    impl<CH1: ChannelIndex, CH2: ChannelIndex, FROM: ReadTarget, TO: WriteTarget, STATE>
        super::WaitDone
        for AsyncTransfer<
            super::DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
            DmaIrq0,
        >
    {
        async fn wait_done(&mut self) {
            DMA_IRQ_0_HANDLER
                .wait_double_buffer_transfer(&mut self.0)
                .await
        }
    }
}

#[cfg(feature = "dma1")]
pub mod dma1 {
    use crate::dma::{AsyncTransfer, DmaIrq1, DmaIrqHandler};
    use core::marker::PhantomData;
    use rp2040_hal::dma::{Channel, ChannelIndex, ReadTarget, WriteTarget};
    use rp2040_hal::pac::interrupt;

    pub static DMA_IRQ_1_HANDLER: DmaIrqHandler<DmaIrq1> = DmaIrqHandler::new();

    #[interrupt]
    fn DMA_IRQ_1() {
        DMA_IRQ_1_HANDLER.on_irq();
    }

    impl<CH: ChannelIndex, FROM: ReadTarget, TO: WriteTarget>
        AsyncTransfer<super::SingleBufferTransfer<Channel<CH>, FROM, TO>, DmaIrq1>
    {
        pub fn new_single_buffer_irq1(
            transfer: super::SingleBufferTransfer<Channel<CH>, FROM, TO>,
        ) -> Self {
            Self(transfer, PhantomData)
        }
    }
    impl<CH: ChannelIndex, FROM: ReadTarget, TO: WriteTarget> super::WaitDone
        for AsyncTransfer<super::SingleBufferTransfer<Channel<CH>, FROM, TO>, DmaIrq1>
    {
        async fn wait_done(&mut self) {
            DMA_IRQ_1_HANDLER
                .wait_single_buffer_transfer(&mut self.0)
                .await
        }
    }

    impl<CH1: ChannelIndex, CH2: ChannelIndex, FROM: ReadTarget, TO: WriteTarget, STATE>
        AsyncTransfer<
            super::DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
            DmaIrq1,
        >
    {
        pub fn new_double_buffer_irq1(
            transfer: super::DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
        ) -> Self {
            Self(transfer, PhantomData)
        }
    }
    impl<CH1: ChannelIndex, CH2: ChannelIndex, FROM: ReadTarget, TO: WriteTarget, STATE>
        super::WaitDone
        for AsyncTransfer<
            super::DoubleBufferTransfer<Channel<CH1>, Channel<CH2>, FROM, TO, STATE>,
            DmaIrq1,
        >
    {
        async fn wait_done(&mut self) {
            DMA_IRQ_1_HANDLER
                .wait_double_buffer_transfer(&mut self.0)
                .await
        }
    }
}
