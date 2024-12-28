use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::ops::Range;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;

struct UartSlot {
    len_read: usize,
    buf_waker: Option<(Range<*mut u8>, Waker)>
}
impl UartSlot {
    const fn new() -> Self {
        Self {
            len_read: 0,
            buf_waker: const { None }
        }
    }

    fn register(&mut self, buf: &mut [u8], waker: &Waker) {
        self.len_read = 0;
        self.buf_waker = Some((buf.as_mut_ptr_range(), waker.clone()));
    }

    fn unregister(&mut self) {
        self.len_read = 0;
        self.buf_waker = None;
    }
}

struct UartCell(Mutex<RefCell<UartSlot>>);

impl UartCell {
    const fn new() -> Self {
        Self(Mutex::new(RefCell::new(UartSlot::new())))
    }
}

pub struct UartRxFuture<'a, 'b>(&'a UartCell, &'b mut [u8]);
impl Future for UartRxFuture<'_, '_> {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        critical_section::with(|cs| {
            let mut slot = self.0.0.borrow_ref_mut(cs);
            if slot.len_read > 0 {
                slot.unregister();
                Poll::Ready(slot.len_read)
            } else {
                slot.register(self.1, cx.waker());
                Poll::Pending
            }
        })
    }
}

pub struct Uart0IrqHandler(UartCell);

impl Uart0IrqHandler {
    pub const fn new() -> Self {
        Self(UartCell::new())
    }

    pub fn wait_rx(&self, buf: &mut [u8]) -> UartRxFuture {
        UartRxFuture(&self.0, buf)
    }

    pub fn on_irq(&self) {
        critical_section::with(|cs| {
            let mut slot = self.0.0.borrow_ref_mut(cs);
            if let Some((buf, waker)) = slot.buf_waker.take() {
                let uart0 = unsafe { rp2040_pac::UART0::ptr().as_ref().unwrap() };
                uart0.uartdr().read();
                todo!();

                waker.wake();
            }
        })

    }
}
