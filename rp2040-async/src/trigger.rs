use alloc::borrow::ToOwned;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;

pub(crate) struct TriggerSlot {
    triggered: bool,
    waker: Option<Waker>
}

impl TriggerSlot {
    pub(crate) const fn new() -> Self {
        Self {
            triggered: false,
            waker: None,
        }
    }

    fn register(&mut self, waker: &Waker) {
        if let Some(w) = &mut self.waker {
            waker.clone_into(w)
        } else {
            self.waker.replace(waker.clone());
        }
        self.triggered = false;
    }

    pub(crate) fn wake(&mut self) {
        self.triggered = true;
        if let Some(w) = self.waker.take() {
            w.wake();
        }
    }

    fn unregister(&mut self) {
        self.waker.take();
    }

    fn check_triggered(&mut self) -> bool {
        let triggered = self.triggered;
        self.triggered = false;
        triggered
    }
}

pub(crate) struct TriggerCell(pub(crate) Mutex<RefCell<TriggerSlot>>);

impl TriggerCell {
    pub const fn new() -> Self {
        Self(Mutex::new(RefCell::new(TriggerSlot::new())))
    }

    pub fn as_future(&self) -> TriggerFuture {
        TriggerFuture(self)
    }
}

pub struct TriggerFuture<'a>(&'a TriggerCell);

impl Future for TriggerFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        critical_section::with(|cs| {
            let mut slot = self.0.0.borrow_ref_mut(cs);
            if slot.check_triggered() {
                slot.unregister();
                Poll::Ready(())
            } else {
                slot.register(cx.waker());
                Poll::Pending
            }
        })
    }
}

impl Drop for TriggerFuture<'_> {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            let mut slot = self.0.0.borrow_ref_mut(cs);
            slot.check_triggered();
            slot.unregister();
        })
    }
}
