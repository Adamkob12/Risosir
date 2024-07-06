use super::registers::{csr::Satp, WriteInto};

pub mod privilage;

pub unsafe fn disable_paging() {
    unsafe { Satp.write(0) };
}
