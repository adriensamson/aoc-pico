use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;
use rp2040_hal::uart::{Reader, UartDevice, ValidUartPinout};

struct UartSlot<D: UartDevice, P: ValidUartPinout<D>> {
    len_read: usize,
    reader: *mut Reader<D, P>,
    buf: (*mut u8, usize),
    waker: Waker,
}
unsafe impl<D: UartDevice, P: ValidUartPinout<D>> Send for UartSlot<D, P> {}

struct UartCell<D: UartDevice, P: ValidUartPinout<D>>(Mutex<RefCell<Option<UartSlot<D, P>>>>);

impl<D: UartDevice, P: ValidUartPinout<D>> UartCell<D, P> {
    const fn new() -> Self {
        Self(Mutex::new(RefCell::new(const { None })))
    }

    fn register(&self, reader: *mut Reader<D, P>, buf: (*mut u8, usize), waker: &Waker) {
        critical_section::with(|cs| {
            *self.0.borrow_ref_mut(cs) = Some(UartSlot {
                len_read: 0,
                reader,
                buf,
                waker: waker.clone(),
            });
        });
    }

    fn unregister(&self) {
        critical_section::with(|cs| {
            *self.0.borrow_ref_mut(cs) = None;
        });
    }

    fn check_len(&self) -> Option<usize> {
        critical_section::with(|cs| {
            let len = self.0.borrow_ref(cs).as_ref().map(|slot| slot.len_read)?;
            if len > 0 {
                Some(len)
            } else {
                None
            }
        })
    }

    fn on_irq(&self) {
        critical_section::with(|cs| {
            if let Some(slot) = &mut *self.0.borrow_ref_mut(cs) {
                let device = unsafe { slot.reader.as_mut().unwrap() };
                let buf = unsafe { core::slice::from_raw_parts_mut(slot.buf.0, slot.buf.1) };
                if let Ok(len) = device.read_raw(buf) {
                    slot.len_read = len;
                    slot.waker.wake_by_ref();
                }
            }
        })
    }
}

pub struct UartRxFuture<'a, 'b, D: UartDevice, P: ValidUartPinout<D>>(&'a UartCell<D, P>, &'b mut Reader<D, P>, &'b mut [u8]);
impl<D: UartDevice, P: ValidUartPinout<D>> Future for UartRxFuture<'_, '_, D, P> {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(len) = self.0.check_len() {
            self.0.unregister();
            return Poll::Ready(len);
        }
        self.0.register(&raw mut *self.1, (self.2.as_mut_ptr(), self.2.len()), cx.waker());
        Poll::Pending
    }
}

pub struct UartIrqHandler<D: UartDevice, P: ValidUartPinout<D>>(UartCell<D, P>);

impl<D: UartDevice, P: ValidUartPinout<D>> UartIrqHandler<D, P> {
    pub const fn new() -> Self {
        Self(UartCell::new())
    }

    pub fn wait_rx<'a>(&self, device: &'a mut Reader<D, P>, buf: &'a mut [u8]) -> UartRxFuture<'_, 'a, D, P> {
        UartRxFuture(&self.0, device, buf)
    }

    pub fn on_irq(&self) {
        self.0.on_irq()
    }
}
