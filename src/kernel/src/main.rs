#![feature(custom_test_frameworks)]
#![allow(internal_features)]
#![allow(static_mut_refs)]
#![test_runner(kernel::test_runner)]
#![no_std]
#![no_main]
#![feature(ascii_char)]
#![feature(panic_info_message)]

extern crate alloc;

use crate::fs::FILES;
use arch::asm::wfi;
use arch::gpr::tp;
use arch::register::{sie, stvec};
use core::hint;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::{fence, Ordering};
use elf_parse::parse_executable_file;
use kernel::mem::paging::KERNEL_PAGE_TABLE;
use kernel::trampoline::trampoline;
use kernel::*;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section};

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let cpuid = unsafe { tp::read() };
    cprintln!("Started booting CPU #{}", cpuid);
    if cpuid == 0 {
        unsafe { init_kernel(cpuid) };
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            wfi()
            // Wait for CPU #0 to set up the kernel properly
        }
    }
    if STARTED.load(Ordering::SeqCst) {
        cprintln!("Finished booting CPU #{}", cpuid);
        loop {
            hint::spin_loop();
        }
    } else {
        panic!("Something up with the ordering of instructions");
    }
}

/// Will be called when the kernel is booting, only from CPU#0
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_kernel(hart_id: usize) {
    cprintln!("End of kernel code : {:#x}", end_of_kernel_code_section());
    cprintln!("Trampoline frame   : {:#x}", trampoline as u64);
    cprintln!("End of kernel data : {:#x}", end_of_kernel_data_section());
    mem::init_kernel_allocator();
    mem::paging::init_kernel_page_table();
    mem::paging::set_current_page_table(&KERNEL_PAGE_TABLE);
    cprintln!("Page Table has been initialized.");
    proc::init_procs();
    // Enable S-mode software, external and timer interrupts
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();
    // Init kernel trap handler
    stvec::write(trap::kernelvec as usize, stvec::TrapMode::Direct);
    plic::init_plic_global();
    plic::init_plic_hart(hart_id);
    virtio::init_virtio();
    fs::init_files();
    FILES.lock().ls();
    let _ = parse_executable_file(&FILES.lock().copy_to_ram("ls").unwrap());
    // fence(Ordering::SeqCst);
    // enable();
    // fence(Ordering::SeqCst);
}
