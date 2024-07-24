use crate::arch::registers::{csr::Sstatus, ReadFrom, WriteInto};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
#[non_exhaustive]
/// Table 15
pub enum MachineInterrupt {
    Timer = 7,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
#[non_exhaustive]
/// Table 22
pub enum SupervisorInterrupt {
    Software = 1,
    Timer = 5,
    External = 9,
}

impl MachineInterrupt {
    pub fn bitmask(&self) -> u64 {
        1 << (*self as u64)
    }
}

impl SupervisorInterrupt {
    pub fn bitmask(&self) -> u64 {
        1 << (*self as u64)
    }
}

pub fn enable_interrupts() {
    unsafe { Sstatus.write(Sstatus.read() | (1 << 1)) };
}

pub fn disable_interrupts() {
    unsafe { Sstatus.write(Sstatus.read() & !(1 << 1)) };
}

pub fn without_interrupts<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let sie = unsafe { Sstatus.read() & (1 << 1) };
    disable_interrupts();
    let t = f();
    unsafe { Sstatus.write(Sstatus.read() | sie) };
    t
}
