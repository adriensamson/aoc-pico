use alloc::vec::Vec;
use core::ptr::write_volatile;
use rp_pico::hal::dma::double_buffer::{Config, Transfer, WriteNext};
use rp_pico::hal::dma::{
    Channel, ChannelIndex, EndlessReadTarget, ReadTarget, SingleChannel, WriteTarget,
};
use rp_pico::hal::fugit::ExtU32;
use rp_pico::hal::timer::Alarm;

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
    ALARM: Alarm,
    FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
    const N: usize,
> {
    alarm: ALARM,
    elements: Option<(Channel<CH1>, Channel<CH2>, FROM)>,
    transfer: Option<
        Transfer<Channel<CH1>, Channel<CH2>, FROM, VecCapWriteTarget, WriteNext<VecCapWriteTarget>>,
    >,
}

impl<
        CH1: ChannelIndex,
        CH2: ChannelIndex,
        ALARM: Alarm,
        FROM: ReadTarget<ReceivedWord = u8> + EndlessReadTarget,
        const N: usize,
    > DoubleChannelReader<CH1, CH2, ALARM, FROM, N>
{
    pub fn new(ch1: Channel<CH1>, ch2: Channel<CH2>, alarm: ALARM, from: FROM) -> Self {
        Self {
            alarm,
            elements: Some((ch1, ch2, from)),
            transfer: None,
        }
    }

    pub fn reader(&mut self) -> Option<&mut FROM> {
        if let Some((_, _, reader)) = &mut self.elements {
            Some(reader)
        } else {
            None
        }
    }

    pub fn start(&mut self) -> Result<(), ()> {
        let (ch1, ch2, from) = self.elements.take().ok_or(())?;
        let transfer = Config::new((ch1, ch2), from, VecCapWriteTarget(Vec::with_capacity(N)))
            .start()
            .write_next(VecCapWriteTarget(Vec::with_capacity(N)));
        self.transfer = Some(transfer);
        self.alarm.schedule(100.millis()).unwrap();
        self.alarm.enable_interrupt();
        Ok(())
    }

    pub fn on_dma_irq(&mut self) -> Result<Vec<u8>, ()> {
        let transfer = self.transfer.take().ok_or(())?;

        self.alarm.schedule(100.millis()).unwrap();
        let (target, transfer) = transfer.wait();
        let transfer = transfer.write_next(VecCapWriteTarget(Vec::with_capacity(N)));
        self.transfer = Some(transfer);

        let mut vec = target.0;
        unsafe { vec.set_len(N) };
        Ok(vec)
    }

    pub fn on_alarm_irq(&mut self) -> Result<Vec<u8>, ()> {
        let transfer = self.transfer.take().ok_or(())?;

        self.alarm.disable_interrupt();

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
        let transcount = if transcount1 > 0 { transcount1 } else { transcount2 };
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

        self.elements = Some((ch1, ch2, read));

        let mut vec = target.0;
        unsafe { vec.set_len(N - transcount) };
        Ok(vec)
    }
}
