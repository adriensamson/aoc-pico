#![no_std]
#![no_main]

extern crate alloc;

use panic_probe as _;
use defmt_rtt as _;

mod aoc;
//mod multicore;
mod memory;

#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use defmt::debug;
    use rp_pico::hal::{Sio, gpio::Pin, gpio::Pins, Watchdog, Clock};
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::uart::{UartPeripheral, UartConfig, Reader};
    use rp_pico::pac::UART0;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    use rtic::export::wfi;
    use crate::aoc::AocRunner;
    use aoc_pico::shell::{Commands, Console, ConsoleOutput, ConsoleUartWriter};
    //use crate::multicore::{create_multicore_runner, MulticoreProxy};
    use crate::memory::{init_heap, install_core0_stack_guard};

    type UartPinout = (Pin<Gpio0, FunctionUart, PullDown>, Pin<Gpio1, FunctionUart, PullDown>);

    #[shared]
    struct Shared {
        console: Console,
        console_writer: ConsoleUartWriter<UART0, UartPinout>,
    }

    #[local]
    struct Local {
        uart_rx: Reader<UART0, UartPinout>,
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            wfi()
        }
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        unsafe { init_heap() };
        install_core0_stack_guard();

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

        let sio = Sio::new(pac.SIO);

        let pins = Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let (mut uart_rx, uart_tx) = UartPeripheral::new(
            pac.UART0,
            (pins.gpio0.into_function(), pins.gpio1.into_function()),
            &mut pac.RESETS,
        ).enable(
            UartConfig::default(),
            clocks.peripheral_clock.freq(),
        ).unwrap().split();

        let aoc_runner = AocRunner::new();
        //let fifo = sio.fifo;
        //let multicore_runner = create_multicore_runner(fifo, aoc_runner);
        //debug!("multicore started");
        let mut commands = Commands::new();
        commands.add("aoc", aoc_runner);

        let console = Console::new(commands);
        let console_writer = ConsoleUartWriter(uart_tx);
        uart_rx.enable_rx_interrupt();

        (Shared {
            console,
            console_writer,
        }, Local {
            uart_rx,
        })
    }

    #[task(binds = UART0_IRQ, local = [uart_rx], shared = [console, console_writer])]
    fn read_uart(mut cx: read_uart::Context) {
        let uart_rx = cx.local.uart_rx;

        const CHUNK_SIZE : usize = 32;
        let mut buf = [0u8; CHUNK_SIZE];

        loop {
            if let Ok(count) = uart_rx.read_raw(&mut buf) {
                cx.shared.console.lock(|c| {
                    debug!("push {=[u8]:X}", &buf[..count]);
                    c.push(&buf[..count]);
                    while let Some(out) = c.next() {
                        debug!("get {=[u8]:X}", out);
                        cx.shared.console_writer.lock(|w| w.output(&out));
                    }
                });
            }
        }
    }
}
