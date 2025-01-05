use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use rp2040_hal::fugit::MicrosDurationU32;
use rp2040_hal::timer::{Alarm, Alarm0, Alarm1, Alarm2, Alarm3};
use crate::trigger::{TriggerCell, TriggerFuture};

pub struct TimerIrqHandler<A: AlarmIrq>(TriggerCell, PhantomData<A>);

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

pub trait AlarmIrq: Alarm {
    fn clear_interrupt();
}

impl AlarmIrq for Alarm0 {
    fn clear_interrupt() {
        unsafe { rp2040_hal::pac::TIMER::ptr().as_ref().unwrap().intr().write(|w| w.alarm_0().clear_bit_by_one()) };
    }
}

impl AlarmIrq for Alarm1 {
    fn clear_interrupt() {
        unsafe { rp2040_hal::pac::TIMER::ptr().as_ref().unwrap().intr().write(|w| w.alarm_1().clear_bit_by_one()) };
    }
}

impl AlarmIrq for Alarm2 {
    fn clear_interrupt() {
        unsafe { rp2040_hal::pac::TIMER::ptr().as_ref().unwrap().intr().write(|w| w.alarm_2().clear_bit_by_one()) };
    }
}

impl AlarmIrq for Alarm3 {
    fn clear_interrupt() {
        unsafe { rp2040_hal::pac::TIMER::ptr().as_ref().unwrap().intr().write(|w| w.alarm_3().clear_bit_by_one()) };
    }
}

impl<A: AlarmIrq> TimerIrqHandler<A> {
    pub const fn new() -> Self {
        Self(TriggerCell::new(), PhantomData)
    }

    pub fn wait_alarm<'a>(&self, alarm: &'a mut A) -> AlarmFuture<'_, 'a, A> {
        AlarmFuture(self.0.as_future(), alarm)
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            self.0.0.borrow_ref_mut(cs).wake();
            <A as  AlarmIrq>::clear_interrupt()
        })
    }
}

pub struct AsyncAlarm<A: Alarm>(A);

pub trait AlarmIrqRt: AlarmIrq + Sized {
    fn handler() -> &'static TimerIrqHandler<Self>;
}

impl<A: AlarmIrqRt> AsyncAlarm<A> {
    pub fn new(alarm: A) -> Self {
        Self(alarm)
    }

    pub fn into_inner(mut self) -> A {
        self.0.disable_interrupt();
        self.0
    }
}

impl<A: AlarmIrqRt + 'static> embedded_hal_async::delay::DelayNs for AsyncAlarm<A> {
    async fn delay_ns(&mut self, ns: u32) {
        self.0.schedule(MicrosDurationU32::nanos(ns)).unwrap();
        self.0.clear_interrupt();
        self.0.enable_interrupt();
        A::handler().wait_alarm(&mut self.0).await
    }
}

#[cfg(feature = "alarm0")]
mod alarm0 {
    use rp2040_hal::pac::interrupt;
    use rp2040_hal::timer::Alarm0;
    use super::{AlarmIrqRt, TimerIrqHandler};

    static TIMER_IRQ_0_HANDLER: TimerIrqHandler<Alarm0> = TimerIrqHandler::new();

    #[interrupt]
    fn TIMER_IRQ_0() {
        TIMER_IRQ_0_HANDLER.on_irq();
    }

    impl AlarmIrqRt for Alarm0 {
        fn handler() -> &'static TimerIrqHandler<Alarm0> {
            &TIMER_IRQ_0_HANDLER
        }
    }
}

#[cfg(feature = "alarm1")]
mod alarm1 {
    use rp2040_hal::pac::interrupt;
    use rp2040_hal::timer::Alarm1;
    use super::{AlarmIrqRt, TimerIrqHandler};

    static TIMER_IRQ_1_HANDLER: TimerIrqHandler<Alarm1> = TimerIrqHandler::new();

    #[interrupt]
    fn TIMER_IRQ_1() {
        TIMER_IRQ_1_HANDLER.on_irq();
    }

    impl AlarmIrqRt for Alarm1 {
        fn handler() -> &'static TimerIrqHandler<Alarm1> {
            &TIMER_IRQ_1_HANDLER
        }
    }
}

#[cfg(feature = "alarm2")]
mod alarm2 {
    use rp2040_hal::pac::interrupt;
    use rp2040_hal::timer::Alarm2;
    use super::{AlarmIrqRt, TimerIrqHandler};

    static TIMER_IRQ_2_HANDLER: TimerIrqHandler<Alarm2> = TimerIrqHandler::new();

    #[interrupt]
    fn TIMER_IRQ_2() {
        TIMER_IRQ_2_HANDLER.on_irq();
    }

    impl AlarmIrqRt for Alarm2 {
        fn handler() -> &'static TimerIrqHandler<Alarm2> {
            &TIMER_IRQ_2_HANDLER
        }
    }
}

#[cfg(feature = "alarm3")]
mod alarm3 {
    use rp2040_hal::pac::interrupt;
    use rp2040_hal::timer::Alarm3;
    use super::{AlarmIrqRt, TimerIrqHandler};

    static TIMER_IRQ_3_HANDLER: TimerIrqHandler<Alarm3> = TimerIrqHandler::new();

    #[interrupt]
    fn TIMER_IRQ_3() {
        TIMER_IRQ_3_HANDLER.on_irq();
    }

    impl AlarmIrqRt for Alarm3 {
        fn handler() -> &'static TimerIrqHandler<Alarm3> {
            &TIMER_IRQ_3_HANDLER
        }
    }
}
