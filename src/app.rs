use crate::dma::DoubleChannelReader;
use crate::memory::{init_heap, install_core0_stack_guard, read_sp};
use crate::multicore::create_multicore_runner;
use crate::{ConsoleDmaReader, ConsoleUartDmaWriter, MutexInputQueue};
use alloc::boxed::Box;
use aoc_pico::aoc::AocRunner;
use aoc_pico::shell::{Commands, Console, InputParser};
use cortex_m::asm::wfi;
use cortex_m::peripheral::NVIC;
use cortex_m::singleton;
use defmt::debug;
use rp2040_async::DmaIrq0Listener;
use embed_init::{borrow, give, give_away_cell, shared_cell, take};
use rp_pico::hal::clocks::init_clocks_and_plls;
use rp_pico::hal::dma::{ChannelIndex, DMAExt, SingleChannel, CH0, CH1, CH2};
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{FunctionUart, PullDown};
use rp_pico::hal::timer::Alarm0;
use rp_pico::hal::uart::{UartConfig, UartDevice, UartPeripheral, ValidUartPinout};
use rp_pico::hal::{gpio::Pin, gpio::Pins, Clock, Sio, Timer, Watchdog};
use rp_pico::pac::UART0;
use rp_pico::pac::{interrupt, Interrupt};
use rp_pico::XOSC_CRYSTAL_FREQ;

type UartPinout = (
    Pin<Gpio0, FunctionUart, PullDown>,
    Pin<Gpio1, FunctionUart, PullDown>,
);

#[shared_cell]
static console_reader: ConsoleDmaReader<CH1, CH2, Alarm0, UART0, UartPinout>;

#[give_away_cell]
static console: Console<InputParser<&'static MutexInputQueue>>;
#[give_away_cell]
static console_writer: ConsoleUartDmaWriter<CH0, UART0, UartPinout>;

#[rp_pico::entry]
fn entry() -> ! {
    init();

    unsafe {
        NVIC::unmask(Interrupt::DMA_IRQ_0);
        NVIC::unmask(Interrupt::DMA_IRQ_1);
        NVIC::unmask(Interrupt::UART0_IRQ);
        NVIC::unmask(Interrupt::TIMER_IRQ_0);
    }

    main()
}

fn init() {
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
    give!(console = Console::new(InputParser::new(&*console_input), commands));
    uart_rx.enable_rx_interrupt();

    let mut dma_chans = pac.DMA.split(&mut pac.RESETS);
    dma_chans.ch0.enable_irq0();
    give!(console_writer = ConsoleUartDmaWriter::Ready(uart_tx, dma_chans.ch0));

    dma_chans.ch1.enable_irq1();
    dma_chans.ch2.enable_irq1();
    let double_dma = DoubleChannelReader::new(
        dma_chans.ch1,
        dma_chans.ch2,
        timer.alarm_0().unwrap(),
        uart_rx,
    );
    give!(console_reader = ConsoleDmaReader::new(&*console_input, double_dma));
}

fn main() -> ! {
    debug!("stack pointer: {:x}", read_sp());
    let console = take!(console);
    let console_writer = take!(console_writer);
    myasync::Executor::new([Box::pin(run_console(console, console_writer))], wfi).run();
}

async fn run_console<D: ChannelIndex, U: UartDevice, P: ValidUartPinout<U>>(
    console: &mut Console<InputParser<&'static MutexInputQueue>>,
    console_writer: &mut ConsoleUartDmaWriter<D, U, P>,
) {
    loop {
        let out = console.next_wait().await;
        console_writer.output(out);
        DMA_IRQ_0_LISTENER.wait_done(D::id() as usize).await;
    }
}

pub static DMA_IRQ_0_LISTENER : DmaIrq0Listener = DmaIrq0Listener::new();

#[interrupt]
fn DMA_IRQ_0() {
    DMA_IRQ_0_LISTENER.on_irq();
}

#[interrupt]
fn DMA_IRQ_1() {
    borrow!(console_reader, |console_reader| {
        let _ = console_reader.dma.check_irq1();
        let _ = console_reader.on_dma_irq();
    });
}

#[interrupt]
fn UART0_IRQ() {
    borrow!(console_reader, |console_reader| {
        if let Ok(len) = console_reader.read_into() {
            if len >= 16 {
                console_reader.start().unwrap();
            }
        }
    });
}

#[interrupt]
fn TIMER_IRQ_0() {
    let _ = borrow!(console_reader, |console_reader| console_reader.on_alarm());
}
