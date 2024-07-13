use crate::param::PAGE_SIZE;

pub(super) const PAGE_TABLE_ENTRIES: usize =
    const { PAGE_SIZE / core::mem::size_of::<PageTableEntry>() };

/// As per Figure 60. in the RISC-V manual II
#[repr(transparent)]
pub struct VirtAddr(u64);

/// As per Figure 61. in the RISC-V manual II
#[repr(transparent)]
pub struct PhysAddr(u64);

/// As per Figure 62. in the RISC-V manual II
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl VirtAddr {
    const OFFSET_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111_1111;
    const VPN0_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_1111_1111_0000_0000_0000;
    const VPN1_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0011_1111_1110_0000_0000_0000_0000_0000;
    const VPN2_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0111_1111_1100_0000_0000_0000_0000_0000_0000_0000;
    const LAST_BIT_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000;
    const TRAIL_MASK: u64 =
        0b1111_1111_1111_1111_1111_1111_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000;

    /// The offset (address inside the page)
    pub fn offset(&self) -> u64 {
        self.0 & Self::OFFSET_MASK
    }

    /// (Virtual) Page Number 0
    pub fn vpn0(&self) -> u64 {
        self.0 & Self::VPN0_MASK
    }

    /// (Virtual) Page Number 1
    pub fn vpn1(&self) -> u64 {
        self.0 & Self::VPN1_MASK
    }

    /// (Virtual) Page Number 2
    pub fn vpn2(&self) -> u64 {
        self.0 & Self::VPN2_MASK
    }

    /// Assert that the last 26 bits are the same, just as described in the spec
    pub fn assert_valid(&self) -> bool {
        let last_26_bits = self.0 & (Self::TRAIL_MASK | Self::LAST_BIT_MASK);
        // Make sure the last 26 bits are all the same
        last_26_bits == (Self::TRAIL_MASK | Self::LAST_BIT_MASK) || last_26_bits == 0
    }
}

impl PhysAddr {
    const OFFSET_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111_1111;
    const PPN0_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_1111_1111_0000_0000_0000;
    const PPN1_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0011_1111_1110_0000_0000_0000_0000_0000;
    const PPN2_MASK: u64 =
        0b0000_0000_1111_1111_1111_1111_1111_1111_1100_0000_0000_0000_0000_0000_0000_0000;

    /// The offset (address inside the page)
    pub fn offset(&self) -> u64 {
        self.0 & Self::OFFSET_MASK
    }

    /// (Physical) Page Number 0
    pub fn ppn0(&self) -> u64 {
        self.0 & Self::PPN0_MASK
    }

    /// (Physical) Page Number 1
    pub fn ppn1(&self) -> u64 {
        self.0 & Self::PPN1_MASK
    }

    /// (Physical) Page Number 2
    pub fn vpn2(&self) -> u64 {
        self.0 & Self::PPN2_MASK
    }
}

impl PageTableEntry {
    /// Is the entry valid?
    const V_FLAG_MASK: u64 = 1 << 0;
    /// Read permissions
    const R_FLAG_MASK: u64 = 1 << 1;
    /// Write permissions
    const W_FLAG_MASK: u64 = 1 << 2;
    /// Executable permissions
    const X_FLAG_MASK: u64 = 1 << 3;
    /// Accessible to User mode
    const U_FLAG_MASK: u64 = 1 << 4;
    /// "Global" mapping
    const G_FLAG_MASK: u64 = 1 << 5;
    /// "Accessed" - The page has been accessed (read, write, fetch) since the A flag was last cleared.
    const A_FLAG_MASK: u64 = 1 << 6;
    /// "Dirty" - The page has been written to since the D flag was last cleared
    const D_FLAG_MASK: u64 = 1 << 7;
    /// Free to use
    const RSW_MASK: u64 = 0b11 << 8;
    const PPN0_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0111_1111_1100_0000_0000;
    const PPN1_MASK: u64 =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111_1000_0000_0000_0000_0000;
    const PPN2_MASK: u64 =
        0b0000_0000_0011_1111_1111_1111_1111_1111_1111_0000_0000_0000_0000_0000_0000_0000;

    pub const fn new_invalid() -> Self {
        PageTableEntry(0)
    }
}
