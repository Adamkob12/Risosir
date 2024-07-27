use crate::cprintln;
use elf::{endian::AnyEndian, ElfBytes, ParseError};
use fs::FILE_DATA_SIZE;

const RISCV_E_MACHINE: u16 = 0xf3;

pub struct ParsedExecutable<'f> {
    pub text: &'f [u8],
    /// Virtual address of the text section (in the context of the process running th exe)
    pub text_v: usize,
    pub rodata: &'f [u8],
    /// Virtual address of the rodata section (in the context of the process running th exe)
    pub rodata_v: usize,
    pub data: &'f [u8],
    /// Virtual address of the data section (in the context of the process running th exe)
    pub data_v: usize,
    /// The entry point of the executable
    pub entry_point: usize,
}

/// The file_data must be the complete, uncut elf file
pub fn parse_executable_file(file_data: &[u8]) -> Result<ParsedExecutable, ParseError> {
    let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(file_data)?;
    let segs = elf_bytes.segments().ok_or(ParseError::BadMagic([69; 4]))?;
    assert_eq!(elf_bytes.ehdr.e_machine, RISCV_E_MACHINE);
    #[cfg(debug_assertions)]
    cprintln!("{:#?}", elf_bytes.ehdr);
    #[cfg(debug_assertions)]
    cprintln!("File Segments:");
    for seg in segs.iter() {
        cprintln!("Segment: {:#?}", seg);
    }
    let text_offset = segs.get(0).unwrap().p_offset as usize;
    let text_size = segs.get(0).unwrap().p_memsz as usize;
    let text_v = segs.get(0).unwrap().p_vaddr as usize;

    let rodata_offset = segs.get(1).unwrap().p_offset as usize;
    let rodata_size = segs.get(1).unwrap().p_memsz as usize;
    let rodata_v = segs.get(1).unwrap().p_vaddr as usize;

    let data_offset = segs.get(2).unwrap().p_offset as usize;
    let data_size = segs.get(2).unwrap().p_memsz as usize;
    let data_v = segs.get(2).unwrap().p_vaddr as usize;

    Ok(ParsedExecutable {
        text: &file_data[text_offset..(text_offset + text_size)],
        text_v,
        rodata: &file_data[rodata_offset..(rodata_offset + rodata_size)],
        rodata_v,
        data: &file_data[data_offset..(data_offset + data_size)],
        data_v,
        entry_point: elf_bytes.ehdr.e_entry as usize,
    })
}
