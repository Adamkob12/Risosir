pub use _arch::*;

#[macro_export]
macro_rules! impl_gpr_rw {
    ($name:ident, $abi_name:literal) => {
        pub mod $name {
            use core::arch::asm;
            pub unsafe fn write(val: usize) {
                unsafe {asm!(concat!("mv ", $abi_name, ", {x}"), x = in(reg) val)};
            }
            pub unsafe fn read() -> usize {
                let ret: usize;
                unsafe {asm!(concat!("mv {x}, ", $abi_name), x = out(reg) ret)};
                ret
            }
        }
    };
}

#[cfg(target_arch = "riscv64")]
pub mod _arch {
    pub use riscv::*;
    pub mod gpr {
        impl_gpr_rw!(tp, "tp");
    }
}
