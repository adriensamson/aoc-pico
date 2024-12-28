use crate::waker_slot::{WakerCell, WakerFuture};

pub struct DmaIrq0Listener([WakerCell; 12]);

impl DmaIrq0Listener {
    pub const fn new() -> Self {
        Self([const { WakerCell::new() }; 12])
    }

    pub fn wait_done(&self, channel: usize) -> WakerFuture {
        self.0[channel].as_future()
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            let dma = rp2040_pac::DMA::ptr();
            let status = unsafe { dma.as_ref().unwrap().ints0().read().ints0().bits() };
            for ch in 0..12 {
                if status & (1 << ch) != 0 {
                    self.0[ch].0.borrow_ref_mut(cs).wake();
                }
            }
            unsafe { dma.as_ref().unwrap().ints0().write(|w| w.bits(status as u32)) };
        })
    }
}


pub struct DmaIrq1Listener([WakerCell; 12]);

impl DmaIrq1Listener {
    pub const fn new() -> Self {
        Self([const { WakerCell::new() }; 12])
    }

    pub fn wait_done(&self, channel: usize) -> WakerFuture {
        self.0[channel].as_future()
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            let dma = rp2040_pac::DMA::ptr();
            let status = unsafe { dma.as_ref().unwrap().ints1().read().ints1().bits() };
            for ch in 0..12 {
                if status & (1 << ch) != 0 {
                    self.0[ch].0.borrow_ref_mut(cs).wake();
                }
            }
            unsafe { dma.as_ref().unwrap().ints1().write(|w| w.bits(status as u32)) };
        })
    }
}
