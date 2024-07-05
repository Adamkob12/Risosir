use core::arch::asm;

/// The entry point for the OS, every CPU core starts here.
pub unsafe extern "C" fn _entry() -> ! {
    // Initialize the stack
    asm!("csrr a0, mhartid"); // Read the CPU id
    unreachable!("Jumped to start")
}
