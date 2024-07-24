#![feature(custom_test_frameworks)]
#![allow(internal_features)]
#![reexport_test_harness_main = "test_main"]
#![allow(static_mut_refs)]
#![test_runner(kernel::test_runner)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(riscv_ext_intrinsics)]

use core::arch::asm;
use core::hint;
use core::sync::atomic::Ordering;
use core::{panic::PanicInfo, sync::atomic::AtomicBool};
use kernel::arch::common::privilage::PrivLevel;
use kernel::arch::registers::csr::{Sie, Sstatus, Stvec};
use kernel::arch::registers::WriteInto;
use kernel::arch::registers::{gpr::Tp, ReadFrom};
use kernel::console::init_console;
use kernel::keyboard::{read_recent_input, KEYBOARD};
use kernel::mem::init_kernel_allocator;
use kernel::mem::paging::{init_kernel_page_table, set_current_page_table, KERNEL_PAGE_TABLE};
use kernel::plic::{init_plic_global, init_plic_hart};
use kernel::proc::init_procs;
use kernel::trampoline::trampoline;
use kernel::trap::{self, SupervisorInterrupt, _breakpoint, disable_interrupts, enable_interrupts};
use kernel::uart::UART;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section};

#[panic_handler]
#[cfg(not(feature = "test-kernel"))]
fn panic(info: &PanicInfo) -> ! {
    use core::hint;

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
        hint::spin_loop();
    }
}

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let hart_id = unsafe { Tp.read() };

    if hart_id == 0 {
        unsafe { init_kernel(hart_id) };
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    }
    while !STARTED.load(Ordering::SeqCst) {
        // unsafe { asm!("wfi") };
        // Wait for CPU #0 to set up the kernel properly
    }
    if STARTED.load(Ordering::SeqCst) {
        cprintln!("Hello from Hart #{}", hart_id);

        loop {
            hint::spin_loop();
        }
    } else {
        panic!("Something up with the ordering of instructions");
    }
}

/// Will be called when the kernel is booting, only from CPU#0
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_kernel(hart_id: u64) {
    init_console();
    cprintln!("\nBooting Kernel...");
    cprintln!("End of kernel code : {:#x}", end_of_kernel_code_section());
    cprintln!("Trampoline frame   : {:#x}", trampoline as u64);
    cprintln!("End of kernel data : {:#x}", end_of_kernel_data_section());
    init_kernel_allocator();
    init_kernel_page_table();
    set_current_page_table(&KERNEL_PAGE_TABLE);
    cprintln!("Page Table has been initialized.");
    init_procs();
    // Enable S-mode software, external and timer interrupts
    Sie.write(
        Sie.read()
            | SupervisorInterrupt::External.bitmask()
            | SupervisorInterrupt::Software.bitmask()
            | SupervisorInterrupt::Timer.bitmask(),
    );
    // Init kernel trap handler
    Stvec.write(trap::kernelvec as u64);
    init_plic_global();
    init_plic_hart(hart_id, PrivLevel::S);
}
