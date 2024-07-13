#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Exception {
    MachineTimerInterrupt = 7,
}

impl Exception {
    pub fn bitmask(&self) -> u64 {
        1 << (*self as u64)
    }
}
