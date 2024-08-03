use crate::memlayout::KERNEL_BASE_ADDR;

/// KiloByte
pub const KB: usize = 1024;
/// MegaByte
pub const MB: usize = KB * KB;

/// The maximum amount of devices
pub const NDEV: usize = 10;

/// The maximum amount of CPU cores
pub const NCPU: usize = 8;

// Interrupts

/// Default interval (in cycles) between incoming timer interrupts.
pub const TIMER_INTERRUPT_INTERVAL: usize = 1_000_000;

// Processes

/// Process Id type
pub type ProcId = u8;

pub const PAGE_SIZE: usize = 4096;
pub const PAGES_PER_STACK: usize = 40;
pub const STACK_SIZE: usize = PAGE_SIZE * PAGES_PER_STACK;

pub const PAGES_PER_HEAP: u64 = 1000;
pub const HEAP_SIZE: u64 = PAGES_PER_HEAP * PAGE_SIZE as u64;
/// The start of the heap for a process
pub const HEAP_START: u64 = 0x2200_0000;

/// The maximum amount of active processes at a time
pub const NPROC: usize = ProcId::max_value() as usize;
// pub const NPROC: usize = 100;

pub const RAM_SIZE: usize = 200 * MB;

pub const RAM_END: usize = KERNEL_BASE_ADDR + RAM_SIZE;
