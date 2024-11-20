#![no_std]
#![no_main]

extern crate alloc;

use embedded_alloc::LlffHeap as Heap;
use panic_probe as _;
use defmt_rtt as _;

mod console;
mod aoc;

#[global_allocator]
static HEAP: Heap = Heap::empty();

unsafe fn init_heap() {
    use core::mem::MaybeUninit;
    use core::ptr::addr_of_mut;
    const HEAP_SIZE: usize = 1024 * 16;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
}

#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use alloc::string::String;use rp_pico::hal::{Sio, gpio::Pin, gpio::Pins, Watchdog, Clock, Timer};
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::uart::{UartPeripheral, UartConfig, Enabled};
    use rp_pico::pac::UART0;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    use embedded_hal::delay::DelayNs;
    use crate::aoc::AocRunner;
    use crate::console::Console;
    use crate::init_heap;

    type Uart = UartPeripheral<Enabled, UART0, (Pin<Gpio0, FunctionUart, PullDown>, Pin<Gpio1, FunctionUart, PullDown>)>;

    #[shared]
    struct Shared {
        uart: Uart,
        timer: Timer,
    }

    #[local]
    struct Local {}

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

        let uart = UartPeripheral::new(
            pac.UART0,
            (pins.gpio0.into_function(), pins.gpio1.into_function()),
            &mut pac.RESETS,
        ).enable(
            UartConfig::default(),
            clocks.peripheral_clock.freq(),
        ).unwrap();

        (Shared {
            uart,
            timer,
        }, Local {})
    }

    #[idle(shared = [uart, timer])]
    fn idle(mut cx: idle::Context) -> ! {
        let mut console = Console::new();
        let mut aoc_runner = AocRunner::new();
        let mut current_line = String::new();
        loop {
            cx.shared.uart.lock(|uart| {
                const CHUNK_SIZE : usize = 64;
                let mut buf = [0u8; CHUNK_SIZE];
                loop {
                    match uart.read_raw(&mut buf) {
                        Ok(count) => {
                            console.push(&buf[..count]);
                            while let Some(mut line) = console.pop_line() {
                                uart.write_raw(line.as_bytes()).unwrap();
                                uart.write_raw(b"\r\n").unwrap();
                                if !current_line.is_empty() {
                                    current_line.push_str(&line);
                                    core::mem::swap(&mut current_line, &mut line);
                                    current_line.clear();
                                }
                                for result in aoc_runner.push_line(line) {
                                    uart.write_raw(result.as_bytes()).unwrap();
                                    uart.write_raw(b"\r\n").unwrap();
                                }
                            }
                            let curr = console.pop_current_line();
                            uart.write_raw(curr.as_bytes()).unwrap();
                            current_line.push_str(&curr);
                            if count < CHUNK_SIZE {
                                break;
                            }
                        },
                        Err(_) => break,
                    }
                }
            });
            cx.shared.timer.lock(|timer| {
                timer.delay_ms(10);
            });
        }
    }
}
