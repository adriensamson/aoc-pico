use crate::trigger::{TriggerCell, TriggerFuture};

pub struct TimerIrq0Handler(TriggerCell);

impl TimerIrq0Handler {
    pub const fn new() -> Self {
        Self(TriggerCell::new())
    }

    pub fn wait_alarm(&self) -> TriggerFuture {
        self.0.as_future()
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            self.0.0.borrow_ref_mut(cs).wake();
            unsafe { rp2040_hal::pac::TIMER::ptr().as_ref().unwrap().intr().write(|w| w.alarm_0().clear_bit_by_one()) };
        })
    }
}
