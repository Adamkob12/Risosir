#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
#[non_exhaustive]
/// Table 15
pub enum MachineException {
    Breakpoint = 3,
    UModeEcall = 8,
    PageFault = 12,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
#[non_exhaustive]
/// Table 22
pub enum SupervisorException {
    Breakpoint = 3,
    UModeEcall = 8,
    PageFault = 12,
}
