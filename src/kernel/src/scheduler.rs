use crate::*;
use arch::asm::wfi;
use arch::interrupts::s_enable;
use arch::registers::{ra, sp};
use core::arch::asm;
use core::sync::atomic::*;
use cpu::ccpu;
use param::{ProcId, NPROC, STACK_SIZE};
use proc::{cpuid, proc};
use trap::user_proc_entry;

pub fn scheduler(_hart_id: usize) -> ! {
    loop {
        unsafe { s_enable() };
        for proc_id in 0..NPROC {
            let proc = proc(proc_id as ProcId);
            // cprintln!("Found proc {}", proc.name());
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
