pub use _arch::*;

#[macro_export]
macro_rules! impl_gpr_rw {
    ($name:ident, $abi_name:literal) => {
        pub mod $name {
            use core::arch::asm;
            pub unsafe fn write(val: usize) {
                unsafe {asm!(concat!("mv ", $abi_name, ", {x}"), x = in(reg) val)};
            }
            pub fn read() -> usize {
                let ret: usize;
                unsafe {asm!(concat!("mv {x}, ", $abi_name), x = out(reg) ret)};
                ret
            }
        }
    };
}

#[cfg(target_arch = "riscv64")]
pub mod _arch {
    pub mod asm {
        pub use riscv::asm::*;
    }
    pub mod registers {
        pub mod gpr {
            impl_gpr_rw!(tp, "tp");
            impl_gpr_rw!(ra, "ra");
            impl_gpr_rw!(sp, "sp");
        }
        pub use gpr::*;
        pub use riscv::register::*;
    }
    pub mod interrupts {
        pub use riscv::interrupt::supervisor::disable as s_disable;
        pub use riscv::interrupt::supervisor::enable as s_enable;
        pub use riscv::interrupt::supervisor::free as s_without_interrupts;
        pub use riscv::interrupt::supervisor::nested as s_with_interrupts;
    }
    pub mod clint {
        pub mod mtime {
            use crate::memlayout::MTIME_ADDR;

            pub fn read() -> usize {
                unsafe { (MTIME_ADDR as *const usize).read_volatile() }
            }
        }

        pub mod mtimecmp {
            use crate::memlayout::MTIMECMP_ADDR;

            pub fn write(cpuid: usize, val: usize) {
                unsafe { ((MTIMECMP_ADDR + 8 * cpuid) as *mut usize).write_volatile(val) };
            }
        }
    }
}
