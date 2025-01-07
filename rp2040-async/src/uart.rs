use core::cell::RefCell;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;
use rp2040_hal::uart::{Reader, UartDevice, ValidUartPinout};

struct UartSlot {
    len_read: usize,
    read_fn: fn(*mut (), (*mut u8, usize)) -> Option<usize>,
    reader: *mut (),
    buf: (*mut u8, usize),
    waker: Waker,
}
unsafe impl Send for UartSlot {}

struct UartCell(Mutex<RefCell<Option<UartSlot>>>);

impl UartCell {
    const fn new() -> Self {
        Self(Mutex::new(RefCell::new(const { None })))
    }

    fn register(&self, read_fn: fn(*mut (), (*mut u8, usize)) -> Option<usize>, reader: *mut (), buf: (*mut u8, usize), waker: &Waker) {
        critical_section::with(|cs| {
            *self.0.borrow_ref_mut(cs) = Some(UartSlot {
                len_read: 0,
                read_fn,
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
                if let Some(len) = (slot.read_fn)(slot.reader, slot.buf) {
                    slot.len_read = len;
                    slot.waker.wake_by_ref();
                }
            }
        })
    }
}

pub struct UartRxFuture<'a, 'b, D: UartDevice, P: ValidUartPinout<D>>(&'a UartCell, &'b mut Reader<D, P>, &'b mut [u8]);
impl<D: UartDevice, P: ValidUartPinout<D>> Future for UartRxFuture<'_, '_, D, P> {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(len) = self.0.check_len() {
            self.0.unregister();
            return Poll::Ready(len);
        }
        self.0.register(read_raw::<D, P>, (&raw mut *self.1).cast(), (self.2.as_mut_ptr(), self.2.len()), cx.waker());
        Poll::Pending
    }
}

fn read_raw<D: UartDevice, P: ValidUartPinout<D>>(reader: *mut (), buf: (*mut u8, usize)) -> Option<usize> {
    let reader = unsafe { &mut *reader.cast::<Reader<D, P>>() };
    let buf = unsafe { core::slice::from_raw_parts_mut(buf.0, buf.1) };
    reader.read_raw(buf).ok()
}

pub struct UartIrqHandler<D: UartDevice>(UartCell, PhantomData<D>);

unsafe impl<D: UartDevice> Sync for UartIrqHandler<D> {}

impl<D: UartDevice> UartIrqHandler<D> {
    pub const fn new() -> Self {
        Self(UartCell::new(), PhantomData)
    }

    pub fn wait_rx<'a, P: ValidUartPinout<D>>(&self, device: &'a mut Reader<D, P>, buf: &'a mut [u8]) -> UartRxFuture<'_, 'a, D, P> {
        UartRxFuture(&self.0, device, buf)
    }

    pub fn on_irq(&self) {
        self.0.on_irq()
    }
}

#[cfg(feature = "uart0")]
pub mod uart0 {
    use rp2040_hal::pac::{interrupt, UART0};
    use crate::uart::{UartIrqHandler};

    pub static UART0_IRQ_HANDLER : UartIrqHandler<UART0> = UartIrqHandler::new();

    #[interrupt]
    fn UART0_IRQ() {
        UART0_IRQ_HANDLER.on_irq();
    }
}

#[cfg(feature = "uart1")]
pub mod uart1 {
    use rp2040_hal::pac::{interrupt, UART1};
    use crate::uart::{UartIrqHandler};

    pub static UART1_IRQ_HANDLER : UartIrqHandler<UART1> = UartIrqHandler::new();

    #[interrupt]
    fn UART1_IRQ() {
        UART1_IRQ_HANDLER.on_irq();
    }
}
