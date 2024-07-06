use super::{ReadFrom, WriteInto};
use core::arch::asm;

macro_rules! impl_gpt_reg_rw {
    ($name:ident, $t:ty, $abi_name:literal) => {
        impl WriteInto for $name {
            type In = $t;
            unsafe fn write(&self, val: Self::In) {
                unsafe {asm!(concat!("mv ", $abi_name, ", {x}"), x = in(reg) val)};
            }
        }

        impl ReadFrom for $name {
            type Out = $t;
            unsafe fn read(&self, ) -> Self::Out {
                let ret: $t;
                unsafe {asm!(concat!("mv {x}, ", $abi_name), x = out(reg) ret)};
                ret
            }
        }
    };
}

/// General Purpose Register - Thread Pointer (x4)
pub struct Tp;
impl_gpt_reg_rw!(Tp, u64, "tp");
