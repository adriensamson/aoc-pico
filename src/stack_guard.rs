use core::ptr::addr_of_mut;

#[inline(always)]
fn install_stack_guard(stack_bottom: *mut usize) {
    let core = unsafe { rp_pico::pac::CorePeripherals::steal() };

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

extern "C" {
    static mut _stack_end: usize;
    static mut core1_stack_end: usize;
}

pub fn install_core0_stack_guard() {
    install_stack_guard(addr_of_mut!(_stack_end))
}

pub fn install_core1_stack_guard() {
    install_stack_guard(addr_of_mut!(core1_stack_end))
}
