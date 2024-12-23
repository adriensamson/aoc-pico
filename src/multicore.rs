use crate::memory::install_core1_stack_guard;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use aoc_pico::shell::{Command, RunningCommand};
use core::cell::UnsafeCell;
use core::future::{ready, Future};
use core::pin::Pin;
use cortex_m::asm::wfe;
use cortex_m::singleton;
use critical_section::Mutex;
use defmt::debug;
use rp_pico::hal::sio::SioFifo;
use rp_pico::hal::Sio;
use rp_pico::pac::Peripherals;

pub struct MulticoreProxy {
    pub fifo: &'static mut SioFifo,
}

impl Command for MulticoreProxy {
    fn exec(&mut self, args: Vec<String>, input: Vec<String>) -> Box<dyn RunningCommand> {
        let boxed = Box::new((args, input));
        let ptr = Box::into_raw(boxed);
        debug!("core0: write {:X}", ptr);
        self.fifo.write_blocking(ptr as u32);
        let fifo: &'static mut SioFifo = unsafe { core::mem::transmute(&mut self.fifo) };
        Box::new(MulticoreReceiver::new(fifo))
    }
}

pub struct MulticoreReceiver {
    fifo: &'static mut SioFifo,
    finished: bool,
}

impl MulticoreReceiver {
    fn new(fifo: &'static mut SioFifo) -> Self {
        Self {
            fifo,
            finished: false,
        }
    }
}

impl RunningCommand for MulticoreReceiver {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<String>>>> {
        if self.finished {
            return Box::pin(ready(None));
        }
        let addr = self.fifo.read_blocking() as *mut Option<String>; // TODO: async
        debug!("core0: read {:X}", addr);
        let item = unsafe { Box::from_raw(addr) };
        if item.is_none() {
            self.finished = true;
        }
        Box::pin(ready(*item))
    }
}

struct MulticoreRunner<C: Command> {
    fifo: SioFifo,
    inner: C,
}

impl<C: Command + 'static> MulticoreRunner<C> {
    fn new(fifo: SioFifo, inner: C) -> Self {
        Self { fifo, inner }
    }

    fn run(mut self) -> ! {
        myasync::Executor::new(
            [Box::pin(async move {
                loop {
                    let addr = self.fifo.read_blocking() as *mut (Vec<String>, Vec<String>);
                    debug!("core1 read {:X}", addr);
                    let line = unsafe { Box::from_raw(addr) };
                    let (args, input) = *line;
                    let mut running = self.inner.exec(args, input);
                    while let Some(res) = running.next().await {
                        let boxed = Box::new(Some(res));
                        debug!("core1 write some");
                        self.fifo.write_blocking(Box::into_raw(boxed) as u32);
                    }
                    let none = Box::new(Option::<String>::None);
                    debug!("core1 write none");
                    self.fifo.write_blocking(Box::into_raw(none) as u32);
                }
            })],
            wfe,
        )
        .run()
    }
}

pub fn create_multicore_runner(
    fifo0: SioFifo,
    runner: impl Command + Send + 'static,
) -> MulticoreProxy {
    let f = move || {
        let fifo1 = unsafe { Sio::new(Peripherals::steal().SIO).fifo };
        let multicore_runner = MulticoreRunner::new(fifo1, runner);
        debug!("core1: run!");
        multicore_runner.run()
    };
    let fifo0 = singleton!(: SioFifo = fifo0).unwrap();
    start_core1_with_fn(fifo0, f);
    MulticoreProxy { fifo: fifo0 }
}

type MutexCell<T> = Mutex<UnsafeCell<T>>;

static CORE1_FN: MutexCell<Option<Box<dyn FnOnce() + Send>>> = Mutex::new(UnsafeCell::new(None));
extern "C" fn core1_entry() {
    debug!("core1: entry");
    install_core1_stack_guard();
    let f = critical_section::with(|cs| {
        let cell = CORE1_FN.borrow(cs);
        unsafe { (*cell.get()).take().unwrap() }
    });
    f()
}
extern "C" {
    static core1_stack_start: u32;
}

fn start_core1_with_fn(fifo: &mut SioFifo, f: impl FnOnce() + Send + 'static) {
    critical_section::with(|cs| {
        let cell = CORE1_FN.borrow(cs);
        unsafe {
            *cell.get() = Some(Box::new(f));
        }
    });
    start_core1(fifo, &raw const core1_stack_start, core1_entry as *const _);
}

fn start_core1(fifo: &mut SioFifo, stack_ptr: *const u32, entry: *const fn()) {
    // Reset core1
    let psm = unsafe { Peripherals::steal().PSM };
    psm.frce_off().modify(|_, w| w.proc1().set_bit());
    while !psm.frce_off().read().proc1().bit_is_set() {
        cortex_m::asm::nop();
    }
    psm.frce_off().modify(|_, w| w.proc1().clear_bit());

    let vector_table = unsafe { Peripherals::steal().PPB.vtor().read().bits() };
    let cmd_seq = [
        0,
        0,
        1,
        vector_table as usize,
        stack_ptr as usize,
        entry as usize,
    ];

    let mut seq = 0;
    let mut fails = 0;
    loop {
        let cmd = cmd_seq[seq] as u32;
        if cmd == 0 {
            fifo.drain();
            cortex_m::asm::sev();
        }
        debug!("core1 start seq: send {:X}", cmd);
        fifo.write_blocking(cmd);
        let response = fifo.read_blocking();
        if cmd == response {
            seq += 1;
        } else {
            seq = 0;
            fails += 1;
            if fails > 16 {
                panic!("Cannot start core1");
            }
        }
        if seq >= cmd_seq.len() {
            break;
        }
    }
}
