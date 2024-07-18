use super::{alloc_frame, virtual_mem::*};
use crate::{
    arch::{
        common::sfence_vma,
        memlayout::{CLINT_BASE_ADDR, KERNEL_BASE_ADDR, MTIMECMP_ADDR, MTIME_ADDR, UART_BASE_ADDR},
        registers::{csr::Satp, WriteInto},
    },
    cprint, cprintln, end_of_kernel_code_section,
    param::{PAGE_SIZE, RAM_SIZE},
};
use core::ptr::NonNull;

/// The kernel L3 page table
pub static mut KERNEL_PAGE_TABLE: PageTable = PageTable::empty();

#[derive(Clone, Copy)]
#[repr(C, align(4096))]
pub struct Frame([u8; PAGE_SIZE]);

pub(super) const fn garbage_frame() -> Frame {
    Frame([0; PAGE_SIZE])
}

pub(super) const fn garbage_frames<const N: usize>() -> [Frame; N] {
    [garbage_frame(); N]
}

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; PAGE_TABLE_ENTRIES]);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageTableLevel {
    L2 = 2,
    L1 = 1,
    L0 = 0,
}

/// Updates the current page table in a safe way
/// The given page table must be valid and safe to use
pub unsafe fn set_current_page_table(pt: &'static PageTable) {
    sfence_vma();
    let sv39_mode: u64 = 8 << 60;
    cprintln!("{:#p}", pt);
    let pt_ppn = pt as *const PageTable as u64 >> 12;
    cprintln!("{:#x}", pt_ppn);
    Satp.write(sv39_mode | pt_ppn);
    sfence_vma();
}

/// Only call during bootup, from one thread only, call once
pub unsafe fn init_kernel_page_table() {
    #[cfg(feature = "debug-allocations")]
    cprintln!("Mapping UART");
    // Map UART to the same address
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(UART_BASE_ADDR),
        PhysAddr::from_raw(UART_BASE_ADDR),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );

    #[cfg(feature = "debug-allocations")]
    cprintln!("Mapping CLINT");
    // Map CLINT
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(CLINT_BASE_ADDR),
        PhysAddr::from_raw(CLINT_BASE_ADDR),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(MTIMECMP_ADDR),
        PhysAddr::from_raw(MTIMECMP_ADDR),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(MTIME_ADDR),
        PhysAddr::from_raw(MTIME_ADDR),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );

    #[cfg(feature = "debug-allocations")]
    cprintln!("Mapping boot ROM");
    // Map boot ROM
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(0x1000),
        PhysAddr::from_raw(0x1000),
        PTEFlags::valid().readable().executable(),
        PageTableLevel::L2,
    );

    #[cfg(feature = "debug-allocations")]
    cprintln!("Mapping Kernel code (text section)");
    // Map kernel source code (text section)
    for addr in (KERNEL_BASE_ADDR..end_of_kernel_code_section()).step_by(PAGE_SIZE) {
        KERNEL_PAGE_TABLE.strong_map(
            VirtAddr::from_raw(addr),
            PhysAddr::from_raw(addr),
            PTEFlags::valid().readable().executable(),
            PageTableLevel::L2,
        );
    }

    #[cfg(feature = "debug-allocations")]
    cprintln!("Mapping Entire RAM");
    // Map the entire RAM 1 to 1 for the kernel
    for addr in (end_of_kernel_code_section()..(KERNEL_BASE_ADDR + RAM_SIZE)).step_by(PAGE_SIZE) {
        KERNEL_PAGE_TABLE.strong_map(
            VirtAddr::from_raw(addr),
            PhysAddr::from_raw(addr),
            PTEFlags::valid().readable().writable().executable(),
            PageTableLevel::L2,
        );
    }

    KERNEL_PAGE_TABLE.debug("\t", 0);
}

impl PageTableLevel {
    fn one_level_down(&self) -> Option<Self> {
        match self {
            Self::L2 => Some(Self::L1),
            Self::L1 => Some(Self::L0),
            Self::L0 => None,
        }
    }
}

impl PageTable {
    pub const fn empty() -> Self {
        PageTable([PageTableEntry::new_invalid(); PAGE_TABLE_ENTRIES])
    }

    /// Map a page to a frame, allocate frames for additional tables if needed.
    /// Panic if failed.
    /// Must be called in the kernel while paging is off (or if the entire RAM is mapped 1 to 1 for the kernel)
    /// If there was a page that was previously mapped to that frame (it had a valid entry), return it - otherwise return `None`.
    pub fn strong_map(
        &mut self,
        va: VirtAddr,
        pa: PhysAddr,
        flags: PTEFlags,
        current_level: PageTableLevel,
    ) -> Option<PageTableEntry> {
        let pte = &mut self.0[va.pte(current_level) as usize];
        if let Some(level_down) = current_level.one_level_down() {
            if pte.is_valid() {
                if pte.is_redirect() {
                    // The PTE points to another page table, map from it recursively
                    unsafe {
                        (pte.frame_addr() as *mut PageTable)
                            .as_mut()
                            .expect("Page Table Entry had 0 in PPN")
                    }
                    .strong_map(va, pa, flags, level_down)
                } else {
                    // The page is valid, but it doesn't contain a pointer to another page table like we expected (we aren't in L0 yet)
                    panic!("Expected a redirect to another page table");
                }
            } else {
                // Need to allocate a new page table
                let frame = unsafe { alloc_frame() }
                    .expect("Couldn't allocate a frame for a new page table.");
                // The frame address is 4096 bytes aligned so it will fit as is into the pte ppn
                *pte = PageTableEntry::new(frame, PTEFlags::redirect());
                // Now map from the new page table
                unsafe { frame.cast::<PageTable>().as_mut() }.strong_map(va, pa, flags, level_down)
            }
        } else {
            // We reached L0
            // Save the PTE in case we need to return it
            let prev_pte = *pte;

            // Set the new entry according to the given physical address and flags
            *pte = PageTableEntry::new(
                NonNull::new(pa.frame_adrr() as *mut Frame)
                    .expect("Can't map physical address corresponding to PPN 0"),
                flags,
            );

            // Return the previous entry if needed
            if prev_pte.is_valid() {
                Some(prev_pte)
            } else {
                None
            }
        }
    }

    pub fn debug(&self, prefix: &'static str, level: usize) {
        macro_rules! prefix_print {
            ($($arg:tt)*) =>{
                for _ in 0..level {
                    cprint!("{}", prefix);
                }
                cprintln!($($arg)*);
            };
        }
        prefix_print!("Debugging Page Table (level {})", 3 - level);
        for pte in self.0.iter().filter(|e| e.is_valid()) {
            if pte.is_redirect() {
                prefix_print!("Found Redirect to new page table -> ");
                let pt = pte.frame_addr() as *const PageTable;
                unsafe { (*pt).debug(prefix, level + 1) };
            } else {
                prefix_print!("% Found pointer to frame: {:#x}", pte.frame_addr());
            }
        }
    }
}
