pub mod paging;
pub mod virtual_mem;

use crate::{end_of_kernel_code_address, param::RAM_SIZE};
use core::{alloc::Layout, ptr::NonNull};
use linked_list_allocator::LockedHeap;
use paging::Frame;

#[cfg(not(feature = "debug-allocations"))]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

#[cfg(feature = "debug-allocations")]
pub static mut ALLOCATOR: DebugAllocator = DebugAllocator(LockedHeap::empty());

pub unsafe fn init_kernel_allocator() {
    ALLOCATOR
        .lock()
        .init(end_of_kernel_code_address() + 0x1000, RAM_SIZE);
}

#[repr(transparent)]
pub struct DebugAllocator(LockedHeap);

impl core::ops::Deref for DebugAllocator {
    type Target = LockedHeap;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub unsafe fn alloc_frame() -> Option<NonNull<Frame>> {
    ALLOCATOR
        .lock()
        .allocate_first_fit(Layout::new::<Frame>())
        .ok()
        .map(|p| p.cast())
}

pub unsafe fn alloc_frames<const N: usize>() -> Option<NonNull<[Frame; N]>> {
    ALLOCATOR
        .lock()
        .allocate_first_fit(Layout::array::<Frame>(N).ok()?)
        .ok()
        .map(|p| p.cast())
}
