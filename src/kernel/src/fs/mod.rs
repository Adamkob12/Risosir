use core::{
    ascii,
    mem::{transmute, MaybeUninit},
};

pub use fs::*;
use spin::Mutex;

use crate::{cprint, cprintln, virtio::try_read_from_disk};

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
}
