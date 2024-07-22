//! This kernel is built to run on qemu-system-riscv64 -machine virt.
//! All of the utilities in this module are defined for this purpose, up to date for Jul 2024.
//! A lot of the info here is from the
//! [`qemu source code`](https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c) and [`qemu 'virt' spec`](https://www.qemu.org/docs/master/system/riscv/virt.html)

// CLINT

use crate::param::{PAGE_SIZE, RAM_SIZE};

/// Qemu-virt defaults to emulate the [`SiFive CLINT`](https://sifive.cdn.prismic.io/sifive%2Fc89f6e5a-cf9e-44c3-a3db-04420702dcc1_sifive+e31+manual+v19.08.pdf)
pub const CLINT_BASE_ADDR: usize = 0x0200_0000;
/// The offset that the `mtimecmp` registers are stored at. For the `mtimecmp` register that corresponds to hard i, add `i * 8` to this offset.
pub const MTIMECMPS_OFFSET: usize = 0x0000_4000;
/// The offset that the `mtime` register is stored at in the clint.
pub const MTIME_OFFSET: usize = 0x0000_bff8;
/// The physical memory address of the `mtime` register.
pub const MTIME_ADDR: usize = CLINT_BASE_ADDR + MTIME_OFFSET;
/// The physical memory address of the `mtimecmp` registers.
pub const MTIMECMP_ADDR: usize = CLINT_BASE_ADDR + MTIMECMPS_OFFSET;

// UART

/// Qemu-virt emulates a single NS16550 compatible UART
pub const UART_BASE_ADDR: usize = 0x1000_0000;

// KERNEL

/// The start of the kernel source code in RAM
pub const KERNEL_BASE_ADDR: usize = 0x8000_0000;

pub const TRAMPOLINE_ADDR: usize = KERNEL_BASE_ADDR + RAM_SIZE - PAGE_SIZE;
pub const TRAPFRAME_ADDR: usize = TRAMPOLINE_ADDR - PAGE_SIZE;

// VIRTIO

// virtio mmio interface
pub const VIRTIO0: usize = 0x1000_1000;
pub const VIRTIO0_IRQ: u32 = 1;

// PLIC

// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC: usize = 0x0C00_0000;
pub const PLIC_PRIORITY: usize = PLIC;
pub const PLIC_PENDING: usize = PLIC + 0x1000;
#[allow(non_snake_case)]
pub const fn PLIC_SENABLE(hart: usize) -> usize {
    PLIC + 0x2080 + hart * 0x100
}
#[allow(non_snake_case)]
pub const fn PLIC_SPRIORITY(hart: usize) -> usize {
    PLIC + 0x201000 + hart * 0x2000
}
#[allow(non_snake_case)]
pub const fn PLIC_SCLAIM(hart: usize) -> usize {
    PLIC + 0x201004 + hart * 0x2000
}
