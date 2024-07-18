use core::arch::asm;

use crate::mem::paging::PageTable;

use super::registers::{csr::Satp, WriteInto};

pub mod privilage;

/// Call "Sfence.vma" instruction (often used to make sure updating the satp goes smoothly)
pub unsafe fn sfence_vma() {
    asm!("sfence.vma zero, zero");
}
