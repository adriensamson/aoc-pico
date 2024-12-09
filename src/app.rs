#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use crate::memory::{init_heap, install_core0_stack_guard, read_sp};
    use crate::multicore::create_multicore_runner;
    use crate::{ConsoleUartDmaWriter, MutexInputQueue, OutQueue};
    use aoc_pico::aoc::AocRunner;
    use aoc_pico::shell::{Commands, Console, InputParser};
    use cortex_m::peripheral::NVIC;
    use defmt::debug;
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::dma::{DMAExt, SingleChannel, CH0};
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::uart::{Reader, UartConfig, UartPeripheral};
    use rp_pico::hal::{gpio::Pin, gpio::Pins, Clock, Sio, Watchdog};
    use rp_pico::pac::interrupt;
    use rp_pico::pac::UART0;
    use rp_pico::XOSC_CRYSTAL_FREQ;

    type UartPinout = (
        Pin<Gpio0, FunctionUart, PullDown>,
        Pin<Gpio1, FunctionUart, PullDown>,
    );

    #[shared]
    struct Shared {
        out_queue: OutQueue,
    }

    #[local]
    struct Local {
        uart_rx: Reader<UART0, UartPinout>,
        console: Console<InputParser<MutexInputQueue>>,
        console_input: MutexInputQueue,
        console_writer: ConsoleUartDmaWriter<CH0, UART0, UartPinout>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        debug!("init");
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
        )
        .unwrap();

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

        let console_input = MutexInputQueue::new();
        let console = Console::new(InputParser::new(console_input.clone()), commands);
        uart_rx.enable_rx_interrupt();

        let mut dma_chans = pac.DMA.split(&mut pac.RESETS);
        dma_chans.ch0.enable_irq0();
        let console_writer = ConsoleUartDmaWriter::Ready(uart_tx, dma_chans.ch0);

        run_console::spawn().unwrap();

        (
            Shared {
                out_queue: OutQueue::new(),
            },
            Local {
                uart_rx,
                console,
                console_input,
                console_writer,
            },
        )
    }

    #[task(local = [console], shared = [out_queue])]
    async fn run_console(cx: run_console::Context) {
        debug!("stack pointer: {:x}", read_sp());
        let console = cx.local.console;
        let mut out_queue = cx.shared.out_queue;
        loop {
            for out in &mut *console {
                let need_pend = out_queue.lock(|queue| queue.push(out));
                if need_pend {
                    NVIC::pend(interrupt::DMA_IRQ_0);
                }
            }
        }
    }

    #[task(binds = UART0_IRQ, local = [uart_rx, console_input])]
    fn uart0_irq(cx: uart0_irq::Context) {
        let uart_rx = cx.local.uart_rx;
        let console_input = cx.local.console_input;

        while let Ok(_) = console_input.read_into(uart_rx) {}
    }

    #[task(binds = DMA_IRQ_0, local = [console_writer], shared = [out_queue])]
    fn dma_irq0(mut cx: dma_irq0::Context) {
        let console_writer = cx.local.console_writer;
        console_writer.check_irq0();
        if let Some(data) = cx.shared.out_queue.lock(|vec| vec.pop()) {
            console_writer.output(data);
        } else {
            console_writer.flush();
        }
    }
}
