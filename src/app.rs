use crate::dma::DoubleChannelReader;
use crate::memory::{init_heap, install_core0_stack_guard, read_sp};
use crate::multicore::create_multicore_runner;
use crate::{ConsoleDmaReader, MutexInputQueue};
use alloc::boxed::Box;
use core::future::Future;
use aoc_pico::aoc::AocRunner;
use aoc_pico::shell::{Commands, Console, InputParser};
use cortex_m::asm::wfi;
use cortex_m::peripheral::NVIC;
use cortex_m::singleton;
use defmt::debug;
use rp2040_async::{DmaIrq0Handler, DmaIrq1Handler, TimerIrq0Handler, Uart0IrqHandler};
use embed_init::{give, give_away_cell, take};
use rp_pico::hal::clocks::init_clocks_and_plls;
use rp_pico::hal::dma::{DMAExt, SingleChannel, CH1, CH2};
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{FunctionUart, PullDown};
use rp_pico::hal::timer::Alarm0;
use rp_pico::hal::uart::{UartConfig, UartPeripheral};
use rp_pico::hal::{gpio::Pin, gpio::Pins, Clock, Sio, Timer, Watchdog};
use rp_pico::pac::UART0;
use rp_pico::pac::{interrupt, Interrupt};
use rp_pico::XOSC_CRYSTAL_FREQ;

type UartPinout = (
    Pin<Gpio0, FunctionUart, PullDown>,
    Pin<Gpio1, FunctionUart, PullDown>,
);

#[give_away_cell]
static console_reader: ConsoleDmaReader<CH1, CH2, Alarm0, UART0, UartPinout>;

#[rp_pico::entry]
fn entry() -> ! {
    let futures = init();

    unsafe {
        NVIC::unmask(Interrupt::DMA_IRQ_0);
        NVIC::unmask(Interrupt::DMA_IRQ_1);
        NVIC::unmask(Interrupt::UART0_IRQ);
        NVIC::unmask(Interrupt::TIMER_IRQ_0);
    }

    debug!("stack pointer: {:x}", read_sp());
    myasync::Executor::new(futures, wfi).run()
}

fn init() -> [core::pin::Pin<Box<dyn Future<Output=()>>>; 2] {
    debug!("init");
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
    )
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

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
    )
    .enable(UartConfig::default(), clocks.peripheral_clock.freq())
    .unwrap()
    .split();

    let aoc_runner = AocRunner::new();
    let fifo = sio.fifo;
    let multicore_runner = create_multicore_runner(fifo, aoc_runner);
    debug!("multicore started");
    let mut commands = Commands::new();
    commands.add("aoc", multicore_runner);

    let console_input = singleton!(: MutexInputQueue = MutexInputQueue::new()).unwrap();
    let console = Console::new(InputParser::new(&*console_input), commands);
    uart_rx.enable_rx_interrupt();

    let mut dma_chans = pac.DMA.split(&mut pac.RESETS);
    dma_chans.ch0.enable_irq0();

    dma_chans.ch1.enable_irq1();
    dma_chans.ch2.enable_irq1();
    let double_dma = DoubleChannelReader::new(
        dma_chans.ch1,
        dma_chans.ch2,
        timer.alarm_0().unwrap(),
        uart_rx,
    );
    give!(console_reader = ConsoleDmaReader::new(&*console_input, double_dma));

    [
        Box::pin(crate::run_console(console, uart_tx, dma_chans.ch0, &DMA_IRQ_0_HANDLER)),
        Box::pin(run_uart_reader()),
    ]
}

async fn run_uart_reader() {
    let console_reader = take!(console_reader);
    console_reader.run(&UART0_IRQ_HANDLER, &TIMER_IRQ_0_HANDLER, &DMA_IRQ_1_HANDLER).await;
}

pub static DMA_IRQ_0_HANDLER: DmaIrq0Handler = DmaIrq0Handler::new();

#[interrupt]
fn DMA_IRQ_0() {
    DMA_IRQ_0_HANDLER.on_irq();
}

pub static DMA_IRQ_1_HANDLER: DmaIrq1Handler = DmaIrq1Handler::new();

#[interrupt]
fn DMA_IRQ_1() {
    DMA_IRQ_1_HANDLER.on_irq();
}

pub static UART0_IRQ_HANDLER: Uart0IrqHandler<UartPinout> = Uart0IrqHandler::new();

#[interrupt]
fn UART0_IRQ() {
    UART0_IRQ_HANDLER.on_irq();
}

pub static TIMER_IRQ_0_HANDLER: TimerIrq0Handler = TimerIrq0Handler::new();

#[interrupt]
fn TIMER_IRQ_0() {
    TIMER_IRQ_0_HANDLER.on_irq();
}
