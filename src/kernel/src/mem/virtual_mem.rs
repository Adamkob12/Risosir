use core::ptr::NonNull;

use crate::param::PAGE_SIZE;

use super::paging::{Frame, PageTableLevel};

pub(super) const PAGE_TABLE_ENTRIES: usize =
    const { PAGE_SIZE / core::mem::size_of::<PageTableEntry>() };

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PTEFlags(u64);

/// As per Figure 60. in the RISC-V manual II
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct VirtAddr(u64);

/// As per Figure 61. in the RISC-V manual II
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct PhysAddr(u64);

/// As per Figure 62. in the RISC-V manual II
#[derive(Clone, Copy, Debug)]
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

    pub fn from_raw(addr: u64) -> Self {
        Self(addr)
    }

    /// The offset (address inside the page)
    pub fn offset(&self) -> u64 {
        self.0 & Self::OFFSET_MASK
    }

    /// (Virtual) Page Number 0
    /// return range: [0, 2^9 - 1]
    pub fn vpn0(&self) -> u64 {
        (self.0 & Self::VPN0_MASK) >> 12
    }

    /// (Virtual) Page Number 1 (index in L2 table)
    /// return range: [0, 2^9 - 1]
    pub fn vpn1(&self) -> u64 {
        (self.0 & Self::VPN1_MASK) >> (12 + 9)
    }

    /// (Virtual) Page Number 2 (index in L3 table)
    /// return range: [0, 2^9 - 1]
    pub fn vpn2(&self) -> u64 {
        (self.0 & Self::VPN2_MASK) >> (12 + 9 + 9)
    }

    /// Index in L{x} page table
    /// return range: [0, 2^9 - 1]
    pub fn vpn(&self, level: PageTableLevel) -> u64 {
        match level {
            PageTableLevel::L2 => self.vpn2(),
            PageTableLevel::L1 => self.vpn1(),
            PageTableLevel::L0 => self.vpn0(),
        }
    }

    /// Assert that the last 26 bits are the same, just as described in the spec
    pub fn assert_valid(&self) -> bool {
        let last_26_bits = self.0 & (Self::TRAIL_MASK | Self::LAST_BIT_MASK);
        // Make sure the last 26 bits are all the same
        last_26_bits == (Self::TRAIL_MASK | Self::LAST_BIT_MASK) || last_26_bits == 0
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn round_down(&mut self) {
        self.0 &= !Self::OFFSET_MASK
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

    pub fn from_raw(addr: u64) -> Self {
        Self(addr)
    }

    /// The offset (address inside the page)
    pub fn offset(&self) -> u64 {
        self.0 & Self::OFFSET_MASK
    }

    /// The address of the frame
    pub fn frame_adrr(&self) -> u64 {
        self.0 & (Self::PPN0_MASK | Self::PPN1_MASK | Self::PPN2_MASK)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn round_down(&mut self) {
        self.0 &= !Self::OFFSET_MASK
    }
}

impl PTEFlags {
    pub fn valid() -> Self {
        PTEFlags(PageTableEntry::V_FLAG_MASK)
    }

    pub fn redirect() -> Self {
        PTEFlags(PageTableEntry::V_FLAG_MASK)
    }

    pub fn readable(self) -> Self {
        PTEFlags(self.0 | PageTableEntry::R_FLAG_MASK)
    }

    pub fn writable(self) -> Self {
        PTEFlags(self.0 | PageTableEntry::W_FLAG_MASK)
    }

    pub fn executable(self) -> Self {
        PTEFlags(self.0 | PageTableEntry::X_FLAG_MASK)
    }

    pub fn is_valid(&self) -> bool {
        (self.0 & PageTableEntry::V_FLAG_MASK) > 0
    }

    pub fn is_redirect(&self) -> bool {
        !self.is_readable() && !self.is_writable() && !self.is_executable()
    }

    pub fn is_readable(&self) -> bool {
        (self.0 & PageTableEntry::R_FLAG_MASK) > 0
    }

    pub fn is_writable(&self) -> bool {
        (self.0 & PageTableEntry::W_FLAG_MASK) > 0
    }

    pub fn is_executable(&self) -> bool {
        (self.0 & PageTableEntry::X_FLAG_MASK) > 0
    }
}

#[allow(unused)]
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

    pub fn new(frame: NonNull<Frame>, flags: PTEFlags) -> Self {
        Self(((frame.as_ptr() as u64 >> 12) << 10) | flags.0)
    }

    pub const fn is_valid(&self) -> bool {
        (self.0 & Self::V_FLAG_MASK) > 0
    }

    pub const fn is_readable(&self) -> bool {
        (self.0 & Self::R_FLAG_MASK) > 0
    }

    pub const fn is_writable(&self) -> bool {
        (self.0 & Self::W_FLAG_MASK) > 0
    }

    pub const fn is_executable(&self) -> bool {
        (self.0 & Self::X_FLAG_MASK) > 0
    }

    /// Is this page entry a redirect to another page table
    pub const fn is_redirect(&self) -> bool {
        !self.is_readable() && !self.is_executable() && !self.is_writable()
    }

    /// The frame addr, aligned to 4096 bytes
    pub fn frame_addr(&self) -> u64 {
        // self.0 & (Self::PPN0_MASK | Self::PPN1_MASK | Self::PPN2_MASK)
        ((self.0 >> 10) << 12)
    }

    pub fn set(&mut self, frame_addr: u64, flags: PTEFlags) {
        self.0 = ((frame_addr >> 12) << 10) | flags.0;
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
