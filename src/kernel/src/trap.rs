#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Trap {
    MachineTimerInterrupt = 7,
}

impl Trap {
    pub fn bitmask(&self) -> u64 {
        1 << (*self as u64)
    }
}
