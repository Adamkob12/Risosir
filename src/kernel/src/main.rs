#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::asm;
use core::sync::atomic::Ordering;
use core::{panic::PanicInfo, sync::atomic::AtomicBool};
use kernel::arch::registers::{gpr::Tp, ReadFrom};
use kernel::console::init_console;
use kernel::mem::init_kernel_allocator;
use kernel::mem::paging::{init_kernel_page_table, set_current_page_table, KERNEL_PAGE_TABLE};
use kernel::uart::UART;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section, CONSOLE};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        // UART.force_unlock();
        // CONSOLE.force_unlock();
        let mut uart = UART.lock();
        uart.write_chars(b"\nPANIC: ");
        if let Some(msg) = info.message().as_str() {
            uart.write_chars(msg.as_bytes());
        } else {
            uart.write_chars(b"X");
        }
        uart.write_chars(b"\nFILE: ");
        uart.write_chars(info.location().unwrap().file().as_bytes());
        uart.write_chars(b"\nLINE: ");
        let mut line = info.location().unwrap().line();
        while line != 0 {
            uart.put_char((line % 10) as u8 + 48);
            line /= 10;
        }
        uart.put_char(b'\n');
    };
    cprintln!(
        "Encountered Panic (tp={}): {:#}",
        unsafe { Tp.read() },
        info
    );
    loop {
        unsafe { asm!("wfi") };
    }
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
    cprintln!("Hello from Hart #{}", cpuid,);
    loop {
        unsafe { asm!("wfi") };
    }
}

/// Will be called when the kernel is booting, only from CPU#0
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_kernel() {
    init_console();
    cprintln!("\nBooting Kernel...");
    cprintln!("End of kernel code={:#x}", end_of_kernel_code_section());
    cprintln!("End of kernel data={:#x}", end_of_kernel_data_section());
    // Set up allocations & paging
    init_kernel_allocator();
    init_kernel_page_table();
    set_current_page_table(&KERNEL_PAGE_TABLE);
    panic!("ARHARH");
    cprintln!("Hi, does paging work?");
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
}
