use crate::{arch::memlayout::VIRTIO0, cprintln};

// Taken from spec: https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.pdf#89
// Section 4.2

/// Should be 0x74726976
pub const VIRTIO_MMIO_MAGIC_VALUE: usize = VIRTIO0 + 0x0;
/// Should be 0x2
pub const VIRTIO_MMIO_VERSION_NUM: usize = VIRTIO0 + 0x4;
/// Should be 0x2 (id of block device type)
pub const VIRTIO_MMIO_DEVICE_ID: usize = VIRTIO0 + 0x8;
/// Should be 0x554d4551 (according to qemu)
pub const VIRTIO_MMIO_VENDOR_ID: usize = VIRTIO0 + 0xc;
pub const VIRTIO_MMIO_DEV_FEATURES: usize = VIRTIO0 + 0x10;
pub const VIRTIO_MMIO_FEATURES_SEL: usize = VIRTIO0 + 0x14;
pub const VIRTIO_MMIO_DRIVER_FEATURES: usize = VIRTIO0 + 0x20;
pub const VIRTIO_MMIO_DRIVER_FEATURES_SEL: usize = VIRTIO0 + 0x24;
pub const VIRTIO_MMIO_QUEUE_SEL: usize = VIRTIO0 + 0x30;
pub const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = VIRTIO0 + 0x34;
pub const VIRTIO_MMIO_QUEUE_NUM: usize = VIRTIO0 + 0x38;
pub const VIRTIO_MMIO_QUEUE_READY: usize = VIRTIO0 + 0x44;
pub const VIRTIO_MMIO_QUEUE_NOTIFY: usize = VIRTIO0 + 0x50;
pub const VIRTIO_MMIO_INTERRUPT_STATUS: usize = VIRTIO0 + 0x60;
pub const VIRTIO_MMIO_INTERRUPT_ACK: usize = VIRTIO0 + 0x64;
pub const VIRTIO_MMIO_DEV_STATUS: usize = VIRTIO0 + 0x70;
pub const VIRTIO_MMIO_QUEUE_DESC_LO: usize = VIRTIO0 + 0x80;
pub const VIRTIO_MMIO_QUEUE_DESC_HI: usize = VIRTIO0 + 0x84;
pub const VIRTIO_MMIO_QUEUE_DRIVER_LO: usize = VIRTIO0 + 0x90;
pub const VIRTIO_MMIO_QUEUE_DRIVER_HI: usize = VIRTIO0 + 0x94;
pub const VIRTIO_MMIO_QUEUE_DEV_LO: usize = VIRTIO0 + 0xa0;
pub const VIRTIO_MMIO_QUEUE_DEV_HI: usize = VIRTIO0 + 0xa4;
pub const VIRTIO_MMIO_CONFIG_GEN: usize = VIRTIO0 + 0xfc;
/// The start of the config space
pub const VIRTIO_MMIO_CONFIG: usize = VIRTIO0 + 0x100;

// Section 2.1 of the spec

/// Indicates that the guest OS has found the device and recognized it as a valid virtio device
pub const ACK_STATUS_BIT: u32 = 1 << 0;
/// Indicates that the guest OS knows how to drive the device
pub const DRIVER_STATUS_BIT: u32 = 1 << 1;
/// Indicates that the driver is set up and ready to drive the device
pub const DRIVER_OK_STATUS_BIT: u32 = 1 << 2;
/// Indicates that the driver has acknowledged all the features it understands, and feature
/// negotiation is complete
pub const FEATURES_OK_STATUS_BIT: u32 = 1 << 3;
/// Indicates that the device has experienced an error from which it can’t re-
/// cover.
pub const DEV_NEEDS_RESET_STATUS_BIT: u32 = 1 << 6;
/// Indicates that something went wrong in the guest, and it has given up on the device. This
/// could be an internal error, or the driver didn’t like the device for some reason, or even a fatal error
/// during device operation
pub const DEV_FAILED_STATUS_BIT: u32 = 1 << 7;

fn w_virtio_register<const REG: usize>(val: u32) {
    unsafe { (REG as *mut u32).write_volatile(val) }
}

fn r_virtio_register<const REG: usize>() -> u32 {
    unsafe { (REG as *mut u32).read_volatile() }
}

pub fn init_virtio() {
    // According to section 3.1.1 in the spec

    // Verify that the device is present and matching the expected parameters
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_MAGIC_VALUE>(), 0x74726976);
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_VENDOR_ID>(), 0x554d4551);
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_VERSION_NUM>(), 0x2);
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_DEVICE_ID>(), 0x2);

    let mut status: u32 = 0;

    // 1) Reset the device
    w_virtio_register::<VIRTIO_MMIO_DEV_STATUS>(status);
    // 2) Set the ack status bit
    status |= ACK_STATUS_BIT;
    w_virtio_register::<VIRTIO_MMIO_DEV_STATUS>(status);
    // 3) Set the driver status bit
    status |= DRIVER_STATUS_BIT;
    w_virtio_register::<VIRTIO_MMIO_DEV_STATUS>(status);
    // 4) Negotiate features
    let dev_features = r_virtio_register::<VIRTIO_MMIO_DEV_FEATURES>();
    cprintln!("{:#b}", dev_features);

    //
    todo!()
}
