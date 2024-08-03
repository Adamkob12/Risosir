use crate::{
    param::{ProcId, NCPU},
    proc::{cpuid, procs, Process},
};

#[derive(Clone, Copy)]
pub struct CPU {
    pub current_proc: ProcId,
}

pub static mut CURRENT_CPUS: [CPU; NCPU] = [CPU::new(); NCPU];

impl CPU {
    const fn new() -> Self {
        CPU { current_proc: 0 }
    }
}

/// Current cpu
pub fn ccpu<'a>() -> &'a mut CPU {
    // SAFETY: guarnteed to only be accessed from one CPU at a time
    unsafe { &mut CURRENT_CPUS[cpuid()] }
}

/// Current running process
pub fn cproc<'a>() -> &'a Process {
    &procs()[ccpu().current_proc]
}
