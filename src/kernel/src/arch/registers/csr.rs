use super::{ReadFrom, WriteInto};
use crate::arch::common::privilage::PrivLevel;
use core::arch::asm;

macro_rules! impl_csr_reg_rw {
    ($name:ident, $t:ty, $abi_name:literal) => {
        impl WriteInto for $name {
            type In = $t;
            unsafe fn write(&self, val: Self::In) {
                unsafe {asm!(concat!("csrw ", $abi_name, ", {x}"), x = in(reg) val)};
            }
        }

        impl ReadFrom for $name {
            type Out = $t;
            unsafe fn read(&self, ) -> Self::Out {
                let ret: $t;
                unsafe {asm!(concat!("csrr {x}, ", $abi_name), x = out(reg) ret)};
                ret
            }
        }
    };
}

/// CSR register that stores info about the execution status of the current hart.
pub struct Mstatus;
impl_csr_reg_rw!(Mstatus, u64, "mstatus");

/// CSR register that stores the id of the current executing hart.
pub struct Mhartid;
impl_csr_reg_rw!(Mhartid, u64, "mhartid");

/// CSR register that holds the adress will be jumped to after `mret` is called.
pub struct Mepc;
impl_csr_reg_rw!(Mepc, u64, "mepc");

/// CSR register that holds info for handling interrupts
pub struct Mscratch;
impl_csr_reg_rw!(Mscratch, u64, "mscratch");

/// CSR register that holds data to where to jump on interrupts / exceptions
pub struct Mtvec;
impl_csr_reg_rw!(Mtvec, u64, "mtvec");

/// CSR register that holds machine-mode interrupt enable bits
pub struct Mie;
impl_csr_reg_rw!(Mie, u64, "mie");

/// CSR register that stores information about which exceptions should be delegated to S-mode.
pub struct Medeleg;
impl_csr_reg_rw!(Medeleg, u64, "medeleg");

/// CSR register that stores information about which interrupts should be delegated to S-mode.
pub struct Mideleg;
impl_csr_reg_rw!(Mideleg, u64, "mideleg");

/// CSR register that contains the exception sepcific info on trap
pub struct Mtval;
impl_csr_reg_rw!(Mtval, u64, "mtval");

/// CSR register that contains the cause of the exception
pub struct Mcause;
impl_csr_reg_rw!(Mcause, u64, "mcause");

/// CSR register that contains information about paging.
pub struct Satp;
impl_csr_reg_rw!(Satp, u64, "satp");

/// CSR register that contains the cause of the exception
pub struct Scause;
impl_csr_reg_rw!(Scause, u64, "scause");

/// CSR register that stores info about the execution status of the current hart.
pub struct Sstatus;
impl_csr_reg_rw!(Sstatus, u64, "sstatus");

/// CSR register that contains the exception sepcific info on trap
pub struct Stval;
impl_csr_reg_rw!(Stval, u64, "stval");

/// CSR register that contains the trap handler address
pub struct Stvec;
impl_csr_reg_rw!(Stvec, u64, "stvec");

/// CSR register that contains the address that created the exception
pub struct Sepc;
impl_csr_reg_rw!(Sepc, u64, "sepc");

/// CSR register that stores interrupt enable bits (interrupt `i` will not be accepted unless Sie[i] is on)
pub struct Sie;
impl_csr_reg_rw!(Sie, u64, "sie");

/// CSR register that stores information on pending interrupts.
pub struct Sip;
impl_csr_reg_rw!(Sip, u64, "sip");

/// CSR for Physical Memory Protection.
pub struct Pmpaddr0;
impl_csr_reg_rw!(Pmpaddr0, u64, "pmpaddr0");

/// CSR for configuring pmpaddr0,1,2,3,4,5,6,7
pub struct Pmpcfg0;
impl_csr_reg_rw!(Pmpcfg0, u64, "pmpcfg0");

/// Data stored inside Mstatus[12:11]
pub struct MstatusMpp;
const MPP_MASK: u64 = 3u64 << 11;

impl ReadFrom for MstatusMpp {
    type Out = PrivLevel;
    unsafe fn read(&self) -> Self::Out {
        let mstatus = unsafe { Mstatus.read() };
        let mpp = mstatus & MPP_MASK;
        PrivLevel::try_from(mpp >> 11).unwrap()
    }
}

impl WriteInto for MstatusMpp {
    type In = PrivLevel;
    unsafe fn write(&self, val: Self::In) {
        let mut mstatus = unsafe { Mstatus.read() };
        mstatus &= !MPP_MASK;
        mstatus |= (val as u64) << 11;
        unsafe { Mstatus.write(mstatus) };
    }
}

/// Data stored inside Mstatus[3]
pub struct MstatusMie;
const MIE_MASK: u64 = 1 << 3;

impl ReadFrom for MstatusMie {
    type Out = bool;
    unsafe fn read(&self) -> Self::Out {
        let mstatus = unsafe { Mstatus.read() };
        let mie = mstatus & MIE_MASK;
        mie > 0
    }
}

impl WriteInto for MstatusMie {
    type In = bool;
    unsafe fn write(&self, val: Self::In) {
        let mut mstatus = unsafe { Mstatus.read() };
        mstatus &= !MIE_MASK;
        mstatus |= (val as u64) << 3;
        unsafe { Mstatus.write(mstatus) };
    }
}
