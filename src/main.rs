#![no_std]
#![no_main]

use panic_probe as _;
use defmt_rtt as _;

#[rtic::app(device = rp_pico::pac, peripherals = true)]
mod app {
    use defmt::{debug, info};
    use rp_pico::hal::{Sio, gpio::Pin, gpio::Pins, Watchdog, Clock, Timer};
    use rp_pico::hal::clocks::init_clocks_and_plls;
    use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
    use rp_pico::hal::gpio::{FunctionUart, PullDown};
    use rp_pico::hal::uart::{UartPeripheral, UartConfig, Enabled};
    use rp_pico::pac::UART0;
    use rp_pico::XOSC_CRYSTAL_FREQ;
    use embedded_hal::delay::DelayNs;

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
        loop {
            cx.shared.uart.lock(|uart| {
                let mut buf = [0u8; 64];
                match uart.read_raw(&mut buf) {
                    Ok(count) => {
                        uart.write_raw(&buf[..count]).unwrap();
                        info!("read: {}", core::str::from_utf8(&buf[..count]).unwrap());
                    },
                    Err(_) => {},
                }
            });
            cx.shared.timer.lock(|timer| {
                timer.delay_ms(10);
            });
        }
    }
}
