#![no_std]
#![no_main]

extern crate alloc;

use cortex_m_rt::heap_start;
use embedded_alloc::LlffHeap as Heap;
use panic_probe as _;
use defmt_rtt as _;

mod console;
mod aoc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

unsafe fn init_heap() {
    const HEAP_SIZE: usize = 1024 * 16;
    unsafe { HEAP.init(heap_start() as usize, HEAP_SIZE) }
}

#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use rp_pico::hal::{Sio, gpio::Pin, gpio::Pins, Watchdog, Clock, Timer};
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::uart::{UartPeripheral, UartConfig, Reader};
    use rp_pico::pac::UART0;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    use crate::aoc::AocRunner;
    use crate::console::{Console, ConsoleUartWriter};
    use crate::init_heap;

    type UartPinout = (Pin<Gpio0, FunctionUart, PullDown>, Pin<Gpio1, FunctionUart, PullDown>);

    #[shared]
    struct Shared {
        console: Console<ConsoleUartWriter<UART0, UartPinout>, AocRunner>,
        timer: Timer,
    }

    #[local]
    struct Local {
        uart_rx: Reader<UART0, UartPinout>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        unsafe { init_heap() };

        let mut pac = cx.device;
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        ).unwrap();
        let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

        let sio = Sio::new(pac.SIO);

        let pins = Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let (uart_rx, uart_tx) = UartPeripheral::new(
            pac.UART0,
            (pins.gpio0.into_function(), pins.gpio1.into_function()),
            &mut pac.RESETS,
        ).enable(
            UartConfig::default(),
            clocks.peripheral_clock.freq(),
        ).unwrap().split();

        let aoc_runner = AocRunner::new();
        let console = Console::new(ConsoleUartWriter(uart_tx), aoc_runner);

        (Shared {
            console,
            timer,
        }, Local {
            uart_rx,
        })
    }

    #[idle(local = [uart_rx], shared = [console])]
    fn idle(mut cx: idle::Context) -> ! {
        let uart_rx = cx.local.uart_rx;

        const CHUNK_SIZE : usize = 64;
        let mut buf = [0u8; CHUNK_SIZE];

        loop {
            match uart_rx.read_raw(&mut buf) {
                Ok(count) => {
                    cx.shared.console.lock(|c| c.push(&buf[..count]));
                },
                Err(_) => {},
            }
        }
    }
}
