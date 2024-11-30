#![no_std]
#![no_main]

extern crate alloc;

use cortex_m::peripheral::NVIC;
use panic_probe as _;
use defmt_rtt as _;

mod aoc;
mod multicore;
mod memory;

use defmt::debug;
use rp_pico::hal::{Sio, gpio::Pin, gpio::Pins, Watchdog, Clock};
use rp_pico::hal::clocks::init_clocks_and_plls;
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{FunctionUart, PullDown};
use rp_pico::hal::uart::{UartPeripheral, UartConfig, Reader};
use rp_pico::pac::UART0;
use rp_pico::pac::interrupt;
use rp_pico::{entry, XOSC_CRYSTAL_FREQ};
use aoc_pico::{give, give_away_cell, take};
use crate::aoc::AocRunner;
use aoc_pico::shell::{Commands, Console, ConsoleOutput, ConsoleUartWriter};
use crate::multicore::{create_multicore_runner};
use crate::memory::{init_heap, install_core0_stack_guard, read_sp};

type UartPinout = (Pin<Gpio0, FunctionUart, PullDown>, Pin<Gpio1, FunctionUart, PullDown>);

fn idle() -> ! {
    debug!("stack pointer: {:x}", read_sp());
    loop {
        cortex_m::asm::wfi()
    }
}

give_away_cell!(UART_RX: Reader<UART0, UartPinout>);
give_away_cell!(CONSOLE: Console);
give_away_cell!(CONSOLE_WRITER: ConsoleUartWriter<UART0, UartPinout>);

#[entry]
fn entry() -> ! {
    init();

    unsafe {
        NVIC::unmask(interrupt::UART0_IRQ);
    }

    idle()
}

fn init() {
    unsafe { init_heap() };
    install_core0_stack_guard();

    let mut pac = rp_pico::pac::Peripherals::take().unwrap();
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
    let fifo = sio.fifo;
    let multicore_runner = create_multicore_runner(fifo, aoc_runner);
    debug!("multicore started");
    let mut commands = Commands::new();
    commands.add("aoc", multicore_runner);

    let console = Console::new(commands);
    let console_writer = ConsoleUartWriter(uart_tx);
    uart_rx.enable_rx_interrupt();

    give!(UART_RX = uart_rx);
    give!(CONSOLE = console);
    give!(CONSOLE_WRITER = console_writer);
}

#[interrupt]
fn UART0_IRQ() {
    let uart_rx = take!(UART_RX);
    let console = take!(CONSOLE);
    let console_writer = take!(CONSOLE_WRITER);

    const CHUNK_SIZE : usize = 32;
    let mut buf = [0u8; CHUNK_SIZE];

    while let Ok(count) = uart_rx.read_raw(&mut buf) {
        console.push(&buf[..count]);
        for out in &mut *console {
            console_writer.output(&out);
        }
    }
}
