use crate::{
    arch::*,
    cprint, cprintln,
    kernelvec::timervec,
    memlayout::MTIMECMP_ADDR,
    param::{NCPU, STACK_SIZE, TIMER_INTERRUPT_INTERVAL},
};
use clint::{mtime, mtimecmp};
use core::{arch::asm, ptr::addr_of};
use registers::*;

/// The stacks of all the CPU cores combined.
/// Each CPU core will use a part of the global stack.
#[repr(C, align(16))]
struct GlobalStack([u8; STACK_SIZE * NCPU]);

/// Init the global stack, dont mangle the name so we can use it from asm.
#[no_mangle]
static mut GLOBAL_STACK: GlobalStack = GlobalStack([0; STACK_SIZE * NCPU]);

#[allow(unsafe_op_in_unsafe_fn)]
#[no_mangle]
pub unsafe fn start() -> ! {
    // Set Mstatus.MPP to Supervisor, so after calling `mret` we'll end up in S-mode
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    // Set the Mepc to point to the main function, after calling `mret`, it will start executing.
    #[cfg(not(feature = "test-kernel"))]
    mepc::write(main as u64 as usize);
    #[cfg(feature = "test-kernel")]
    mepc::write(main as usize);
    // Disabe paging for now
    satp::write(0);
    // Delegate exception to S-Mode
    medeleg::set_breakpoint();
    medeleg::set_load_fault();
    medeleg::set_load_fault();
    medeleg::set_user_env_call();
    medeleg::set_load_misaligned();
    medeleg::set_load_page_fault();
    medeleg::set_instruction_fault();
    medeleg::set_illegal_instruction();
    medeleg::set_supervisor_env_call();
    medeleg::set_instruction_misaligned();
    medeleg::set_instruction_page_fault();
    // Delegate interrupts to S-Mode
    mideleg::set_sext();
    mideleg::set_ssoft();
    mideleg::set_stimer();
    // Allow S-mode External, Software & Timer interrupts
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();
    // Configure Physical Memory Protection to give supervisor mode access to all of physical memory.
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);
    // Save the hart id (AKA cpu id) in TP because we won't have access to it outside of machine mode
    let cpuid = mhartid::read();
    tp::write(cpuid);

    if cpuid == 0 {
        unsafe { crate::console::init_console() };
    }

    // The function `main` is defined in main.rs, but we don't have access to it so we can't reference it directly.
    // Fortunately, it must be #[no_mangle], so we can act as though it's defined here.
    extern "C" {
        fn main() -> !;
    }

    setup_timer_interrupts();

    asm!("mret");

    panic!("unreachable");
}

/// The timer interrupt handler will recieve this data:
/// [0..2]: space for timer interrupt handler to save registers
/// [3]: address of the CLINT `mtimecmp` register
/// [4]: interval between timer interrupts (in cycles)
pub type DataToHandleTimerInt = [usize; 5];

/// An instance of [`DataToHandleTimerInt`] for each hart.
static mut TIMER_INTERRUPT_DATA: [DataToHandleTimerInt; NCPU] = [[0; 5]; NCPU];

/// Set up timer interrupts
pub unsafe fn setup_timer_interrupts() {
    let hart_id = tp::read();
    // Schedule the next timer interrupt to happen in `TIMER_INTERRUPT_INTERVAL` cycles.
    mtimecmp::write(hart_id, mtime::read() + TIMER_INTERRUPT_INTERVAL);
    // Set the correct data for the timer interrupt handler
    TIMER_INTERRUPT_DATA[hart_id][3] = MTIMECMP_ADDR + 8 * hart_id;
    TIMER_INTERRUPT_DATA[hart_id][4] = TIMER_INTERRUPT_INTERVAL;
    // Set the mscratch register to hold a pointer to the `DataToHandleTiemrInt` for the exact hart.
    // The mscratch register will be read when the interrupt is triggered.
    mscratch::write(addr_of!(TIMER_INTERRUPT_DATA[hart_id]) as usize);
    // Set all interrupts to be handled by `timervec` (will be changed later)
    mtvec::write(timervec as usize, mtvec::TrapMode::Direct);
    // Enable machine-mode interrupts
    mstatus::set_mie();
    // Enable machine-mode timer interrupts
    mie::set_mtimer();
}
