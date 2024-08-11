#![allow(internal_features)]
#![no_std]
#![no_main]
#![feature(ascii_char)]
#![feature(panic_info_message)]

extern crate alloc;

use crate::files::FILES;
use arch::asm::wfi;
use arch::interrupts::s_enable;
use arch::registers::{ra, sp, stvec};
use core::arch::asm;
use core::ptr::addr_of;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::*;
use cpu::ccpu;
use elf_parse::parse_executable_file;
use kernel::mem::paging::KERNEL_PAGE_TABLE;
use kernel::trampoline::trampoline;
use kernel::*;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section};
use param::{ProcId, NPROC, STACK_SIZE};
use proc::{cpuid, proc, procs};
use trap::user_proc_entry;

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let hart_id = cpuid();
    cprintln!("Started booting CPU #{}", hart_id);
    if hart_id == 0 {
        unsafe { init_kernel() };
        cprintln!("Finished booting CPU #{}", hart_id);
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            wfi()
        }
        unsafe { mem::paging::set_current_page_table(addr_of!(KERNEL_PAGE_TABLE) as usize) };
        plic::init_plic_hart(hart_id);
        // Init kernel trap handler
        unsafe { stvec::write(trap::kernelvec as usize, stvec::TrapMode::Direct) };
        cprintln!("Finished booting CPU #{}", hart_id);
    }

    unsafe { s_enable() };

    if hart_id == 0 {
        FILES.lock().cat("hi.txt");
        let data = FILES.lock().copy_to_ram("test").unwrap();
        cprintln!("JJJ: {:#p}", data.as_ptr());
        let pid = procs().alloc_proc("test").unwrap();
        let exe = parse_executable_file(&data).unwrap();
        proc(pid).activate(exe);

        loop {
            for proc_id in 0..NPROC {
                let proc = proc(proc_id as ProcId);
                if proc
                    .status
                    .compare_exchange(
                        proc::ProcStatus::Runnable,
                        proc::ProcStatus::Running,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    cprintln!(
                        "CPU {} is Running Proc {}: {}",
                        cpuid(),
                        proc_id,
                        proc.name()
                    );
                    ccpu().current_proc = proc.id;
                    unsafe {
                        sp::write(proc.kernel_stack as usize + STACK_SIZE);
                        ra::write(user_proc_entry as usize);
                        asm!("ret");
                    }
                }
            }
            wfi();
        }
    }

    loop {
        wfi()
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
    assert_eq!(proc::procs().alloc_proc("kernel").unwrap(), 0);
    // Init kernel trap handler
    stvec::write(trap::kernelvec as usize, stvec::TrapMode::Direct);
    // Enable S-mode software, external and timer interrupts
    plic::init_plic_global();
    plic::init_plic_hart(0);
    virtio::init_virtio();
    files::init_files();
}
