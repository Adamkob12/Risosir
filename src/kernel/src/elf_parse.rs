use crate::cprintln;
use elf::{endian::AnyEndian, segment::SegmentTable, ElfBytes};

const RISCV_E_MACHINE: u16 = 0xf3;

pub struct ParsedExecutable<'a> {
    pub file_data: &'a [u8],
    pub segs: SegmentTable<'a, AnyEndian>,
    pub entry_point: usize,
}

/// The file_data must be the complete, uncut elf file
pub fn parse_executable_file<'f>(file_data: &'f [u8]) -> Option<ParsedExecutable<'f>> {
    let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(file_data).ok()?;
    let segs = elf_bytes.segments()?;
    assert_eq!(elf_bytes.ehdr.e_machine, RISCV_E_MACHINE);
    #[cfg(debug_assertions)]
    cprintln!("{:#?}", elf_bytes.ehdr);
    #[cfg(debug_assertions)]
    cprintln!("File Segments:");
    for seg in segs.iter() {
        #[cfg(debug_assertions)]
        cprintln!("Segment: {:#?}", seg);
    }
    Some(ParsedExecutable {
        file_data,
        segs,
        entry_point: elf_bytes.ehdr.e_entry as usize,
    })
}
