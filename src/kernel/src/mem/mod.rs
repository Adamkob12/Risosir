pub mod paging;
pub mod virtual_mem;

use crate::{end_of_kernel_data_section, param::RAM_SIZE};
use core::{alloc::Layout, ptr::NonNull};
use linked_list_allocator::LockedHeap;
use paging::{zerod_frame, Frame};

#[cfg(not(feature = "debug-allocations"))]
#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::empty();

#[cfg(feature = "debug-allocations")]
#[global_allocator]
pub static mut ALLOCATOR: DebugAllocator = DebugAllocator(LockedHeap::empty());

pub unsafe fn init_kernel_allocator() {
    ALLOCATOR
        .lock()
        .init(end_of_kernel_data_section() + 0x1000, RAM_SIZE);
}

#[repr(transparent)]
#[cfg(feature = "debug-allocations")]
pub struct DebugAllocator(LockedHeap);

#[cfg(feature = "debug-allocations")]
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
    p.write(zerod_frame());
    p
}

pub unsafe fn alloc_frame() -> Option<NonNull<Frame>> {
    ALLOCATOR
        .lock()
        .allocate_first_fit(Layout::new::<Frame>())
        .ok()
        .map(|p| {
            let p = p.cast();
            p.write_volatile(zerod_frame());
            #[cfg(feature = "debug-allocations")]
            {
                // crate::cprintln!("Allocated Frame at {:#x}", p.as_ptr() as usize);
            }
            p
        })
}

#[cfg(feature = "debug-allocations")]
unsafe impl core::alloc::GlobalAlloc for DebugAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = self.0.alloc(layout);
        cprintln!("Allocated: {:#?}", layout);
        cprintln!("At address: {:#p}", p);
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.dealloc(ptr, layout);
        cprintln!("Deallocated: {:#?}", layout);
        cprintln!("At address: {:#p}", ptr);
    }
}
