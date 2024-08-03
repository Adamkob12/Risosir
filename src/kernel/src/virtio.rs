use crate::memlayout::VIRTIO0;
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::{mem::MaybeUninit, sync::atomic::fence};
use riscv::asm::wfi;
use spin::Mutex;

//
// In this file, `spec` refers to:
// https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.pdf#89
//

/// The capacity (refered to as `QueueNum` in the spec) of the virtqueue.
pub const VIRTQ_CAP: usize = 8;

pub const SECTOR_SIZE: usize = 512;

/// The global disk instance
pub static DISK: OnceCell<Mutex<VirtioDisk>> = OnceCell::uninit();

// MMIO Device Register Layout -- Section 4.2 of the spec

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

// Status bits of virtio devices -- Section 2.1 of the spec

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

// Feature bits of the block device -- Section 5.2.3 of the spec

/// Device is read-only.
pub const VIRTIO_BLK_F_RO: u32 = 1 << 5;
/// Device can toggle its cache between writeback and writethrough modes
pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 1 << 11;
/// Block size of disk is in blk_size.
pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 1 << 6;
pub const VIRTIO_BLK_F_RING_INDIRECT_DESC: u32 = 1 << 28;
pub const VIRTIO_BLK_F_RING_EVENT_IDX: u32 = 1 << 29;

// Request types of the block device -- Section 5.2.6 of the spec

/// Read data from the disk into the buffer
pub const VIRTIO_BLK_T_IN: u32 = 0;
/// Write data from the buffer into the disk
pub const VIRTIO_BLK_T_OUT: u32 = 0;

// Status values for block device requests -- Section 5.2.6 of the spec

pub const VIRTIO_BLK_S_OK: u8 = 0;
pub const VIRTIO_BLK_S_IOERR: u8 = 1;

// Flag values for buffer descriptors -- Section 2.6.5 of the spec

pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;

#[repr(C)]
pub struct VirtioDisk {
    desc_table: &'static mut [VirtqDesc; VIRTQ_CAP],
    avail_ring: &'static mut VirtqAvail,
    used_ring: &'static mut VirtqUsed,
    /// Indexed by idx % VIRTQ_CAP, is the desc free to use?
    free_desc: [bool; VIRTQ_CAP],
    req_placeholder: [MaybeUninit<VirtioBlkReq>; VIRTQ_CAP],
    // statuses: [u8; VIRTQ_CAP],
    used_idxs: u16,
}

/// A descriptor in the virtqueue descriptor table
/// 2.6.5 in the spec
#[repr(C, align(16))]
pub struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

/// The virtqueue available ring
/// 2.6.6 in the spec
#[repr(C, align(2))]
pub struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; VIRTQ_CAP],
    used_event: u16,
}

#[repr(C, align(4))]
pub struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; VIRTQ_CAP],
    avail_event: u16,
}

#[repr(C)]
pub struct VirtqUsedElem {
    /// Index of the start of the used descriptor chain
    desc_id: u32,
    len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtioBlkReq {
    ty: u32,
    _reserved: u32,
    /// The sector on the disk
    sector: u64,
}

fn w_virtio_register<const REG: usize>(val: u32) {
    unsafe { (REG as *mut u32).write_volatile(val) }
}

fn r_virtio_register<const REG: usize>() -> u32 {
    unsafe { (REG as *mut u32).read_volatile() }
}

pub fn init_virtio() {
    // Verify that the device is present and matching the expected parameters
    // (4.2.3.1.1 in the spec)
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_MAGIC_VALUE>(), 0x74726976);
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_VENDOR_ID>(), 0x554d4551);
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_VERSION_NUM>(), 0x2);
    assert_eq!(r_virtio_register::<VIRTIO_MMIO_DEVICE_ID>(), 0x2);

    // The following steps are according to section 3.1.1 in the spec
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
    let mut features = r_virtio_register::<VIRTIO_MMIO_DEV_FEATURES>();
    // #[cfg(debug_assertions)]
    // cprintln!("{:#32b}", features);
    features &= !(VIRTIO_BLK_F_RO);
    features &= !(VIRTIO_BLK_F_CONFIG_WCE);
    features &= !(VIRTIO_BLK_F_RING_EVENT_IDX);
    features &= !(VIRTIO_BLK_F_RING_INDIRECT_DESC);
    // write the negotiated features back to the device
    w_virtio_register::<VIRTIO_MMIO_DRIVER_FEATURES>(features);

    // 5) Set the FEATURES_OK status bit
    status |= FEATURES_OK_STATUS_BIT;
    w_virtio_register::<VIRTIO_MMIO_DEV_STATUS>(status);

    // 6) Re-read dev status to make sure the features have been accepted
    assert_ne!(
        r_virtio_register::<VIRTIO_MMIO_DEV_STATUS>() & FEATURES_OK_STATUS_BIT,
        0,
        "VirtIO-MMIO Block device has rejected the features"
    );

    // 7) Device specifc setup
    {
        // The following steps are according to section 4.2.3.2 in the spec

        // 1) Select virtqueue 0 (its the only one we have acoording to 5.2.2)
        w_virtio_register::<VIRTIO_MMIO_QUEUE_SEL>(0);

        // 2) Make sure the queue isn't in use
        assert_eq!(r_virtio_register::<VIRTIO_MMIO_QUEUE_READY>(), 0x0);

        // 3)
        let max_queue_size = r_virtio_register::<VIRTIO_MMIO_QUEUE_NUM_MAX>();
        assert_ne!(max_queue_size, 0);
        assert!(max_queue_size > VIRTQ_CAP as u32);

        // 4) Allocate the memory for the queue
        let desc_table =
            unsafe { Box::leak(Box::<[VirtqDesc; VIRTQ_CAP]>::new_zeroed()).assume_init_mut() };
        let avail_ring = unsafe { Box::leak(Box::<VirtqAvail>::new_zeroed()).assume_init_mut() };
        let used_ring = unsafe { Box::leak(Box::<VirtqUsed>::new_zeroed()).assume_init_mut() };

        // 5) Update QueueNum
        w_virtio_register::<VIRTIO_MMIO_QUEUE_NUM>(VIRTQ_CAP as u32);

        // 6) Write the memory address of the virtq parts to the appropriate registers
        let desc_area_addr = desc_table as *mut _ as usize;
        let driver_area_addr = avail_ring as *mut _ as usize;
        let device_area_addr = used_ring as *mut _ as usize;
        w_virtio_register::<VIRTIO_MMIO_QUEUE_DESC_LO>(desc_area_addr as u32);
        w_virtio_register::<VIRTIO_MMIO_QUEUE_DESC_HI>((desc_area_addr >> 32) as u32);
        w_virtio_register::<VIRTIO_MMIO_QUEUE_DRIVER_LO>(driver_area_addr as u32);
        w_virtio_register::<VIRTIO_MMIO_QUEUE_DRIVER_HI>((driver_area_addr >> 32) as u32);
        w_virtio_register::<VIRTIO_MMIO_QUEUE_DEV_LO>(device_area_addr as u32);
        w_virtio_register::<VIRTIO_MMIO_QUEUE_DEV_HI>((device_area_addr >> 32) as u32);

        // 7) Write 0x1 to QueueReady
        w_virtio_register::<VIRTIO_MMIO_QUEUE_READY>(0x1);

        DISK.init_once(|| {
            Mutex::new(VirtioDisk {
                desc_table,
                avail_ring,
                used_ring,
                free_desc: [true; VIRTQ_CAP],
                req_placeholder: [MaybeUninit::zeroed(); VIRTQ_CAP],
                // statuses: [0; VIRTQ_CAP],
                used_idxs: 0,
            })
        });
    }

    // 8) Driver OK
    status |= DRIVER_OK_STATUS_BIT;
    w_virtio_register::<VIRTIO_MMIO_DEV_STATUS>(status);

    // The device is live!
}

impl VirtioDisk {
    fn alloc_desc(&mut self) -> Option<u16> {
        for desc_id in 0..VIRTQ_CAP {
            if self.free_desc[desc_id] {
                self.free_desc[desc_id] = false;
                return Some(desc_id as u16);
            }
        }
        None
    }

    fn free_desc_chain(&mut self, desc: u16) {
        if self.free_desc[desc as usize] {
            panic!("Tried freeing free desc");
        }
        self.free_desc[desc as usize] = true;
        if self.desc_table[desc as usize].flags & VIRTQ_DESC_F_NEXT != 0 {
            self.free_desc_chain(self.desc_table[desc as usize].next);
        }
    }
}

pub fn read_from_disk(sector: u64, data: &mut [u8; 1024]) -> Result<(), u8> {
    let status: u8 = 0xff;
    let status_addr: u64 = (&status) as *const _ as u64;
    let head_desc_chain = {
        let mut disk = DISK.get().unwrap().lock();
        let desc_id1 = disk.alloc_desc().ok_or(0)?;
        let desc_id2 = disk.alloc_desc().ok_or(0)?;
        let desc_id3 = disk.alloc_desc().ok_or(0)?;

        // disk.statuses[desc_id1 as usize] = 0xff;
        // let status_addr: u64 = (&disk.statuses[desc_id1 as usize]) as *const _ as u64;

        let req = &mut disk.req_placeholder[desc_id1 as usize];
        let req_addr = req.write(VirtioBlkReq {
            ty: VIRTIO_BLK_T_IN,
            _reserved: 0,
            sector,
        }) as *mut _ as u64;

        // The first descriptor - the header, it describes the first of the chain
        let desc1 = &mut disk.desc_table[desc_id1 as usize];
        desc1.addr = req_addr;
        desc1.len = size_of::<VirtioBlkReq>() as u32;
        desc1.flags = VIRTQ_DESC_F_NEXT;
        desc1.next = desc_id2;

        // The second descriptor - this descriptor defines the data buffer
        let desc2 = &mut disk.desc_table[desc_id2 as usize];
        desc2.addr = data as *mut _ as u64;
        desc2.len = data.len() as u32;
        desc2.flags = VIRTQ_DESC_F_NEXT | VIRTQ_DESC_F_WRITE;
        desc2.next = desc_id3;

        // The third buffer - this descriptor defines a 1 byte buffer that the device
        // will write the status of the operation after it ends
        let desc3 = &mut disk.desc_table[desc_id3 as usize];
        desc3.addr = status_addr;
        desc3.len = 1;
        desc3.flags = VIRTQ_DESC_F_WRITE;

        let idx = disk.avail_ring.idx as usize % VIRTQ_CAP;
        disk.avail_ring.ring[idx] = desc_id1;

        fence(core::sync::atomic::Ordering::SeqCst);

        disk.avail_ring.idx += 1;

        fence(core::sync::atomic::Ordering::SeqCst);

        w_virtio_register::<VIRTIO_MMIO_QUEUE_NOTIFY>(0); // 0 is the index of the only queue

        desc_id1
    };

    loop {
        match status {
            0xff => wfi(),
            s => {
                let mut disk = DISK.get().unwrap().lock();
                disk.free_desc_chain(head_desc_chain);

                if s != 0 {
                    return Err(s);
                } else {
                    return Ok(());
                }
            }
        }
    }
}

pub fn virtio_intr() {
    let mut disk = DISK.get().unwrap().lock();

    w_virtio_register::<VIRTIO_MMIO_INTERRUPT_ACK>(
        r_virtio_register::<VIRTIO_MMIO_INTERRUPT_STATUS>() & 0x3,
    );

    fence(core::sync::atomic::Ordering::SeqCst);

    let mut current_idx = disk.used_idxs;
    while current_idx < disk.used_ring.idx {
        fence(core::sync::atomic::Ordering::SeqCst);
        // let head_desc_id = disk.used_ring.ring[current_idx as usize % VIRTQ_CAP].desc_id as u16;
        // match disk.statuses[head_desc_id as usize] {
        //     0 => {}
        //     err => panic!("Disk operation resulted in error - Status: {}", err),
        // }
        // disk.free_desc_chain(head_desc_id);
        //
        current_idx += 1;
    }
    disk.used_idxs = disk.used_ring.idx;
}
