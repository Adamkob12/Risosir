use core::{
    ascii,
    mem::{transmute, MaybeUninit},
};

use alloc::boxed::Box;
pub use fs::*;
use spin::Mutex;

use crate::{cprint, cprintln, mem::paging::Frame, param::PAGE_SIZE, virtio::try_read_from_disk};

#[repr(transparent)]
pub struct FileTable([FileMeta; MAX_FILES]);

pub static FILES: Mutex<FileTable> = Mutex::new(FileTable(
    [unsafe { core::mem::transmute(MaybeUninit::<FileMeta>::zeroed()) }; MAX_FILES],
));

pub fn init_files() {
    let mut files = FILES.lock();
    let mut buff = [0; 1024];
    try_read_from_disk(0, &mut buff).unwrap();
    for _ in 0..10_000 {
        // Wait untill the data is read from the disk
    }
    let file_buff: [FileMeta; const { 1024 / size_of::<FileMeta>() }] = unsafe { transmute(buff) };
    (&mut files.0[0..(1024 / size_of::<FileMeta>())]).copy_from_slice(&file_buff);
}

impl FileTable {
    pub fn ls(&self) {
        cprintln!("FILE ID\t\tNAME\t\t\tSIZE");
        for file_meta in self
            .0
            .iter()
            .filter(|fm| fm.magic_number == FILE_MAGIC_NUMBER)
        {
            cprint!("{}\t\t", file_meta.file_id);
            for chr in file_meta.name.iter().map(|c| {
                if c.to_u8() == 0 {
                    ascii::Char::Space
                } else {
                    *c
                }
            }) {
                cprint!("{}", chr);
            }
            cprintln!("\t{}", file_meta.size);
        }
    }

    /// Copy the entire file data to ram, returning a slice of contigous Physical
    /// Frames that contain the file data.
    pub fn copy_to_ram(&self, file_name: &str) -> Option<Box<[u8]>> {
        let file_name_ascii = file_name.as_ascii()?;
        let file_meta = self
            .0
            .iter()
            .find(|fm| strcmp_ascii(file_name_ascii, fm.name))?;
        let mut file_data_heap = Box::<[u8]>::new_zeroed_slice(file_meta.size as usize);
        let mut current_node_id = file_meta.node_list_start;

        todo!()
    }
}

fn read_node(buf: &mut Node, node_id: u32) {
    todo!()
}

fn strcmp_ascii<const N: usize>(s: &[ascii::Char], ass: [ascii::Char; N]) -> bool {
    for i in 0..N {
        if *s.get(i).unwrap_or(&ascii::Char::from_u8(0).unwrap()) != ass[i] {
            return false;
        }
    }
    true
}
