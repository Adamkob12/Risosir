use crate::{
    arch::registers::tp,
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
use core::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
};

const INACTIVE_PROC_NAME: &str = "X";

pub static mut PROCS_ADDR: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum ProcStatus {
    Unused = 0,
    Inactive = 1,
    Runnable = 2,
    Running = 3,
}

#[repr(transparent)]
pub struct AtomicProcStatus(AtomicUsize);

pub struct Process {
    /// The name of the process, inactive processes are named "X"
    name: Cell<&'static str>,
    /// Indexes [`ProcTable`]
    pub id: ProcId,
    /// The status of the process
    pub status: AtomicProcStatus,
    /// After [`init_procs`] is called, must be valid.
    pub kernel_stack: *mut [u8; STACK_SIZE],
    /// After [`init_procs`] is called, must be valid.
    pub page_table: *mut PageTable,
    /// After [`init_procs`] is called, must be valid.
    pub trapframe: *mut Trapframe,
}

pub struct ProcTable([Process; NPROC]);

pub fn init_procs() {
    let procs = Box::leak::<'static>(Box::new(ProcTable(core::array::from_fn(|idx| {
        Process::new_inactive(idx as u8)
    }))));
    unsafe { PROCS_ADDR = procs as *mut _ as usize };
}

pub fn procs<'a>() -> &'a ProcTable {
    unsafe { &*(PROCS_ADDR as *const _) }
}

pub fn proc<'a>(id: ProcId) -> &'a Process {
    &procs()[id]
}

pub fn cpuid() -> usize {
    // s_without_interrupts(|| tp::read())
    tp::read()
}

impl Process {
    fn new_inactive(id: ProcId) -> Self {
        let pt: &mut PageTable = Box::leak(unsafe { Box::new_zeroed().assume_init() });
        let tf: &mut Trapframe = Box::leak(unsafe { Box::new_zeroed().assume_init() });
        let ks: &mut [u8; STACK_SIZE] = Box::leak(unsafe { Box::new_zeroed().assume_init() });
        Process {
            name: Cell::new(INACTIVE_PROC_NAME),
            id,
            status: AtomicProcStatus::new(ProcStatus::Unused),
            page_table: pt as *mut _,
            trapframe: tf as *mut _,
            kernel_stack: ks as *mut _,
        }
    }

    pub fn name(&self) -> &str {
        self.name.get()
    }

    pub fn pagetable<'a>(&'a self) -> &'a PageTable {
        unsafe { self.page_table.as_ref() }
            .expect("init_procs wasn't called before trying to access the process")
    }

    pub fn trapframe<'a>(&'a self) -> &'a Trapframe {
        unsafe { self.trapframe.as_ref() }
            .expect("init_procs wasn't called before trying to access the process")
    }

    /// After calling this function, the process will be ready to run
    pub fn activate<'a>(&self, exe: ParsedExecutable<'a>) {
        if self
            .status
            .compare_exchange(
                ProcStatus::Inactive,
                ProcStatus::Runnable,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            let pt = unsafe { self.page_table.as_mut().unwrap() };
            let tf = unsafe { self.trapframe.as_mut().unwrap() };
            // The program counter needs to start at the start of the code section
            let program_counter = exe.entry_point as u64;
            let file_base = exe.file_data.as_ptr() as usize;
            let mut data_end = 0;
            // Map the text section (code of the process), it needs to be readable and executable
            for seg in exe.segs {
                // cprintln!("{:#?}", seg);
                let flags = PTEFlags::valid()
                    .readable()
                    .writable()
                    .executable()
                    .userable();
                // if seg.p_flags & (1 << 2) != 0 {
                //     flags = flags.readable();
                // }
                // if seg.p_flags & (1 << 1) != 0 {
                //     flags = flags.writable();
                // }
                // if seg.p_flags & (1 << 0) != 0 {
                //     flags = flags.executable();
                // }
                let vaddr_base = seg.p_vaddr;
                let size = seg.p_memsz;

                for offset in (0..size).into_iter().step_by(PAGE_SIZE) {
                    let va = vaddr_base + offset;
                    let pa = file_base as u64 + offset + seg.p_offset;
                    pt.strong_map(
                        VirtAddr::from_raw(va),
                        PhysAddr::from_raw(pa),
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
                    if let Some(_) = pt.strong_map(
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
                    if let Some(_) = pt.strong_map(
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
            pt.strong_map(
                VirtAddr::from_raw(TRAPFRAME_VADDR as u64),
                PhysAddr::from_raw(self.trapframe as u64),
                PTEFlags::valid().readable().writable(),
                PageTableLevel::L2,
            );
            // map the trampoline
            pt.strong_map(
                VirtAddr::from_raw(TRAMPOLINE_VADDR as u64),
                PhysAddr::from_raw(trampoline as u64),
                PTEFlags::valid().readable().executable(),
                PageTableLevel::L2,
            );

            tf.sp = stack_pointer as usize;
            tf.epc = program_counter as usize;
        } else {
            panic!("Can't activate Unused or Active Proc");
        }
    }
}

impl ProcTable {
    pub fn new() -> Self {
        ProcTable(core::array::from_fn(|i| Process::new_inactive(i as ProcId)))
    }

    pub fn alloc_proc(&self, name: &'static str) -> Option<ProcId> {
        for proc in &self.0 {
            if proc
                .status
                .compare_exchange(
                    ProcStatus::Unused,
                    ProcStatus::Inactive,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                proc.name.replace(name);
                return Some(proc.id);
            }
        }
        None
    }
}

impl core::ops::Index<ProcId> for ProcTable {
    type Output = Process;

    fn index(&self, index: ProcId) -> &Self::Output {
        &self.0[index as usize]
    }
}

/// The saved values of the registers while executing in user mode.
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
