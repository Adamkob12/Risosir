#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::Ordering;
use core::{panic::PanicInfo, sync::atomic::AtomicBool};
use kernel::arch::registers::{gpr::Tp, ReadFrom};
use kernel::console::init_console;
use kernel::mem::heap::init_heap;
use kernel::{cprintln, end_of_kernel_code_address};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cprintln!("Encountered Panic: {:#}", info);
    loop {}
}

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let cpuid = unsafe { Tp.read() };

    if cpuid == 0 {
        unsafe { init_kernel() };
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    }
    while !STARTED.load(Ordering::SeqCst) {
        // Wait for CPU #0 to set up the kernel properly
    }
    cprintln!("Hello from Hart #{}", cpuid);
    loop {}
}

/// Will be called when the kernel is booting, only from CPU#0
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_kernel() {
    init_console();
    cprintln!("\nBooting Kernel...");
    cprintln!("End of kernel code={:#x}", end_of_kernel_code_address());
    init_heap();
}
