use super::registers::{csr::Satp, WriteInto};

pub mod privilage;

pub unsafe fn disable_paging() {
    Satp::write(0);
}
