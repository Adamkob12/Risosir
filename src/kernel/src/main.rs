#![allow(internal_features)]
#![no_std]
#![no_main]
#![feature(ascii_char)]
#![feature(panic_info_message)]

extern crate alloc;

use crate::files::FILES;
use arch::asm::wfi;
use arch::interrupts::s_disable;
use arch::registers::stvec;
use core::ptr::addr_of;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::*;
use elf_parse::parse_executable_file;
use kernel::mem::paging::KERNEL_PAGE_TABLE;
use kernel::trampoline::trampoline;
use kernel::*;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section};
use proc::{cpuid, proc, procs};
use scheduler::scheduler;

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let hart_id = cpuid();
    cprintln!("Started booting CPU #{}", hart_id);
    if hart_id == 0 {
        unsafe { init_kernel() };
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            wfi()
        }
        unsafe { mem::paging::set_current_page_table(addr_of!(KERNEL_PAGE_TABLE) as usize) };
        plic::init_plic_hart(hart_id);
        unsafe { stvec::write(trap::kernelvec as usize, stvec::TrapMode::Direct) };
    }

    scheduler(hart_id);

    #[allow(unreachable_code)]
    {
        unreachable!();
    }
}

/// Will be called when the kernel is booting, only from CPU#0
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_kernel() {
    cprintln!("End of kernel code : {:#x}", end_of_kernel_code_section());
    cprintln!("Trampoline frame   : {:#x}", trampoline as u64);
    cprintln!("End of kernel data : {:#x}", end_of_kernel_data_section());
    s_disable();
    mem::init_kernel_allocator();
    mem::paging::init_kernel_page_table();
    mem::paging::set_current_page_table(addr_of!(KERNEL_PAGE_TABLE) as usize);
    cprintln!("Page Table has been initialized.");
    proc::init_procs();
    assert_eq!(proc::procs().alloc_proc("kernel").unwrap(), 0);
    // Init kernel trap handler
    stvec::write(trap::kernelvec as usize, stvec::TrapMode::Direct);
    // Enable S-mode software, external and timer interrupts
    plic::init_plic_global();
    plic::init_plic_hart(0);
    virtio::init_virtio();
    files::init_files();

    let data = FILES.lock().copy_to_ram("print").unwrap();
    let exe = parse_executable_file(&data).unwrap();
    for _ in 0..30 {
        let pid = procs().alloc_proc("print").unwrap();
        proc(pid).activate(&exe);
    }

    // let data1 = FILES.lock().copy_to_ram("print").unwrap();
    // let pid1 = procs().alloc_proc("print1").unwrap();
    // let exe1 = parse_executable_file(&data1).unwrap();
    // proc(pid1).activate(exe1);
}
