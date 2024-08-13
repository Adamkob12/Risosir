use riscv::{asm::sfence_vma_all, register::satp};

use super::{alloc_frame_unwrap, virtual_mem::*};
use crate::{
    cprint, cprintln, end_of_kernel_code_section, end_of_kernel_data_section,
    memlayout::{
        CLINT_BASE_ADDR, KERNEL_BASE_ADDR, MTIMECMP_ADDR, MTIME_ADDR, PLIC, TRAMPOLINE_VADDR,
        UART_BASE_ADDR, VIRTIO0,
    },
    param::{PAGE_SIZE, RAM_SIZE},
    trampoline::trampoline,
};

/// The kernel L3 page table
pub static mut KERNEL_PAGE_TABLE: PageTable = PageTable::zeroed();

#[derive(Clone, Copy)]
#[repr(C, align(4096))]
pub struct Frame([u8; PAGE_SIZE]);

#[derive(Clone, Copy, Debug)]
#[repr(C, align(4096))]
#[allow(dead_code)]
pub struct Page([u8; PAGE_SIZE]);

pub const fn zerod_frame() -> Frame {
    Frame([0; PAGE_SIZE])
}

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; PAGE_TABLE_ENTRIES]);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PageTableLevel {
    L2 = 2,
    L1 = 1,
    L0 = 0,
}

pub fn make_satp(pt_addr: usize) -> usize {
    let sv39_mode: u64 = 8 << 60;
    let pt_ppn = pt_addr as u64 >> 12;
    (sv39_mode | pt_ppn) as usize
}

/// Updates the current page table in a safe way
/// The given page table must be valid and safe to use
pub unsafe fn set_current_page_table(pt: usize) {
    satp::write(make_satp(pt) as usize);
    sfence_vma_all();
}

/// Only call during bootup, from one thread only, call once
pub unsafe fn init_kernel_page_table() {
    #[cfg(debug_assertions)]
    cprintln!("Mapping UART");
    // Map UART to the same address
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(UART_BASE_ADDR as u64),
        PhysAddr::from_raw(UART_BASE_ADDR as u64),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );

    #[cfg(debug_assertions)]
    cprintln!("Mapping VIRTIO MMIO");
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(VIRTIO0 as u64),
        PhysAddr::from_raw(VIRTIO0 as u64),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );

    #[cfg(debug_assertions)]
    cprintln!("Mapping PLIC");
    for addr in (PLIC..(PLIC + 0x0040_0000)).into_iter().step_by(PAGE_SIZE) {
        KERNEL_PAGE_TABLE.strong_map(
            VirtAddr::from_raw(addr as u64),
            PhysAddr::from_raw(addr as u64),
            PTEFlags::valid().readable().writable(),
            PageTableLevel::L2,
        );
    }

    #[cfg(debug_assertions)]
    cprintln!("Mapping Kernel Text (Source code)");
    // Map kernel source code (text section), leave space for trampoline
    for addr in (KERNEL_BASE_ADDR..(end_of_kernel_code_section() - PAGE_SIZE)).step_by(PAGE_SIZE) {
        KERNEL_PAGE_TABLE.strong_map(
            VirtAddr::from_raw(addr as u64),
            PhysAddr::from_raw(addr as u64),
            PTEFlags::valid().readable().executable(),
            PageTableLevel::L2,
        );
    }

    #[cfg(debug_assertions)]
    cprintln!(
        "Mapping Trampoline: {:#x} -> {:#x}",
        TRAMPOLINE_VADDR,
        trampoline as u64
    );
    // Map trampoline page
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(TRAMPOLINE_VADDR as u64),
        PhysAddr::from_raw(trampoline as u64),
        PTEFlags::valid().readable().executable(),
        PageTableLevel::L2,
    );

    #[cfg(debug_assertions)]
    cprintln!("Mapping Kernel Data (data + rodata sections)");
    // Map kernel source code (text section)
    for addr in (end_of_kernel_code_section()..end_of_kernel_data_section()).step_by(PAGE_SIZE) {
        KERNEL_PAGE_TABLE.strong_map(
            VirtAddr::from_raw(addr as u64),
            PhysAddr::from_raw(addr as u64),
            PTEFlags::valid().readable().writable(),
            PageTableLevel::L2,
        );
    }

    #[cfg(debug_assertions)]
    cprintln!("Mapping CLINT");
    // Map CLINT
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(CLINT_BASE_ADDR as u64),
        PhysAddr::from_raw(CLINT_BASE_ADDR as u64),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(MTIMECMP_ADDR as u64),
        PhysAddr::from_raw(MTIMECMP_ADDR as u64),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(MTIME_ADDR as u64),
        PhysAddr::from_raw(MTIME_ADDR as u64),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw((PAGE_SIZE + MTIME_ADDR) as u64),
        PhysAddr::from_raw((PAGE_SIZE + MTIME_ADDR) as u64),
        PTEFlags::valid().readable().writable(),
        PageTableLevel::L2,
    );

    #[cfg(debug_assertions)]
    cprintln!("Mapping boot ROM");
    // Map boot ROM
    KERNEL_PAGE_TABLE.strong_map(
        VirtAddr::from_raw(0x1000),
        PhysAddr::from_raw(0x1000),
        PTEFlags::valid().readable().executable(),
        PageTableLevel::L2,
    );

    #[cfg(debug_assertions)]
    cprintln!("Mapping Entire RAM");
    // Map the entire RAM 1 to 1 for the kernel
    for addr in (end_of_kernel_data_section()..(KERNEL_BASE_ADDR + RAM_SIZE - 20 * PAGE_SIZE))
        .step_by(PAGE_SIZE)
    {
        KERNEL_PAGE_TABLE.strong_map(
            VirtAddr::from_raw(addr as u64),
            PhysAddr::from_raw(addr as u64),
            PTEFlags::valid().readable().writable().executable(),
            PageTableLevel::L2,
        );
    }
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
    pub const fn zeroed() -> Self {
        PageTable([PageTableEntry::new_invalid(); PAGE_TABLE_ENTRIES])
    }

    /// Map a page to a frame, allocate frames for additional tables if needed.
    /// Panic if failed.
    /// Must be called in the kernel while paging is off (or if the entire RAM is mapped 1 to 1 for the kernel)
    /// If there was a page that was previously mapped to that frame (it had a valid entry), return it - otherwise return `None`.
    pub fn strong_map(
        &mut self,
        mut va: VirtAddr,
        mut pa: PhysAddr,
        flags: PTEFlags,
        current_level: PageTableLevel,
    ) -> Option<PageTableEntry> {
        va.round_down();
        pa.round_down();
        let vpn = va.vpn(current_level);
        // cprintln!("vpn: {}", vpn);
        let pte = &mut self.0[vpn as usize];
        if let Some(level_down) = current_level.one_level_down() {
            // cprint!("level down, ");
            if pte.is_valid() {
                // cprint!("valid, ");
                if pte.is_redirect() {
                    // cprint!("redirect, ");
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
                // cprint!("invalid, ");
                // Need to allocate a new page table
                let frame = unsafe { alloc_frame_unwrap() };
                // .expect("Couldn't allocate a frame for a new page table.");
                // The frame address is 4096 bytes aligned so it will fit as is into the pte ppn
                // unsafe {
                //     (pte as *mut PageTableEntry)
                //         .write_volatile(PageTableEntry::new(frame, PTEFlags::redirect()))
                pte.set(frame.as_ptr() as u64, PTEFlags::redirect());
                // };
                // Now map from the new page table
                unsafe { frame.cast::<PageTable>().as_mut() }.strong_map(va, pa, flags, level_down)
            }
        } else {
            // cprint!("level zero, ");
            // We reached L0
            // Save the PTE in case we need to return it
            let prev_pte = *pte;

            // let new_pte = PageTableEntry::new(
            //     NonNull::new(pa.frame_adrr() as *mut Frame)
            //         .expect("Can't map physical address corresponding to PPN 0"),
            //     flags,
            // );
            // cprintln!("New PTE: {:#b}", new_pte.as_u64());
            // Set the new entry according to the given physical address and flags
            // unsafe { (pte as *mut PageTableEntry).write_volatile(new_pte) };
            pte.set(pa.frame_adrr(), flags);

            // Return the previous entry if needed
            if prev_pte.is_valid() {
                Some(prev_pte)
            } else {
                None
            }
        }
    }

    pub fn debug(&self, prefix: &str, level: usize) {
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
                let pt = pte.frame_addr() as *const PageTable;
                prefix_print!("Found Redirect to new page table at {:#p}", pt);
                unsafe { (*pt).debug(prefix, level + 1) };
            } else {
                prefix_print!("% Found pointer to frame: {:#x}", pte.frame_addr());
            }
        }
    }
}

/// Based on page 109 of The RISC-V Manual II
pub fn translate(pt: &PageTable, va: VirtAddr, i: PageTableLevel, flags: PTEFlags) -> PhysAddr {
    // step 2
    let index = va.vpn(i);
    let pte = pt.0[index as usize];
    if !pte.is_valid() {
        panic!("Invalid, va: {:#x}", va.as_u64());
    }
    if !pte.is_readable() && pte.is_writable() {
        panic!("JJA");
    }
    if pte.is_redirect() {
        // step 4
        translate(
            unsafe { &*(pte.frame_addr() as *const PageTable) },
            va,
            i.one_level_down().unwrap(),
            flags,
        )
    } else {
        // step 5
        if flags.is_executable() {
            assert!(pte.is_executable());
        }
        if flags.is_readable() {
            assert!(pte.is_readable());
        }
        if flags.is_writable() {
            assert!(pte.is_writable());
        }
        if pte.is_readable() {
            PhysAddr::from_raw(pte.frame_addr() | va.offset())
        } else {
            panic!("JJJ");
        }
    }
}
