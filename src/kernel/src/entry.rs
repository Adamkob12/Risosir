use core::arch::asm;

use crate::start::start;

/// The entry point for the OS, every CPU core starts here.
#[allow(unsafe_op_in_unsafe_fn)]
#[link_section = ".entry"]
#[no_mangle]
pub unsafe extern "C" fn _entry() -> ! {
    // Initialize the stack

    // Read the CPU id
    asm!("csrr a0, mhartid");
    // The stack grows down, so we need the first stack to be at 1 offset
    asm!("addi a0, a0, 1");
    asm!("la sp, GLOBAL_STACK"); // Defined in start.rs
    asm!("li a1, {}", const crate::param::STACK_SIZE);
    asm!("mul a1, a1, a0"); // Get the offset to the beginning of the stack
    asm!("add sp, sp, a1"); // Add it to the stack pointer

    // SP is now properly init, jump to start function
    start()
}
