use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use rp2040_hal::timer::{Alarm, Alarm0};
use crate::trigger::{TriggerCell, TriggerFuture};

pub struct TimerIrq0Handler(TriggerCell);

pub struct AlarmFuture<'c, 'a, A: Alarm>(TriggerFuture<'c>, &'a mut A);

impl<'c, 'a, A: Alarm> Future for AlarmFuture<'c, 'a, A> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fut = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        fut.poll(cx)
    }
}

impl<'c, 'a, A: Alarm> Drop for AlarmFuture<'c, 'a, A> {
    fn drop(&mut self) {
        self.1.disable_interrupt()
    }
}

impl TimerIrq0Handler {
    pub const fn new() -> Self {
        Self(TriggerCell::new())
    }

    pub fn wait_alarm<'a>(&self, alarm: &'a mut Alarm0) -> AlarmFuture<'_, 'a, Alarm0> {
        AlarmFuture(self.0.as_future(), alarm)
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            self.0.0.borrow_ref_mut(cs).wake();
            unsafe { rp2040_hal::pac::TIMER::ptr().as_ref().unwrap().intr().write(|w| w.alarm_0().clear_bit_by_one()) };
        })
    }
}

pub struct AsyncAlarm<A: Alarm>(A);

#[cfg(feature = "alarm0")]
mod alarm0 {
    use rp2040_hal::fugit::MicrosDurationU32;
    use rp2040_hal::pac::interrupt;
    use rp2040_hal::timer::{Alarm, Alarm0};
    use crate::TimerIrq0Handler;

    static TIMER_IRQ_0_HANDLER: TimerIrq0Handler = TimerIrq0Handler::new();

    #[interrupt]
    fn TIMER_IRQ_0() {
        TIMER_IRQ_0_HANDLER.on_irq();
    }

    impl super::AsyncAlarm<Alarm0> {
        pub fn new(alarm0: Alarm0) -> Self {
            Self(alarm0)
        }

        pub fn into_inner(mut self) -> Alarm0 {
            self.0.disable_interrupt();
            self.0
        }
    }

    impl embedded_hal_async::delay::DelayNs for super::AsyncAlarm<Alarm0> {
        async fn delay_ns(&mut self, ns: u32) {
            self.0.schedule(MicrosDurationU32::nanos(ns)).unwrap();
            self.0.clear_interrupt();
            self.0.enable_interrupt();
            TIMER_IRQ_0_HANDLER.wait_alarm(&mut self.0).await
        }
    }
}
