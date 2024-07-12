use spin::Mutex;

use crate::param::{ProcId, NPROC};

#[derive(Clone, Copy)]
pub enum ProcStatus {
    Running,
}

#[derive(Clone, Copy)]
pub struct Process {
    name: &'static str,
    id: ProcId,
    parent_id: ProcId,
    status: ProcStatus,
}

/// Indexed by [`ProcId`]
pub static PROC_TABLE: Mutex<[Option<Process>; NPROC]> = Mutex::new([None; NPROC]);
