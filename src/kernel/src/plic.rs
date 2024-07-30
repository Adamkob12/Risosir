use riscv_peripheral::plic::{claim::CLAIM, Plic};

use crate::memlayout::{
    PLIC, PLIC_CLAIM_BASE, PLIC_CLAIM_THRESHOLD, PLIC_ENABLE_BASE, PLIC_PRIORITY_BASE, UART_IRQ,
    VIRTIO0_IRQ,
};

pub const PLIC_CLAIM: CLAIM = unsafe { CLAIM::new(PLIC_CLAIM_BASE) };

const fn plic_priority(source_id: usize) -> *mut u32 {
    (PLIC_PRIORITY_BASE + source_id * 4) as *mut u32
}

fn plic_enable(source_id: usize, context_id: usize) {
    if source_id > 31 {
        panic!("Only 31 devices allowed")
    }
    let p = (PLIC_ENABLE_BASE + context_id as usize * 0x80) as *mut u32;
    unsafe { p.write_volatile(p.read_volatile() | (1 << source_id)) };
}

/// Basic inititializatin of the PLIC for all the cores
pub fn init_plic_global() {
    // Set the priority of the UART and VIRTIO to 1
    unsafe {
        plic_priority(UART_IRQ).write_volatile(1);
        plic_priority(VIRTIO0_IRQ).write_volatile(1);
    };
}

/// Basic inititializatin of the PLIC for the current core
/// Enables this core to recieve UART and VIRTIO
/// Only for S-Mode
pub fn init_plic_hart(hart_id: usize) {
    let context_id = hart_id;

    let p = (PLIC_ENABLE_BASE + context_id as usize * 0x80) as *mut u32;
    unsafe { p.write_volatile((1 << UART_IRQ) | (1 << VIRTIO0_IRQ)) };

    unsafe {
        ((PLIC_CLAIM_THRESHOLD + 0x1000 * context_id as usize) as *mut u32).write_volatile(0)
    };
}

/// Returns `Some(device_id)` if the claim for this target was successful
pub fn plic_claim(hart_id: usize) -> Option<usize> {
    let context_id = hart_id;
    let p = (PLIC_CLAIM_BASE + context_id as usize * 0x1000) as *mut u32;

    match unsafe { p.read_volatile() } {
        0 => None,
        id => Some(id as usize),
    }
}

/// Signal that we completed servicing the interrupt
pub fn plic_complete(hart_id: usize, irq: usize) {
    let context_id = hart_id;
    let p = (PLIC_CLAIM_BASE + context_id as usize * 0x1000) as *mut u32;
    unsafe { p.write_volatile(irq as u32) };
}
