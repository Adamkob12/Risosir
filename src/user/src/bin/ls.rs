#![no_std]
#![no_main]

use fs::FILES;
use kernel::*;

#[no_mangle]
fn maine() {
    FILES.lock().ls();
}
