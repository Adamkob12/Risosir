use crate::{
    elf_parse::ParsedExecutable,
    mem::{
        alloc_frame,
        paging::{PageTable, PageTableLevel},
        virtual_mem::{PTEFlags, PhysAddr, VirtAddr},
    },
    memlayout::{TRAMPOLINE_VADDR, TRAPFRAME_VADDR},
    param::{ProcId, HEAP_SIZE, HEAP_START, NPROC, PAGE_SIZE, STACK_SIZE},
    trampoline::trampoline,
};
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::{
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};
use spin::Mutex;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum ProcStatus {
    Unused = 0,
    Inactive = 1,
    Runnable = 2,
    Running = 3,
}

pub struct AtomicProcStatus(AtomicUsize);

pub struct Process<'p> {
    /// The name of the process, inactive processes are named "X"
    name: &'p str,
    /// Indexes [`ProcTable`]
    id: ProcId,
    /// The status of the process
    status: AtomicProcStatus,
    pub page_table: &'p mut PageTable,
    pub trapframe: &'p mut Trapframe,
}

const INACTIVE_PROC_NAME: &str = "X";

pub struct ProcTable([Mutex<Process<'static>>; NPROC]);

pub static PROCS: OnceCell<ProcTable> = OnceCell::uninit();

pub unsafe fn init_procs() {
    PROCS.init_once(|| ProcTable::new());
}

impl<'a> Process<'a> {
    pub fn new_inactive(id: ProcId) -> Self {
        let pt: &mut PageTable = Box::leak(unsafe { Box::new_zeroed().assume_init() });
        let tf: &mut Trapframe = Box::leak(unsafe { Box::new_zeroed().assume_init() });
        Process {
            name: INACTIVE_PROC_NAME,
            id,
            status: AtomicProcStatus::new(ProcStatus::Unused),
            page_table: pt,
            trapframe: tf,
        }
    }

    pub fn activate(&mut self, exe: ParsedExecutable<'a>) {
        // The program counter needs to start at the start of the code section
        let program_counter = exe.entry_point as u64;
        let file_base = exe.file_data.as_ptr() as usize;
        let mut data_end = 0;
        // Map the text section (code of the process), it needs to be readable and executable
        for seg in exe.segs {
            let mut flags = PTEFlags::valid();
            if seg.p_flags & (1 << 2) != 0 {
                flags = flags.readable();
            }
            if seg.p_flags & (1 << 1) != 0 {
                flags = flags.writable();
            }
            if seg.p_flags & (1 << 0) != 0 {
                flags = flags.executable();
            }
            let vaddr_base = seg.p_vaddr;
            let size = seg.p_memsz;

            for offset in (0..size).into_iter().step_by(PAGE_SIZE) {
                self.page_table.strong_map(
                    VirtAddr::from_raw(vaddr_base + offset),
                    PhysAddr::from_raw(file_base as u64 + offset),
                    flags,
                    PageTableLevel::L2,
                );
            }

            if seg.p_type <= 0x00000007 {
                data_end = vaddr_base + size;
            }
        }

        // Allocate and map the stack, take into account that we want to keep at least
        // a single non-mapped region of memory (with the size of a single page) between the
        // stack and the data, so in case of a stack overflow, a stack overflow exception
        // will occur and no data will be corrupted.
        let stack_pointer = {
            let stack_addr = data_end + (STACK_SIZE + PAGE_SIZE) as u64;
            for offset in (0..STACK_SIZE as u64).into_iter().step_by(PAGE_SIZE) {
                let frame_addr = unsafe { alloc_frame() }.unwrap().as_ptr() as u64;
                if let Some(_) = self.page_table.strong_map(
                    VirtAddr::from_raw(stack_addr as u64 + offset),
                    PhysAddr::from_raw(frame_addr),
                    PTEFlags::valid().readable().writable(),
                    PageTableLevel::L2,
                ) {
                    panic!("Stack section overlaps with data segments");
                }
            }
            stack_addr + STACK_SIZE as u64
        };

        // Allocate and map the heap
        {
            for offset in (0..HEAP_SIZE).into_iter().step_by(PAGE_SIZE) {
                let frame_addr = unsafe { alloc_frame() }.unwrap().as_ptr() as u64;
                if let Some(_) = self.page_table.strong_map(
                    VirtAddr::from_raw(HEAP_START + offset as u64),
                    PhysAddr::from_raw(frame_addr),
                    PTEFlags::valid().readable().writable(),
                    PageTableLevel::L2,
                ) {
                    panic!("Heap section overlaps with data segments");
                }
            }
        }

        // map the trapframe
        self.page_table.strong_map(
            VirtAddr::from_raw(TRAPFRAME_VADDR as u64),
            PhysAddr::from_raw(self.trapframe as *mut _ as u64),
            PTEFlags::valid().readable().writable(),
            PageTableLevel::L2,
        );
        // map the trampoline
        self.page_table.strong_map(
            VirtAddr::from_raw(TRAMPOLINE_VADDR as u64),
            PhysAddr::from_raw(trampoline as u64),
            PTEFlags::valid().readable().executable(),
            PageTableLevel::L2,
        );

        self.trapframe.sp = stack_pointer as usize;
        self.trapframe.epc = program_counter as usize;
        self.status.store(ProcStatus::Runnable, Ordering::Relaxed);
    }
}

impl ProcTable {
    pub fn new() -> Self {
        ProcTable(core::array::from_fn(|i| {
            Mutex::new(Process::new_inactive(i as ProcId))
        }))
    }

    pub fn alloc_proc(&self, name: &'static str) -> Option<ProcId> {
        for proc in &self.0 {
            if let Some(mut proc) = proc.try_lock() {
                if proc.status.load(Ordering::SeqCst) == ProcStatus::Unused {
                    proc.status.store(ProcStatus::Inactive, Ordering::SeqCst);
                    proc.name = name;
                    return Some(proc.id);
                }
            }
        }
        None
    }
}

impl core::ops::Index<ProcId> for ProcTable {
    type Output = Mutex<Process<'static>>;

    fn index(&self, index: ProcId) -> &Self::Output {
        &self.0[index as usize]
    }
}

pub fn trapframe() -> &'static mut Trapframe {
    unsafe { &mut *(TRAPFRAME_VADDR as *mut Trapframe) }
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

impl Deref for ProcTable {
    type Target = [Mutex<Process<'static>>; NPROC];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
