#![no_std]
#![no_main]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use cortex_m::peripheral::NVIC;
use panic_probe as _;
use defmt_rtt as _;

mod multicore;
mod memory;

use defmt::debug;
use rp_pico::hal::{gpio::Pin, gpio::Pins, Clock, Sio, Watchdog};
use rp_pico::hal::clocks::init_clocks_and_plls;
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{FunctionUart, PullDown};
use rp_pico::hal::uart::{Reader, UartConfig, UartDevice, UartPeripheral, ValidUartPinout, Writer};
use rp_pico::pac::{UART0};
use rp_pico::pac::interrupt;
use rp_pico::{entry, XOSC_CRYSTAL_FREQ};
use rp_pico::hal::dma::{Channel, ChannelIndex, DMAExt, ReadTarget, SingleChannel, CH0};
use rp_pico::hal::dma::single_buffer::{Config, Transfer};
use aoc_pico::{borrow, give, give_away_cell, shared_cell, take};
use aoc_pico::aoc::AocRunner;
use aoc_pico::shell::{Commands, Console};
use crate::multicore::create_multicore_runner;
use crate::memory::{init_heap, install_core0_stack_guard, read_sp};

type UartPinout = (Pin<Gpio0, FunctionUart, PullDown>, Pin<Gpio1, FunctionUart, PullDown>);

give_away_cell!(UART_RX: Reader<UART0, UartPinout>);
give_away_cell!(CONSOLE: Console);
shared_cell!(OUT_DATA: OutQueue);
give_away_cell!(CONSOLE_WRITER: ConsoleUartDmaWriter<CH0, UART0, UartPinout>);

#[entry]
fn entry() -> ! {
    init();

    unsafe {
        NVIC::unmask(interrupt::UART0_IRQ);
        NVIC::unmask(interrupt::DMA_IRQ_0);
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
    uart_rx.enable_rx_interrupt();

    let mut dma_chans = pac.DMA.split(&mut pac.RESETS);
    dma_chans.ch0.enable_irq0();
    let console_writer = ConsoleUartDmaWriter::Ready(uart_tx, dma_chans.ch0);

    give!(UART_RX = uart_rx);
    give!(CONSOLE = console);
    give!(CONSOLE_WRITER = console_writer);
    give!(OUT_DATA = OutQueue::new());
}

fn idle() -> ! {
    debug!("stack pointer: {:x}", read_sp());

    loop {
        cortex_m::asm::wfi()
    }
}


#[interrupt]
fn UART0_IRQ() {
    let uart_rx = take!(UART_RX);
    let console = take!(CONSOLE);

    const CHUNK_SIZE : usize = 32;

    while let Some(vec) = read_into_vec(uart_rx, CHUNK_SIZE) {
        console.push(vec);
        for out in &mut *console {
            let need_pend = borrow!(OUT_DATA, |queue| queue.push(out));
            if need_pend {
                NVIC::pend(interrupt::DMA_IRQ_0);
            }
        }
    }
}

#[interrupt]
fn DMA_IRQ_0() {
    let console_writer = take!(CONSOLE_WRITER);
    console_writer.check_irq0();
    if let Some(data) = borrow!(OUT_DATA, |vec| vec.pop()) {
        console_writer.output(data);
    } else {
        console_writer.flush();
    }
}

fn read_into_vec<D: UartDevice, P: ValidUartPinout<D>>(uart: &Reader<D, P>, max_len: usize) -> Option<Vec<u8>> {
    let mut vec = Vec::with_capacity(max_len);
    let cap = vec.spare_capacity_mut();
    let buf = unsafe {core::slice::from_raw_parts_mut(cap.as_mut_ptr() as *mut u8, cap.len())};
    let len = uart.read_raw(buf).ok()?;
    unsafe { vec.set_len(vec.len() + len) };
    Some(vec)
}

struct OutQueue(VecDeque<Vec<u8>>);

impl OutQueue {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn push(&mut self, data: Vec<u8>) -> bool {
        self.0.push_back(data);
        self.0.len() == 1
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        self.0.pop_front()
    }
}

struct VecReadTarget(Vec<u8>);

unsafe impl ReadTarget for VecReadTarget {
    type ReceivedWord = u8;

    fn rx_treq() -> Option<u8> {
        None
    }

    fn rx_address_count(&self) -> (u32, u32) {
        (self.0.as_ptr() as u32, self.0.len() as u32)
    }

    fn rx_increment(&self) -> bool {
        true
    }
}

enum ConsoleUartDmaWriter<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>> {
    Ready(Writer<U, P>, Channel<D>),
    Transferring(Transfer<Channel<D>, VecReadTarget, Writer<U, P>>),
    Poisoned,
}

impl <D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>> ConsoleUartDmaWriter<D, U, P> {
    fn output(&mut self, line: Vec<u8>) {
        match core::mem::replace(self, Self::Poisoned) {
            Self::Ready(writer, ch) => {
                *self = Self::Transferring(Config::new(ch, VecReadTarget(line), writer).start())
            },
            Self::Transferring(transfer) => {
                let (ch, _, writer) = transfer.wait();
                *self = Self::Ready(writer, ch);
                self.output(line);
            },
            Self::Poisoned => unreachable!(),
        }
    }

    fn flush(&mut self) {
        match core::mem::replace(self, Self::Poisoned) {
            Self::Ready(writer, ch) => *self = Self::Ready(writer, ch),
            Self::Transferring(transfer) => {
                let (ch, _, writer) = transfer.wait();
                *self = Self::Ready(writer, ch);
            },
            Self::Poisoned => unreachable!(),
        }
    }

    fn check_irq0(&mut self) -> bool {
        match self {
            Self::Ready(_, ch) => ch.check_irq0(),
            Self::Transferring(transfer) => transfer.check_irq0(),
            Self::Poisoned => false,
        }
    }
}
