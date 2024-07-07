pub const PAGE_SIZE: usize = 4096;
pub const PAGES_PER_STACK: usize = 10;
pub const STACK_SIZE: usize = PAGE_SIZE * PAGES_PER_STACK;

/// The maximum amount of devices
pub const NDEV: usize = 10;

/// The maximum amount of CPU cores
pub const NCPU: usize = 8;

// Interrupts

/// Default interval (in cycles) between incoming timer interrupts.
pub const TIMER_INTERRUPT_INTERVAL: u64 = 1_000_000 * 20;
