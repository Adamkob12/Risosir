#![feature(asm_const)]
#![allow(static_mut_refs)]
#![feature(naked_functions)]
#![feature(new_uninit)]
#![feature(fn_align)]
#![feature(panic_info_message)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![no_std]
#![no_main]

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::hint;

    // unsafe {
    //     UART.force_unlock();
    //     CONSOLE.force_unlock();
    //     let mut uart = UART.lock();

    //     uart.write_chars(b"\nPANIC: ");
    //     if let Some(msg) = info.message().as_str() {
    //         uart.write_chars(msg.as_bytes());
    //     } else {
    //         uart.write_chars(b"X");
    //     }
    //     uart.write_chars(b"\nFILE: ");
    //     uart.write_chars(info.location().unwrap().file().as_bytes());
    //     uart.write_chars(b"\nLINE: ");
    //     let mut line = info.location().unwrap().line();
    //     while line != 0 {
    //         uart.put_char((line % 10) as u8 + 48);
    //         line /= 10;
    //     }
    //     uart.put_char(b'\n');
    // };

    cprintln!(
        "Encountered Panic (tp={}): {:#}",
        unsafe { Tp.read() },
        info
    );
    loop {
        hint::spin_loop();
    }
}

pub const TESTS: &[&dyn Test] = &[&trivial_test];

fn trivial_test() {
    assert_eq!(1, 1);
}

extern crate alloc;

pub mod arch;
pub mod console;
pub mod entry;
pub mod file;
pub mod fs;
pub mod kernelvec;
pub mod keyboard;
pub mod mem;
pub mod param;
pub mod plic;
pub mod proc;
pub mod spinlock;
pub mod start;
pub mod trampoline;
pub mod trap;
pub mod uart;
pub mod virtio;

use arch::registers::{gpr::Tp, ReadFrom};
pub use console::*;
use core::{
    arch::asm,
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};
use mem::{
    init_kernel_allocator,
    paging::{init_kernel_page_table, set_current_page_table, KERNEL_PAGE_TABLE},
};
use proc::init_procs;
use uart::UART;

extern "C" {
    fn end();
    fn etext();
}

/// *Includes* the kernel's stack(s) (one stack per hart)
pub fn end_of_kernel_data_section() -> usize {
    end as usize
}

/// *Doesn't Include* kernel's stack(s), only includes "text" memory section
/// (includes trampoline)
pub fn end_of_kernel_code_section() -> usize {
    etext as usize
}

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "test_kernel"]
pub extern "C" fn test_kernel() -> ! {
    let cpuid = unsafe { Tp.read() };

    if cpuid == 0 {
        unsafe { init_kernel() };
        // The kernel has officially booted
        test_runner(TESTS);
        cprintln!("\n");
        cprintln!("All Tests Passed.");
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
    init_kernel_allocator();
    init_kernel_page_table();
    set_current_page_table(&KERNEL_PAGE_TABLE);
    init_procs();
}

pub fn test_runner(tests: &[&dyn Test]) {
    cprintln!();
    cprintln!("~~~~~~~~~ Running {} tests ~~~~~~~~~", tests.len());
    cprintln!();
    for test in tests {
        test.execute_test();
    }
    cprintln!();
    cprintln!("~~~~~~~~~ All tests passed ~~~~~~~~~");
}

pub trait Test {
    fn name(&self) -> &'static str;
    fn run(&self);
    fn execute_test(&self) {
        cprintln!("RUNNING TEST: {}...", self.name());
        self.run();
        cprintln!("[ok]");
    }
}

impl<T> Test for T
where
    T: Fn(),
{
    fn name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    fn run(&self) {
        self();
    }
}
