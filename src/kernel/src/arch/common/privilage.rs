#[repr(u64)]
pub enum PrivLevel {
    /// Machine
    M = 3,
    /// Supervisor
    S = 1,
    /// User
    U = 0,
}

impl TryFrom<u64> for PrivLevel {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PrivLevel::U),
            1 => Ok(PrivLevel::S),
            3 => Ok(PrivLevel::M),
            _ => Err(()),
        }
    }
}
