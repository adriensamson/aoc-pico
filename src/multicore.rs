use alloc::boxed::Box;
use alloc::string::String;
use core::ptr::addr_of;
use defmt::debug;
use rp_pico::hal::Sio;
use rp_pico::hal::sio::SioFifo;
use rp_pico::pac::Peripherals;
use crate::aoc::AocRunner;
use crate::console::ConsoleRunner;
use crate::memory::install_core1_stack_guard;

pub struct MulticoreProxy {
    pub fifo: SioFifo,
}

impl ConsoleRunner for MulticoreProxy {
    type Output<'a> = MulticoreReceiver<'a>;

    fn push_line(&mut self, line: String) -> Self::Output<'_> {
        let boxed = Box::new(line);
        let ptr = Box::into_raw(boxed);
        debug!("core0: write {:X}", ptr);
        self.fifo.write_blocking(ptr as u32);
        MulticoreReceiver::new(&mut self.fifo)
    }
}

pub struct MulticoreReceiver<'a> {
    fifo: &'a mut SioFifo,
    finished: bool,
}

impl<'a> MulticoreReceiver<'a> {
    fn new(fifo: &'a mut SioFifo) -> Self {
        Self {
            fifo,
            finished: false,
        }
    }
}

impl<'a> Iterator for MulticoreReceiver<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let addr = self.fifo.read_blocking() as *mut Option<String>;
        debug!("core0: read {:X}", addr);
        let item  = unsafe { Box::from_raw(addr) };
        if item.is_none() {
            self.finished = true;
        }
        *item
    }
}

pub(crate) struct MulticoreRunner<R: ConsoleRunner> {
    fifo: SioFifo,
    inner: R,
}

impl <R: ConsoleRunner> MulticoreRunner<R> {
    pub(crate) fn new(fifo: SioFifo, inner: R) -> Self {
        Self { fifo, inner }
    }

    pub(crate) fn run(&mut self) -> ! {
        loop {
            let addr = self.fifo.read_blocking() as *mut String;
            debug!("core1 read {:X}", addr);
            let line = unsafe { Box::from_raw(addr) };
            for res in self.inner.push_line(*line) {
                let boxed = Box::new(res);
                debug!("core1 write some");
                self.fifo.write_blocking(Box::into_raw(boxed) as u32);
            }
            let none = Box::new(Option::<String>::None);
            debug!("core1 write none");
            self.fifo.write_blocking(Box::into_raw(none) as u32);
        }
    }
}

static mut RUNNER : Option<AocRunner> = None;
pub fn create_multicore_runner(mut fifo: SioFifo, runner: AocRunner) -> MulticoreProxy {
    unsafe { RUNNER = Some(runner); }
    extern "C" fn core1_entry() {
        debug!("core1: entry");
        install_core1_stack_guard();
        let runner = unsafe { RUNNER.take() }.unwrap();
        let fifo = unsafe { Sio::new(Peripherals::steal().SIO).fifo };
        let mut multicore_runner = MulticoreRunner::new(fifo, runner);
        debug!("core1: run!");
        multicore_runner.run()
    }
    extern "C" {
        static core1_stack_start: u32;
    }
    start_core1(&mut fifo, addr_of!(core1_stack_start) as *mut u32, core1_entry as *const _);

    MulticoreProxy { fifo }
}

pub fn start_core1(fifo: &mut SioFifo, stack_ptr: *mut u32, entry: *const fn()) {
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
            }
        }
        if seq >= cmd_seq.len() {
            break;
        }
    }
}
