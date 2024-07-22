use crate::{
    arch::memlayout::{TRAMPOLINE_VADDR, TRAPFRAME_VADDR},
    mem::{
        alloc_frame,
        paging::{Frame, Page, PageTable, PageTableLevel},
        virtual_mem::{PTEFlags, PhysAddr, VirtAddr},
    },
    param::{ProcId, HEAP_SIZE, NPROC, PAGES_PER_HEAP, PAGES_PER_STACK, PAGE_SIZE},
    trampoline::trampoline,
};
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::{
    mem::transmute,
    sync::atomic::{AtomicUsize, Ordering},
};
use spin::RwLock;

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum ProcStatus {
    Inactive = 0,
    Active = 1,
    Running = 2,
}

pub struct AtomicProcStatus(AtomicUsize);

impl AtomicProcStatus {
    pub fn new(val: ProcStatus) -> Self {
        Self(AtomicUsize::new(val as usize))
    }

    pub fn load(&self, order: Ordering) -> ProcStatus {
        // 1 to 1 correspondance between usize and ProcStatus
        unsafe { core::mem::transmute(self.0.load(order)) }
    }

    pub fn store(&self, val: ProcStatus, order: Ordering) {
        self.0.store(val as usize, order)
    }

    pub fn compare_exchange(
        &self,
        current: ProcStatus,
        new: ProcStatus,
        success: Ordering,
        failure: Ordering,
    ) -> Result<ProcStatus, ProcStatus> {
        // 1 to 1 correspondance between usize and ProcStatus
        unsafe {
            core::mem::transmute(self.0.compare_exchange(
                current as usize,
                new as usize,
                success,
                failure,
            ))
        }
    }

    pub fn compare_exchange_weak(
        &self,
        current: ProcStatus,
        new: ProcStatus,
        success: Ordering,
        failure: Ordering,
    ) -> Result<ProcStatus, ProcStatus> {
        // 1 to 1 correspondance between usize and ProcStatus
        unsafe {
            core::mem::transmute(self.0.compare_exchange_weak(
                current as usize,
                new as usize,
                success,
                failure,
            ))
        }
    }
}

pub struct Process<'p> {
    /// The name of the process, inactive processes are named "X"
    name: &'p str,
    /// Indexes [`ProcTable`]
    id: ProcId,
    /// The status of the process
    status: AtomicProcStatus,
    context: Option<ProcContext<'p>>,
}

// All of the values here are *Virtual* addresses with respect to this processes page table.
pub struct ProcContext<'p> {
    stack_pointer: u64,
    program_counter: u64,
    heap_start: u64,
    heap_size: u64,
    trapframe: &'p mut Trapframe,
    page_table: &'p PageTable,
}

const INACTIVE_PROC_NAME: &str = "X";

pub struct ProcTable([RwLock<Process<'static>>; NPROC]);

pub static PROCS: OnceCell<ProcTable> = OnceCell::uninit();

pub unsafe fn init_procs() {
    PROCS.init_once(|| ProcTable::new());
}

impl<'a> Process<'a> {
    pub fn new_inactive(id: ProcId) -> Self {
        Process {
            name: INACTIVE_PROC_NAME,
            id,
            status: AtomicProcStatus::new(ProcStatus::Inactive),
            context: None,
        }
    }

    pub fn activate(&mut self, text: Box<[Frame]>, data: Box<[Frame]>) {
        self.status.store(ProcStatus::Active, Ordering::Relaxed);
        let pt = Box::leak(Box::new(PageTable::empty()));
        let mut current = 0x0;
        // "map" the page at 0x0 (we dont want to actually use it so null deref would trigger a page fault)
        current += PAGE_SIZE;
        // The program counter needs to start at the start of the code section
        let program_counter = current as u64;
        // Map the text section (code of the process), it needs to be readable and executable
        for text_frame in &text {
            pt.strong_map(
                VirtAddr::from_raw(current as u64),
                PhysAddr::from_raw(text_frame as *const Frame as u64),
                PTEFlags::valid().readable().executable(),
                PageTableLevel::L2,
            );
            current += PAGE_SIZE;
        }
        // Map the data section (global variables, statics etc.), it needs to be readable and writable
        for data_frame in &data {
            pt.strong_map(
                VirtAddr::from_raw(current as u64),
                PhysAddr::from_raw(data_frame as *const Frame as u64),
                PTEFlags::valid().readable().writable(),
                PageTableLevel::L2,
            );
            current += PAGE_SIZE;
        }
        // map the stack
        for _ in 0..PAGES_PER_STACK {
            let frame = unsafe { alloc_frame() }.unwrap();
            pt.strong_map(
                VirtAddr::from_raw(current as u64),
                PhysAddr::from_raw(frame.as_ptr() as u64),
                PTEFlags::valid().readable().writable(),
                PageTableLevel::L2,
            );
            current += PAGE_SIZE;
        }
        // The stack pointer is set to the top of the stack
        let stack_pointer = current as u64;
        let heap_start = current as u64;
        // map the heap
        for _ in 0..PAGES_PER_HEAP {
            let frame = unsafe { alloc_frame() }.unwrap();
            pt.strong_map(
                VirtAddr::from_raw(current as u64),
                PhysAddr::from_raw(frame.as_ptr() as u64),
                PTEFlags::valid().readable().writable(),
                PageTableLevel::L2,
            );
            current += PAGE_SIZE;
        }
        // map the trapframe
        {
            let frame = unsafe { alloc_frame() }.unwrap();
            pt.strong_map(
                VirtAddr::from_raw(TRAPFRAME_VADDR as u64),
                PhysAddr::from_raw(frame.as_ptr() as u64),
                PTEFlags::valid().readable().writable(),
                PageTableLevel::L2,
            );
        }
        // map the trampoline
        pt.strong_map(
            VirtAddr::from_raw(TRAMPOLINE_VADDR as u64),
            PhysAddr::from_raw(trampoline as u64),
            PTEFlags::valid().readable().executable(),
            PageTableLevel::L2,
        );

        self.context = Some(ProcContext {
            stack_pointer,
            program_counter,
            heap_start,
            heap_size: HEAP_SIZE as u64,
            trapframe: unsafe { transmute(TRAPFRAME_VADDR) },
            page_table: pt,
        })
    }
}

impl ProcTable {
    pub fn new() -> Self {
        ProcTable(core::array::from_fn(|i| {
            RwLock::new(Process::new_inactive(i as ProcId))
        }))
    }
}

impl core::ops::Index<ProcId> for ProcTable {
    type Output = RwLock<Process<'static>>;

    fn index(&self, index: ProcId) -> &Self::Output {
        &self.0[index as usize]
    }
}

#[derive(Clone, Copy, Default, Debug)]
#[repr(C, align(4096))]
pub struct Trapframe {
    /*   0 */ pub kernel_satp: usize, // kernel page table
    /*   8 */ pub kernel_sp: usize, // top of process's kernel stack
    /*  16 */ pub kernel_trap: usize, // usertrap()
    /*  24 */ pub epc: usize, // saved user program counter
    /*  32 */ pub kernel_hartid: usize, // saved kernel tp
    /*  40 */ pub ra: usize,
    /*  48 */ pub sp: usize,
    /*  56 */ pub gp: usize,
    /*  64 */ pub tp: usize,
    /*  72 */ pub t0: usize,
    /*  80 */ pub t1: usize,
    /*  88 */ pub t2: usize,
    /*  96 */ pub s0: usize,
    /* 104 */ pub s1: usize,
    /* 112 */ pub a0: usize,
    /* 120 */ pub a1: usize,
    /* 128 */ pub a2: usize,
    /* 136 */ pub a3: usize,
    /* 144 */ pub a4: usize,
    /* 152 */ pub a5: usize,
    /* 160 */ pub a6: usize,
    /* 168 */ pub a7: usize,
    /* 176 */ pub s2: usize,
    /* 184 */ pub s3: usize,
    /* 192 */ pub s4: usize,
    /* 200 */ pub s5: usize,
    /* 208 */ pub s6: usize,
    /* 216 */ pub s7: usize,
    /* 224 */ pub s8: usize,
    /* 232 */ pub s9: usize,
    /* 240 */ pub s10: usize,
    /* 248 */ pub s11: usize,
    /* 256 */ pub t3: usize,
    /* 264 */ pub t4: usize,
    /* 272 */ pub t5: usize,
    /* 280 */ pub t6: usize,
}
