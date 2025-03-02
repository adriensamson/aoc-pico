use crate::dma::TimeoutDmaReader;
use crate::memory::{init_heap, install_core0_stack_guard, read_sp};
use crate::multicore::create_multicore_runner;
use crate::MutexInputQueue;
use alloc::boxed::Box;
use aoc_pico::aoc::AocRunner;
use aoc_pico::shell::{Commands, Console, InputParser};
use core::future::Future;
use cortex_m::asm::wfi;
use cortex_m::peripheral::NVIC;
use cortex_m::singleton;
use defmt::debug;
use rp2040_async::timer::AsyncAlarm;
use rp2040_hal::clocks::init_clocks_and_plls;
use rp2040_hal::dma::{DMAExt, SingleChannel};
use rp2040_hal::uart::{UartConfig, UartPeripheral};
use rp2040_hal::{gpio::Pins, Clock, Sio, Timer, Watchdog};
use rp2040_hal::pac::Interrupt;

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

#[rp2040_hal::entry]
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

fn init() -> [core::pin::Pin<Box<dyn Future<Output = ()>>>; 2] {
    debug!("init");
    unsafe { init_heap() };
    install_core0_stack_guard();

    let mut pac = rp2040_hal::pac::Peripherals::take().unwrap();
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

    let (uart_rx, uart_tx) = UartPeripheral::new(
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

    let mut dma_chans = pac.DMA.split(&mut pac.RESETS);
    dma_chans.ch0.enable_irq0();

    dma_chans.ch1.enable_irq1();
    let double_dma = TimeoutDmaReader::<_, _, _, _, 512>::new(
        dma_chans.ch1,
        AsyncAlarm::new(timer.alarm_0().unwrap()),
        uart_rx,
        |v| console_input.push(v),
    );

    [
        Box::pin(crate::run_console(console, uart_tx, dma_chans.ch0)),
        Box::pin(double_dma.run()),
    ]
}
