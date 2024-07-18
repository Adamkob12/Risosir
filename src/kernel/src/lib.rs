#![feature(asm_const)]
#![allow(static_mut_refs)]
#![feature(fn_align)]
#![feature(panic_info_message)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![no_std]
#![no_main]

pub const TESTS: &[&dyn Test] = &[&trivial_test];

fn trivial_test() {
    assert_eq!(1, 1);
}

pub mod arch;
pub mod console;
pub mod entry;
pub mod file;
pub mod kernelvec;
pub mod mem;
pub mod param;
pub mod proc;
pub mod start;
pub mod trap;
pub mod uart;

use arch::registers::{gpr::Tp, ReadFrom};
pub use console::*;
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, Ordering},
};
use mem::{
    init_kernel_allocator,
    paging::{init_kernel_page_table, set_current_page_table, KERNEL_PAGE_TABLE},
};

extern "C" {
    fn end();
    fn etext();
}

/// *Includes* the kernel's stack(s) (one stack per hart)
pub fn end_of_kernel_data_section() -> usize {
    end as usize
}

/// *Doesn't Include* kernel's stack(s), only includes "text" memory section
pub fn end_of_kernel_code_section() -> usize {
    etext as usize
}

#[panic_handler]
#[cfg(feature = "test-kernel")]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use uart::UART;

    unsafe {
        // UART.force_unlock();
        // CONSOLE.force_unlock();
        let mut uart = UART.lock();
        uart.write_chars(b"[TEST FAILED]");
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
    // Set up allocations & paging
    init_kernel_allocator();
    init_kernel_page_table();
    #[allow(static_mut_refs)]
    set_current_page_table(&KERNEL_PAGE_TABLE);
    panic!("ARHARH");
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
