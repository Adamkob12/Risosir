use super::{ReadFrom, WriteInto};
use crate::arch::common::privilage::PrivLevel;
use core::arch::asm;

macro_rules! impl_standard_reg {
    ($name:ident, $t:ty, $abi_name:literal) => {
        impl WriteInto for $name {
            type In = $t;
            unsafe fn write(val: Self::In) {
                asm!(concat!("csrw ", $abi_name, ", {x}"), x = in(reg) val);
            }
        }

        impl ReadFrom for $name {
            type Out = $t;
            unsafe fn read() -> Self::Out {
                let ret: $t;
                asm!(concat!("csrr {x}, ", $abi_name), x = out(reg) ret);
                ret
            }
        }
    };
}

/// CSR register that stores info about the execution status of the current hart.
pub struct Mstatus;
impl_standard_reg!(Mstatus, u64, "mstatus");

/// CSR register that holds the adress will be jumped to after `mret` is called.
pub struct Mepc;
impl_standard_reg!(Mepc, u64, "mepc");

/// CSR register that contains information about paging.
pub struct Satp;
impl_standard_reg!(Satp, u64, "satp");

/// CSR register that stores information about which exceptions should be delegated to S-mode.
pub struct Medeleg;
impl_standard_reg!(Medeleg, u64, "medeleg");

/// CSR register that stores information about which interrupts should be delegated to S-mode.
pub struct Mideleg;
impl_standard_reg!(Mideleg, u64, "mideleg");

/// CSR register that stores interrupt enable bits (interrupt `i` will not be accepted unless Sie[i] is on)
pub struct Sie;
impl_standard_reg!(Sie, u64, "sie");
/// Corresponds to External Interrupts bit in [`SIE`].
pub const SIE_SEIE: u64 = 1 << 9;
/// Corresponds to Timer Interrupts bit in [`SIE`].
pub const SIE_STIE: u64 = 1 << 5;
/// Corresponds to Software Interrupts bit in [`SIE`].
pub const SIE_SSIE: u64 = 1 << 1;

/// CSR register that stores information on pending interrupts.
pub struct Sip;
impl_standard_reg!(Sip, u64, "sip");

pub struct Pmpaddr0;
impl_standard_reg!(Pmpaddr0, u64, "pmpaddr0");

pub struct Pmpcfg0;
impl_standard_reg!(Pmpcfg0, u64, "pmpcfg0");

/// Data stored inside Mstatus
pub struct Mpp;
const MPP_MASK: u64 = 3u64 << 11;

impl ReadFrom for Mpp {
    type Out = PrivLevel;
    unsafe fn read() -> Self::Out {
        let mstatus = Mstatus::read();
        let mpp = mstatus & MPP_MASK;
        PrivLevel::try_from(mpp >> 11).unwrap()
    }
}

impl WriteInto for Mpp {
    type In = PrivLevel;
    unsafe fn write(val: Self::In) {
        let mut mstatus = Mstatus::read();
        mstatus &= !MPP_MASK;
        mstatus |= (val as u64) << 11;
        Mstatus::write(mstatus);
    }
}
