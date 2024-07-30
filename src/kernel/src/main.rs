#![feature(custom_test_frameworks)]
#![allow(internal_features)]
#![reexport_test_harness_main = "test_main"]
#![allow(static_mut_refs)]
#![test_runner(kernel::test_runner)]
#![no_std]
#![no_main]
#![feature(ascii_char)]
#![feature(panic_info_message)]

extern crate alloc;

use crate::fs::FILES;
use alloc::boxed::Box;
use arch::registers::csr::{Satp, Sepc, Sstatus};
use arch::registers::gpr::{Sp, T2};
use core::arch::asm;
use core::hint;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::{fence, Ordering};
use elf_parse::parse_executable_file;
use kernel::arch::common::privilage::PrivLevel;
use kernel::arch::registers::csr::{Sie, Stvec};
use kernel::arch::registers::WriteInto;
use kernel::arch::registers::{gpr::Tp, ReadFrom};
use kernel::mem::paging::KERNEL_PAGE_TABLE;
use kernel::trampoline::trampoline;
use kernel::trap::SupervisorInterrupt;
use kernel::*;
use kernel::{cprintln, end_of_kernel_code_section, end_of_kernel_data_section};
use param::ProcId;
use proc::{Process, PROCS};
use trap::enable_interrupts;

static STARTED: AtomicBool = AtomicBool::new(false);

#[export_name = "main"]
extern "C" fn main() -> ! {
    let hart_id = unsafe { Tp.read() };

    if hart_id == 0 {
        unsafe { init_kernel(hart_id) };
        // The kernel has officially booted
        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {
            // unsafe { asm!("wfi") };
            // Wait for CPU #0 to set up the kernel properly
        }
    }
    if STARTED.load(Ordering::SeqCst) {
        cprintln!("Hello from CPU #{}", hart_id);

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
    console::init_console();
    cprintln!("\nBooting Kernel...");
    cprintln!("End of kernel code : {:#x}", end_of_kernel_code_section());
    cprintln!("Trampoline frame   : {:#x}", trampoline as u64);
    cprintln!("End of kernel data : {:#x}", end_of_kernel_data_section());
    mem::init_kernel_allocator();
    mem::paging::init_kernel_page_table();
    mem::paging::set_current_page_table(&KERNEL_PAGE_TABLE);
    cprintln!("Page Table has been initialized.");
    proc::init_procs();
    // Enable S-mode software, external and timer interrupts
    Sie.write(
        Sie.read()
            | SupervisorInterrupt::External.bitmask()
            | SupervisorInterrupt::Software.bitmask()
            | SupervisorInterrupt::Timer.bitmask(),
    );
    // Init kernel trap handler
    Stvec.write(trap::kernelvec as u64);
    plic::init_plic_global();
    plic::init_plic_hart(hart_id, PrivLevel::S);
    virtio::init_virtio();
    fs::init_files();

    fence(Ordering::SeqCst);
    enable_interrupts();
    fence(Ordering::SeqCst);

    let test = Box::leak(FILES.lock().copy_to_ram("test").unwrap());
    let test_exe = parse_executable_file(test).unwrap();
    let procs = PROCS.get().unwrap();
    let test_proc_id = procs.alloc_proc("test").unwrap();
    procs[test_proc_id].lock().activate(test_exe);
}

unsafe fn exec_proc(proc: &Process<'static>) {
    Sstatus.write(Sstatus.read() & !(PrivLevel::U as u64));
    Sepc.write(proc.trapframe.epc as u64);
}
