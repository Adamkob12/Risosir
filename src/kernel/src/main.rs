#![no_std]
#![no_main]

use core::sync::atomic::Ordering;
use core::{panic::PanicInfo, sync::atomic::AtomicBool};
use kernel::arch::registers::{gpr::Tp, ReadFrom};
use kernel::uart::{THR, UART};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { UART.lock().write_to_register::<THR>(b'P') };
    loop {}
}

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
pub extern "C" fn __main() -> ! {
    let f: extern "C" fn() -> ! = main;
    f()
}

extern "C" fn main() -> ! {
    let cpuid = unsafe { Tp.read() };
    unsafe { UART.lock().write_to_register::<THR>(b'M') };
    if cpuid == 0 {
        init_kernel();
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    }
    while !STARTED.load(Ordering::SeqCst) {
        // Wait for CPU #0 to set up the kernel properly
    }
    loop {}
}

/// Will be called when the kernel is booting, only from CPU#0
fn init_kernel() {}
