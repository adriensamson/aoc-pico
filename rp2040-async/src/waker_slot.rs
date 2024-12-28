use alloc::borrow::ToOwned;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;

pub(crate) struct WakerSlot {
    triggered: bool,
    waker: Option<Waker>
}

impl WakerSlot {
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

pub(crate) struct WakerCell(pub(crate) Mutex<RefCell<WakerSlot>>);

impl WakerCell {
    pub const fn new() -> Self {
        Self(Mutex::new(RefCell::new(WakerSlot::new())))
    }

    pub fn as_future(&self) -> WakerFuture {
        WakerFuture(self)
    }
}

pub struct WakerFuture<'a>(&'a WakerCell);

impl Future for WakerFuture<'_> {
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
