use cortex_m_rt::heap_start;
use crate::debug;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

unsafe extern "C" {
    static _stack_end: usize;
    static core1_stack_end: usize;
}

pub(crate) unsafe fn init_heap() {
    let heap_bottom = heap_start() as usize;
    let heap_top = &raw const core1_stack_end as usize;
    let heap_size = heap_top - heap_bottom;
    debug!("HEAP size: {}k", heap_size / 1024);
    unsafe { HEAP.init(heap_bottom, heap_size) }
}

#[allow(unused)]
pub fn debug_heap_size(place: &str) {
    let used = HEAP.used();
    let free = HEAP.free();
    let percent = 100 * used / (used + free);
    debug!("HEAP usage as {}: {}k ({}%)", place, used / 1024, percent);
}

#[inline(always)]
fn install_stack_guard(stack_bottom: usize) {
    debug!("Installing stack guard at {:X}", stack_bottom);
    let core = unsafe { rp2040_hal::pac::CorePeripherals::steal() };

    // Trap if MPU is already configured
    if core.MPU.ctrl.read() != 0 {
        cortex_m::asm::udf();
    }

    // The minimum we can protect is 32 bytes on a 32 byte boundary, so round up which will
    // just shorten the valid stack range a tad.
    let addr = (stack_bottom as u32 + 31) & !31;
    // Mask is 1 bit per 32 bytes of the 256 byte range... clear the bit for the segment we want
    let subregion_select = 0xff ^ (1 << ((addr >> 5) & 7));
    unsafe {
        core.MPU.ctrl.write(5); // enable mpu with background default map
        const RBAR_VALID: u32 = 0x10;
        core.MPU.rbar.write((addr & !0xff) | RBAR_VALID);
        core.MPU.rasr.write(
            1 // enable region
                | (0x7 << 1) // size 2^(7 + 1) = 256
                | (subregion_select << 8)
                | 0x10000000, // XN = disable instruction fetch; no other bits means no permissions
        );
    }
}

pub(crate) fn install_core0_stack_guard() {
    install_stack_guard(&raw const _stack_end as usize)
}

pub(crate) fn install_core1_stack_guard() {
    install_stack_guard(&raw const core1_stack_end as usize)
}

pub fn read_sp() -> usize {
    let i: usize;
    unsafe {
        core::arch::asm!(
        "mov {0}, sp",
        out(reg) i,
        )
    }
    i
}
