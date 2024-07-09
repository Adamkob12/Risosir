use core::alloc::{GlobalAlloc, Layout};

use crate::{cprintln, end_of_kernel_code_address};
use linked_list_allocator::LockedHeap;

const HEAP_SIZE: usize = 2 * 1024 * 1024;

#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

pub unsafe fn init_heap() {
    ALLOCATOR
        .lock()
        .init(end_of_kernel_code_address() + 0x1000, HEAP_SIZE);
}
