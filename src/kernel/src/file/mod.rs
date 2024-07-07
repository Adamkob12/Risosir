use core::{mem::MaybeUninit, sync::atomic::AtomicBool};

use spin::Mutex;

use crate::param::NDEV;

/// Similar to the Unix [`read syscall`](https://man7.org/linux/man-pages/man2/read.2.html)
pub type ReadFunc = fn(u64, *mut u8, u64);
/// Similar to the Unix [`write syscall`](https://man7.org/linux/man-pages/man2/write.2.html)
pub type WriteFunc = fn(u64, *mut u8, u64);

/// Store the function pointers that read / write from / to devices.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DeviceFuncs {
    read: ReadFunc,
    write: WriteFunc,
}

/// Holds the [`DeviceFuncs`] of each device.
pub static DEVICE_FUNCS: Mutex<[MaybeUninit<DeviceFuncs>; NDEV]> =
    Mutex::new([MaybeUninit::uninit(); NDEV]);

/// Keeps track of which devies have been init
pub static DEVICE_INIT: [AtomicBool; NDEV] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

pub unsafe fn init_device(dev_id: usize, read: ReadFunc, write: WriteFunc) {
    use core::sync::atomic::Ordering;
    if !DEVICE_INIT[dev_id].load(Ordering::SeqCst) {
        let mut devs = DEVICE_FUNCS.lock();
        devs[dev_id].write(DeviceFuncs { read, write });
        DEVICE_INIT[dev_id].store(true, Ordering::SeqCst);
    } else {
        panic!("Can't init device {} twice.", dev_id)
    }
}
