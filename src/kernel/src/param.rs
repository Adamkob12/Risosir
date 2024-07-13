pub const PAGE_SIZE: usize = 4096;
pub const PAGES_PER_STACK: usize = 10;
pub const STACK_SIZE: usize = PAGE_SIZE * PAGES_PER_STACK;

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
pub const TIMER_INTERRUPT_INTERVAL: u64 = 1_000_000;

// Processes

/// Process Id type
pub type ProcId = u8;

/// The maximum amount of active processes at a time
pub const NPROC: usize = ProcId::max_value() as usize;

pub const RAM_SIZE: usize = 512 * MB;
