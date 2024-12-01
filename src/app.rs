#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use cortex_m::peripheral::NVIC;
    use defmt::debug;
    use rp_pico::hal::{gpio::Pin, gpio::Pins, Clock, Sio, Watchdog};
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::uart::{Reader, UartConfig, UartPeripheral};
    use rp_pico::pac::UART0;
    use rp_pico::pac::interrupt;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    use rp_pico::hal::dma::{DMAExt, SingleChannel, CH0};
    use aoc_pico::aoc::AocRunner;
    use aoc_pico::shell::{Commands, Console};
    use crate::multicore::create_multicore_runner;
    use crate::memory::{init_heap, install_core0_stack_guard, read_sp};
    use crate::{read_into_vec, ConsoleUartDmaWriter, OutQueue};

    type UartPinout = (Pin<Gpio0, FunctionUart, PullDown>, Pin<Gpio1, FunctionUart, PullDown>);

    #[shared]
    struct Shared {
        out_queue: OutQueue,
    }

    #[local]
    struct Local {
        uart_rx: Reader<UART0, UartPinout>,
        console: Console,
        console_writer: ConsoleUartDmaWriter<CH0, UART0, UartPinout>,
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

        (Shared {
            out_queue: OutQueue::new(),
        }, Local {
            uart_rx,
            console,
            console_writer,
        })
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        debug!("stack pointer: {:x}", read_sp());

        loop {
            cortex_m::asm::wfi()
        }
    }


    #[task(binds = UART0_IRQ, local = [uart_rx, console], shared = [out_queue])]
    fn uart0_irq(mut cx: uart0_irq::Context) {
        let uart_rx = cx.local.uart_rx;
        let console = cx.local.console;

        const CHUNK_SIZE: usize = 32;

        while let Some(vec) = read_into_vec(uart_rx, CHUNK_SIZE) {
            console.push(vec);
            for out in &mut *console {
                let need_pend = cx.shared.out_queue.lock(|queue| queue.push(out));
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
}
