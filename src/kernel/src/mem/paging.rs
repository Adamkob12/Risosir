use super::{virtual_mem::*, ALLOCATOR};
use crate::param::PAGE_SIZE;

#[repr(C, align(4096))]
pub struct Frame([u8; PAGE_SIZE]);

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; PAGE_TABLE_ENTRIES]);

/// The kernel L3 page table
pub static mut KERNEL_PAGE_TABLE: PageTable = PageTable::empty();

impl PageTable {
    pub const fn empty() -> Self {
        PageTable([PageTableEntry::new_invalid(); PAGE_TABLE_ENTRIES])
    }
}
