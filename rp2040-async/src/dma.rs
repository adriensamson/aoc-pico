use core::marker::PhantomData;
use crate::trigger::{TriggerCell, TriggerFuture};

pub trait DmaIrq {
    fn read_status_mask() -> u16;
    fn write_status_mask(mask: u16);
}
pub struct DmaIrq0;
pub struct DmaIrq1;

impl DmaIrq for DmaIrq0 {
    fn read_status_mask() -> u16 {
        unsafe { rp2040_hal::pac::DMA::ptr().as_ref().unwrap().ints0().read().ints0().bits() }
    }
    fn write_status_mask(mask: u16) {
        unsafe { rp2040_hal::pac::DMA::ptr().as_ref().unwrap().ints0().write(|w| w.bits(mask as u32)) };
    }
}

impl DmaIrq for DmaIrq1 {
    fn read_status_mask() -> u16 {
        unsafe { rp2040_hal::pac::DMA::ptr().as_ref().unwrap().ints1().read().ints1().bits() }
    }
    fn write_status_mask(mask: u16) {
        unsafe { rp2040_hal::pac::DMA::ptr().as_ref().unwrap().ints1().write(|w| w.bits(mask as u32)) };
    }
}


pub struct DmaIrqHandler<I:DmaIrq>([TriggerCell; 12], PhantomData<I>);

impl<I: DmaIrq> DmaIrqHandler<I> {
    pub const fn new() -> Self {
        Self([const { TriggerCell::new() }; 12], PhantomData)
    }

    pub unsafe fn wait_done(&self, channel: u8) -> TriggerFuture {
        self.0[channel as usize].as_future()
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

#[cfg(feature = "dma0")]
pub mod dma0 {
    use rp2040_hal::pac::interrupt;
    use crate::dma::{DmaIrq0, DmaIrqHandler};

    pub static DMA_IRQ_0_HANDLER : DmaIrqHandler<DmaIrq0> = DmaIrqHandler::new();

    #[interrupt]
    fn DMA_IRQ_0() {
        DMA_IRQ_0_HANDLER.on_irq();
    }
}

#[cfg(feature = "dma1")]
pub mod dma1 {
    use rp2040_hal::pac::interrupt;
    use crate::dma::{DmaIrq1, DmaIrqHandler};

    pub static DMA_IRQ_1_HANDLER : DmaIrqHandler<DmaIrq1> = DmaIrqHandler::new();

    #[interrupt]
    fn DMA_IRQ_1() {
        DMA_IRQ_1_HANDLER.on_irq();
    }
}
