#![allow(internal_features)]
#![no_std]
#![no_main]
#![feature(ascii_char)]
#![feature(panic_info_message)]

extern crate alloc;

use arch::asm::wfi;
use arch::interrupts::s_enable;
use arch::registers::tp;
use arch::registers::{sie, stvec};
use core::hint;
use core::ptr::addr_of;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::*;
use kernel::mem::paging::KERNEL_PAGE_TABLE;
use kernel::trampoline::trampoline;
use kernel::*;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section};

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let hart_id = tp::read();
    cprintln!("Started booting CPU #{}", hart_id);
    if hart_id == 0 {
        unsafe { init_kernel() };
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            wfi()
        }
    }
    if STARTED.load(Ordering::SeqCst) {
        cprintln!("Finished booting CPU #{}", hart_id);
        unsafe { mem::paging::set_current_page_table(addr_of!(KERNEL_PAGE_TABLE) as usize) };
        plic::init_plic_hart(hart_id);
        // Init kernel trap handler
        unsafe { stvec::write(trap::kernelvec as usize, stvec::TrapMode::Direct) };
        unsafe { s_enable() };
        loop {
            wfi();
        }
    } else {
        panic!("Something up with the ordering of instructions");
    }
}

/// Will be called when the kernel is booting, only from CPU#0
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_kernel() {
    cprintln!("End of kernel code : {:#x}", end_of_kernel_code_section());
    cprintln!("Trampoline frame   : {:#x}", trampoline as u64);
    cprintln!("End of kernel data : {:#x}", end_of_kernel_data_section());
    mem::init_kernel_allocator();
    mem::paging::init_kernel_page_table();
    mem::paging::set_current_page_table(addr_of!(KERNEL_PAGE_TABLE) as usize);
    cprintln!("Page Table has been initialized.");
    proc::init_procs();
    // Enable S-mode software, external and timer interrupts
    plic::init_plic_global();
    virtio::init_virtio();
    fs::init_files();
}
