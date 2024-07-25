use core::arch::asm;

use crate::cprint;

#[repr(align(16))]
#[allow(unsafe_op_in_unsafe_fn)]
#[no_mangle]
pub unsafe fn timervec() {
    asm!("csrrw a0, mscratch, a0");
    // a0 contains the address of [`DataToHandleTimerInt`], with a dedicated 3 places to save the value of registers.
    asm!("sd a1, 0(a0)");
    asm!("sd a2, 8(a0)");
    asm!("sd a3, 16(a0)");

    // 24(a0) will contain the address of the mtimecmp register
    // 32(a0) will contain the desired amount of cycles to pass before the next timer interrupt
    asm!("ld a1, 24(a0)");
    asm!("ld a2, 32(a0)");
    asm!("ld a3, 0(a1)");
    asm!("add a3, a3, a2"); // Increment mtimecmp by the desired interval
    asm!("sd a3, 0(a1)"); // Store the value back in the mtimecmp register

    asm! {
        "li a1, 2",
        "csrw sip, a1"
    };

    // Restore the registers
    asm!("ld a3, 16(a0)");
    asm!("ld a2, 8(a0)");
    asm!("ld a1, 0(a0)");
    asm!("csrrw a0, mscratch, a0");

    asm!("mret");
}
