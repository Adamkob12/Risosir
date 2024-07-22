use crate::{
    arch::memlayout::TRAMPOLINE_ADDR,
    mem::{
        alloc_frame,
        paging::{Frame, Page, PageTable, PageTableLevel},
        virtual_mem::{PTEFlags, PhysAddr, VirtAddr},
    },
    param::{ProcId, NPROC, PAGES_PER_HEAP, PAGES_PER_STACK, PAGE_SIZE},
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

pub struct ProcContext<'p> {
    stack_pointer: u64,
    program_counter: u64,
    trapframe: &'p Page,
    trampoline: &'p Page,
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
                VirtAddr::from_raw(current as u64),
                PhysAddr::from_raw(frame.as_ptr() as u64),
                PTEFlags::valid().readable().writable(),
                PageTableLevel::L2,
            );
            current += PAGE_SIZE;
        }
        // map the trampoline
        pt.strong_map(
            VirtAddr::from_raw(current as u64),
            PhysAddr::from_raw(TRAMPOLINE_ADDR as u64),
            PTEFlags::valid().readable().executable(),
            PageTableLevel::L2,
        );

        self.context = Some(ProcContext {
            stack_pointer,
            program_counter,
            trapframe: unsafe { transmute(current - PAGE_SIZE) },
            trampoline: unsafe { transmute(current) },
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
