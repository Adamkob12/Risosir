use crate::{
    arch::{
        common::{disable_paging, privilage::PrivLevel},
        registers::{
            csr::{
                Medeleg, Mepc, Mhartid, Mideleg, Mie, Mscratch, MstatusMie, MstatusMpp, Mtvec,
                Pmpaddr0, Pmpcfg0, Satp, Sie, SIE_SEIE, SIE_SSIE, SIE_STIE,
            },
            gpr::Tp,
            mmapped::{Mtime, Mtimecmp},
            AddressOf, ReadFrom, WriteInto,
        },
    },
    kernelvec::timervec,
    param::{NCPU, STACK_SIZE, TIMER_INTERRUPT_INTERVAL},
    trap::Exception,
};
use core::{arch::asm, ptr::addr_of};

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
    MstatusMpp.write(PrivLevel::S);
    // Set the Mepc to point to the main function, after calling `mret`, it will start executing.
    Mepc.write(main as u64);
    // Disabe paging for now
    Satp.write(0);
    // Delegate exception and interrupt handling to S-mode
    Medeleg.write(0xffff);
    Mideleg.write(0xffff);
    // Enable Software, External and Timer interrupts
    Sie.write(Sie.read() | SIE_SEIE | SIE_SSIE | SIE_STIE);
    // Sie.write(Sie.read() | SIE_SEIE | SIE_SSIE);
    // Configure Physical Memory Protection to give supervisor mode access to all of physical memory.
    Pmpaddr0.write(0x3fffffffffffff);
    Pmpcfg0.write(0xf);

    // Save the hart id in TP because we won't have access to it outside of machine mode
    let hart_id = Mhartid.read();
    Tp.write(hart_id);

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
pub type DataToHandleTimerInt = [u64; 5];

/// An instance of [`DataToHandleTimerInt`] for each hart.
static mut TIMER_INTERRUPT_DATA: [DataToHandleTimerInt; NCPU] = [[0; 5]; NCPU];

/// Set up timer interrupts
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn setup_timer_interrupts() {
    // Get hart id
    let hart_id = Mhartid.read();
    // Schedule the next timer interrupt to happen in `TIMER_INTERRUPT_INTERVAL` cycles.
    Mtimecmp { hart_id }.write(Mtime.read() + TIMER_INTERRUPT_INTERVAL);
    // Set the correct data for the timer interrupt handler
    TIMER_INTERRUPT_DATA[hart_id as usize][3] = Mtimecmp { hart_id }.addr_of() as u64;
    TIMER_INTERRUPT_DATA[hart_id as usize][4] = TIMER_INTERRUPT_INTERVAL;
    // Set the mscratch register to hold a pointer to the `DataToHandleTiemrInt` for the exact hart.
    // The mscratch register will be read when the interrupt is triggered.
    Mscratch.write(addr_of!(TIMER_INTERRUPT_DATA[hart_id as usize]) as u64);
    // Set all interrupts to be handled by `timervec` (will be changed later)
    Mtvec.write(timervec as u64);
    // Enable machine-mode interrupts
    MstatusMie.write(true);
    // Enable machine-mode timer interrupts
    Mie.write(Exception::MachineTimerInterrupt.bitmask());
}
