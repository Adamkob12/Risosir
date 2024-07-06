use crate::arch::memlayout::{MTIMECMP_ADDR, MTIME_ADDR};

use super::{AddressOf, ReadFrom, WriteInto};

/// Memory mapped register (part of the CLINT), keeps track how many cycles have passed since boot, wraps around when reaches [`u64::MAX`].
/// __Section 3.2.1 in *The RISC-V Instruction Set Manual: Volume II*__
pub struct Mtime;

impl ReadFrom for Mtime {
    type Out = u64;

    unsafe fn read(&self) -> Self::Out {
        unsafe { *(MTIME_ADDR as *const u64) }
    }
}

/// Memory mapped register (part of the CLINT), unique to each hart. If [`Mtime`] >= [`Mtimecmp`] a timer interrupt will be triggered.
/// Writing to this register allows the kernel developer to schedule the next timer interrupt.
/// __Section 3.2.1 in *The RISC-V Instruction Set Manual: Volume II*__
pub struct Mtimecmp {
    pub hart_id: u64,
}

impl WriteInto for Mtimecmp {
    type In = u64;
    unsafe fn write(&self, val: Self::In) {
        unsafe { *((MTIMECMP_ADDR + 8 * self.hart_id as usize) as *mut u64) = val };
    }
}

impl AddressOf for Mtimecmp {
    fn addr_of(&self) -> usize {
        MTIMECMP_ADDR + 8 * self.hart_id as usize
    }
}
