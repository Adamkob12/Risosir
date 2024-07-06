// This kernel is built to run on qemu-system-riscv64 -machine virt.
// All of the utilities in this module are defined for this purpose, up to date for Jul 2024.

/// Qemu-virt defaults to emulate the [`SiFive CLINT`](https://sifive.cdn.prismic.io/sifive%2Fc89f6e5a-cf9e-44c3-a3db-04420702dcc1_sifive+e31+manual+v19.08.pdf)
pub const CLINT_BASE_ADDR: usize = 0x0200_0000;

/// The offset that the `mtime` register is stored at in the clint.
pub const MTIME_OFFSET: usize = 0x0000_bff8;

/// The offset that the `mtimecmp` registers are stored at. For the `mtimecmp` register that corresponds to hard i, add `i * 8` to this offset.
pub const MTIMECMPS_OFFSET: usize = 0x0000_4000;

/// The physical memory address of the `mtime` register.
pub const MTIME_ADDR: usize = CLINT_BASE_ADDR + MTIME_OFFSET;

/// The physical memory address of the `mtimecmp` registers.
pub const MTIMECMP_ADDR: usize = CLINT_BASE_ADDR + MTIMECMPS_OFFSET;
