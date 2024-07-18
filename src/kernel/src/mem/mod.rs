pub mod paging;
pub mod virtual_mem;

use crate::{cprintln, end_of_kernel_data_section, param::RAM_SIZE};
use core::{alloc::Layout, ptr::NonNull};
use linked_list_allocator::LockedHeap;
use paging::{garbage_frame, garbage_frames, Frame};

#[cfg(not(feature = "debug-allocations"))]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

#[cfg(feature = "debug-allocations")]
pub static mut ALLOCATOR: DebugAllocator = DebugAllocator(LockedHeap::empty());

pub unsafe fn init_kernel_allocator() {
    ALLOCATOR
        .lock()
        .init(end_of_kernel_data_section() + 0x1000, RAM_SIZE);
}

#[repr(transparent)]
pub struct DebugAllocator(LockedHeap);

impl core::ops::Deref for DebugAllocator {
    type Target = LockedHeap;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub unsafe fn alloc_frame_unwrap() -> NonNull<Frame> {
    let p = ALLOCATOR
        .lock()
        .allocate_first_fit(Layout::new::<Frame>())
        .unwrap()
        .cast();
    p.write(garbage_frame());
    p
}

pub unsafe fn alloc_frame() -> Option<NonNull<Frame>> {
    ALLOCATOR
        .lock()
        .allocate_first_fit(Layout::new::<Frame>())
        .ok()
        .map(|p| {
            let p = p.cast();
            p.write(garbage_frame());
            #[cfg(feature = "debug-allocations")]
            // cprintln!("Allocated Frame at {:#x}", p.as_ptr() as usize);
            p
        })
}

pub unsafe fn alloc_frames<const N: usize>() -> Option<NonNull<[Frame; N]>> {
    ALLOCATOR
        .lock()
        .allocate_first_fit(Layout::array::<Frame>(N).ok()?)
        .ok()
        .map(|p| {
            let p = p.cast();
            p.write(garbage_frames());
            #[cfg(feature = "debug-allocations")]
            cprintln!("Allocated a {N} Frames array at {:#x}", p.as_ptr() as usize);
            p
        })
}
