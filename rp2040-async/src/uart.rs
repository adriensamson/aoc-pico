use core::cell::RefCell;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;
use rp2040_hal::uart::{ReadErrorType, Reader, UartDevice, ValidUartPinout};

struct UartSlot {
    read_result: Option<(usize, Result<(), ReadErrorType>)>,
    read_fn: fn(*mut (), (*mut u8, usize)) -> (usize, Result<(), ReadErrorType>),
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

    fn register(&self, read_fn: fn(*mut (), (*mut u8, usize)) -> (usize, Result<(), ReadErrorType>), reader: *mut (), buf: (*mut u8, usize), waker: &Waker) {
        critical_section::with(|cs| {
            *self.0.borrow_ref_mut(cs) = Some(UartSlot {
                read_result: None,
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

    fn check_read_result(&self) -> Option<(usize, Result<(), ReadErrorType>)> {
        critical_section::with(|cs| {
            self.0.borrow_ref_mut(cs).as_mut().and_then(|slot| slot.read_result.take())
        })
    }

    fn on_irq(&self) {
        critical_section::with(|cs| {
            if let Some(slot) = &mut *self.0.borrow_ref_mut(cs) {
                slot.read_result = Some((slot.read_fn)(slot.reader, slot.buf));
                slot.waker.wake_by_ref();
            }
        })
    }
}

pub struct UartRxFuture<'a, 'b, D: UartDevice, P: ValidUartPinout<D>>(&'a UartCell, &'b mut Reader<D, P>, &'b mut [u8]);
impl<D: UartDevice, P: ValidUartPinout<D>> Future for UartRxFuture<'_, '_, D, P> {
    type Output = Result<usize, ReadErrorType>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some((len, err)) = self.0.check_read_result() {
            self.0.unregister();
            return Poll::Ready(err.map(|_| len));
        }
        self.0.register(read_raw::<D, P>, (&raw mut *self.1).cast(), (self.2.as_mut_ptr(), self.2.len()), cx.waker());
        Poll::Pending
    }
}

fn read_raw<D: UartDevice, P: ValidUartPinout<D>>(reader: *mut (), buf: (*mut u8, usize)) -> (usize, Result<(), ReadErrorType>) {
    let reader = unsafe { &mut *reader.cast::<Reader<D, P>>() };
    let buf = unsafe { core::slice::from_raw_parts_mut(buf.0, buf.1) };
    match reader.read_raw(buf) {
        Ok(len) => (len, Ok(())),
        Err(nb::Error::Other(err)) => (err.discarded.len(), Err(err.err_type)),
        Err(_) => unreachable!(),
    }
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

pub struct AsyncReader<D: UartDevice, P: ValidUartPinout<D>>(Reader<D, P>);

impl <D: UartDevice, P: ValidUartPinout<D>> AsyncReader<D, P> {
    pub fn new(reader: Reader<D, P>) -> Self {
        Self(reader)
    }

    pub fn into_inner(self) -> Reader<D, P> {
        self.0
    }
}

#[cfg(feature = "uart0")]
pub mod uart0 {
    use rp2040_hal::pac::{interrupt, UART0};
    use rp2040_hal::uart::{ReadErrorType, ValidUartPinout};
    use crate::uart::{AsyncReader, UartIrqHandler};

    pub static UART0_IRQ_HANDLER : UartIrqHandler<UART0> = UartIrqHandler::new();

    #[interrupt]
    fn UART0_IRQ() {
        UART0_IRQ_HANDLER.on_irq();
    }

    impl<P: ValidUartPinout<UART0>> embedded_io_async::ErrorType for AsyncReader<UART0, P> {
        type Error = ReadErrorType;
    }

    impl<P: ValidUartPinout<UART0>> embedded_io_async::Read for AsyncReader<UART0, P> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            UART0_IRQ_HANDLER.wait_rx(&mut self.0, buf).await
        }
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
