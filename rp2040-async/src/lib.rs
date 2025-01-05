#![no_std]
extern crate alloc;

mod trigger;
mod dma;
mod timer;
mod uart;

pub use dma::DmaIrq0Handler;
pub use dma::DmaIrq1Handler;
use rp2040_hal::pac::UART0;
//use rp2040_hal::uart::ValidUartPinout;
pub use timer::{TimerIrqHandler, AsyncAlarm};
pub use uart::UartIrqHandler;
pub type Uart0IrqHandler<P /*: ValidUartPinout<UART0>*/> = UartIrqHandler<UART0, P>;
