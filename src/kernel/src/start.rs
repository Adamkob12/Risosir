use crate::{
    arch::*,
    kernelvec::timervec,
    param::{NCPU, STACK_SIZE, TIMER_INTERRUPT_INTERVAL},
    trap::MachineInterrupt,
};
use core::{arch::asm, ptr::addr_of};
use gpr::tp;
use register::{
    medeleg::{self},
    mepc, mhartid, mideleg, mstatus, pmpaddr0, pmpcfg0, satp, sie,
};

// The function `main` is defined in main.rs, but we don't have access to it so we can't reference it directly.
// Fortunately, it must be #[no_mangle], so we can act as though it's defined here.
#[allow(dead_code)]
extern "C" {
    fn main() -> !;
}

/// The stacks of all the CPU cores combined.
/// Each CPU core will use a part of the global stack.
#[repr(C, align(16))]
struct GlobalStack([u8; STACK_SIZE * NCPU]);

/// Init the global stack, dont mangle the name so we can use it from asm.
#[no_mangle]
static mut GLOBAL_STACK: GlobalStack = GlobalStack([0; STACK_SIZE * NCPU]);

#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn start() -> ! {
    // Set Mstatus.MPP to Supervisor, so after calling `mret` we'll end up in Supervisor
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    // Set the Mepc to point to the main function, after calling `mret`, it will start executing.
    mepc::write(main as usize);
    // Disabe paging for now
    satp::write(0);
    // Delegate exception and interrupt handling to S-mode
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

    // setup_timer_interrupts();

    asm!("mret");

    panic!("unreachable");
}

/// The timer interrupt handler will recieve this data:
/// [0..2]: space for timer interrupt handler to save registers
/// [3]: address of the CLINT `mtimecmp` register
/// [4]: interval between timer interrupts (in cycles)
pub type DataToHandleTimerInt = [u64; 5];

/// An instance of [`DataToHandleTimerInt`] for each hart.
static mut TIMER_INTERRUPT_DATA: [DataToHandleTimerInt; NCPU] = [[0; 5]; NCPU];

// Set up timer interrupts
// pub unsafe fn setup_timer_interrupts(cpuid: usize) {
//     // Schedule the next timer interrupt to happen in `TIMER_INTERRUPT_INTERVAL` cycles.
//     Mtimecmp { hart_id }.write(Mtime.read() + TIMER_INTERRUPT_INTERVAL);
//     // Set the correct data for the timer interrupt handler
//     TIMER_INTERRUPT_DATA[hart_id as usize][3] = Mtimecmp { hart_id }.addr_of() as u64;
//     TIMER_INTERRUPT_DATA[hart_id as usize][4] = TIMER_INTERRUPT_INTERVAL;
//     // Set the mscratch register to hold a pointer to the `DataToHandleTiemrInt` for the exact hart.
//     // The mscratch register will be read when the interrupt is triggered.
//     Mscratch.write(addr_of!(TIMER_INTERRUPT_DATA[hart_id as usize]) as u64);
//     // Set all interrupts to be handled by `timervec` (will be changed later)
//     Mtvec.write(timervec as u64);
//     // Enable machine-mode interrupts
//     MstatusMie.write(true);
//     // Enable machine-mode timer interrupts
//     Mie.write(MachineInterrupt::Timer.bitmask());
// }
