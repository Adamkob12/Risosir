use crate::{
    arch::{
        common::{disable_paging, privilage::PrivLevel},
        registers::{
            csr::{
                Medeleg, Mepc, Mideleg, Mpp, Pmpaddr0, Pmpcfg0, Sie, SIE_SEIE, SIE_SSIE, SIE_STIE,
            },
            ReadFrom, WriteInto,
        },
    },
    param::{NCPU, STACK_SIZE},
};

/// The stacks of all the CPU cores combined.
/// Each CPU core will use a part of the global stack.
#[repr(C, align(16))]
struct GlobalStack([u8; STACK_SIZE * NCPU]);

/// Init the global stack, dont mangle the name so we can use it from asm.
#[no_mangle]
static mut GLOBAL_STACK: GlobalStack = GlobalStack([0; STACK_SIZE * NCPU]);

pub unsafe fn start() -> ! {
    // Set MPP to Supervisor, so after calling `mret` we'll end up in Supervisor
    Mpp::write(PrivLevel::S);

    // The function `main` is defined in main.rs, but we don't have access to it so we can't reference it directly.
    // Fortunately, it must be #[no_mangle], so we can act as though it's defined here.
    extern "C" {
        fn main() -> !;
    }

    // Set the MPEC to point to the main function, after calling `mret`, it will start executing.
    Mepc::write(main as u64);

    // Disabe paging for now
    disable_paging();

    // Delegate exception and interrupt handling to S-mode
    Medeleg::write(u64::MAX); // TODO: maybe u16::MAX instead
    Mideleg::write(u64::MAX);

    // Enable Software, External and Timer interrupts
    Sie::write(Sie::read() | SIE_SEIE | SIE_SSIE | SIE_STIE);

    // Configure Physical Memory Protection to give supervisor mode access to all of physical memory.
    Pmpaddr0::write(u64::MAX); // TODO: maybe 0x3fffffffffffff instead
    Pmpcfg0::write(u8::MAX as u64);

    todo!()
}
