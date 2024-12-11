#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use crate::app::app::shared_resources::console_reader_that_needs_to_be_locked;
    use crate::memory::{init_heap, install_core0_stack_guard, read_sp};
    use crate::multicore::create_multicore_runner;
    use crate::{ConsoleDmaReader, ConsoleUartDmaWriter, MutexInputQueue, OutQueue};
    use aoc_pico::aoc::AocRunner;
    use aoc_pico::shell::{Commands, Console, InputParser};
    use cortex_m::peripheral::NVIC;
    use cortex_m::singleton;
    use defmt::debug;
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::dma::{DMAExt, SingleChannel, CH0, CH1};
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::timer::Alarm0;
    use rp_pico::hal::uart::{UartConfig, UartPeripheral};
    use rp_pico::hal::{gpio::Pin, gpio::Pins, Clock, Sio, Timer, Watchdog};
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
        console_reader: ConsoleDmaReader<CH1, UART0, UartPinout, Alarm0>,
    }

    #[local]
    struct Local {
        console: Console<InputParser<&'static MutexInputQueue>>,
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
        let console_writer = ConsoleUartDmaWriter::Ready(uart_tx, dma_chans.ch0);
        dma_chans.ch1.enable_irq1();
        let mut console_reader = ConsoleDmaReader::new(
            &*console_input,
            uart_rx,
            dma_chans.ch1,
            timer.alarm_0().unwrap(),
        );

        run_console::spawn().unwrap();

        (
            Shared {
                out_queue: OutQueue::new(),
                console_reader,
            },
            Local {
                console,
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

    #[task(binds = DMA_IRQ_1, shared = [console_reader])]
    fn dma_irq1(mut cx: dma_irq1::Context) {
        cx.shared.console_reader.lock(|console_reader| {
            console_reader.check_irq1();
            console_reader.on_finish();
        });
    }

    #[task(binds = UART0_IRQ, shared = [console_reader])]
    fn uart0(mut cx: uart0::Context) {
        cx.shared.console_reader.lock(|console_reader| {
            if let Ok(len) = console_reader.read_into() {
                if len >= 16 {
                    debug!("len: {} -> start", len);
                    console_reader.start().unwrap();
                }
            }
        });
    }

    #[task(binds = TIMER_IRQ_0, shared = [console_reader])]
    fn timer0(mut cx: timer0::Context) {
        cx.shared
            .console_reader
            .lock(|console_reader| console_reader.on_alarm());
    }
}
