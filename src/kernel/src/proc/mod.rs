use crate::param::{ProcId, NPROC};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use spin::RwLock;

#[derive(Clone, Copy)]
pub enum ProcStatus {
    Running,
    Inactive,
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct Process {
    /// The name of the process, inactive processes are named "X"
    name: &'static str,
    /// Indexes [`ProcTable`]
    id: ProcId,
    /// THe status of the process
    status: ProcStatus,
}

const INACTIVE_PROC_NAME: &str = "X";

pub struct ProcTable([Arc<RwLock<Process>>; NPROC]);

pub static PROCS: OnceCell<ProcTable> = OnceCell::uninit();

pub unsafe fn init_procs() {
    PROCS.init_once(|| ProcTable::new());
}

impl Process {
    pub fn new_inactive(id: ProcId) -> Self {
        Process {
            name: INACTIVE_PROC_NAME,
            id,
            status: ProcStatus::Inactive,
        }
    }
}

impl ProcTable {
    pub fn new() -> Self {
        ProcTable(core::array::from_fn(|i| {
            Arc::new(RwLock::new(Process::new_inactive(i as ProcId)))
        }))
    }
}

impl core::ops::Index<ProcId> for ProcTable {
    type Output = Arc<RwLock<Process>>;

    fn index(&self, index: ProcId) -> &Self::Output {
        &self.0[index as usize]
    }
}
