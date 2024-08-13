use crate::{cprint, cprintln, mem::paging::Page, param::PAGE_SIZE, virtio::read_from_disk};
use alloc::boxed::Box;
use core::{
    ascii,
    mem::{transmute, MaybeUninit},
};
pub use fs::*;
use spin::Mutex;

#[repr(transparent)]
pub struct FileTable([FileMeta; MAX_FILES]);

pub static FILES: Mutex<FileTable> = Mutex::new(FileTable(
    [unsafe { core::mem::transmute(MaybeUninit::<FileMeta>::zeroed()) }; MAX_FILES],
));

pub fn init_files() {
    let mut files = FILES.lock();
    let mut buff = [0; 1024];
    read_from_disk(0, &mut buff).unwrap();
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

    pub fn get_file_meta(&self, file_name: &str) -> Option<&FileMeta> {
        let file_name_ascii = file_name.as_ascii()?;
        self.0
            .iter()
            .find(|fm| strcmp_ascii(file_name_ascii, fm.name))
    }

    /// Copy the entire file data to ram, returning a slice of contigous Physical
    /// Frames that contain the file data.
    pub fn copy_to_ram(&self, file_name: &str) -> Option<Box<[u8]>> {
        let file_meta = self.get_file_meta(file_name)?;
        let pages = file_meta.size as usize / PAGE_SIZE + 2;
        let file_frames: Box<[Page]> = unsafe { Box::new_zeroed_slice(pages).assume_init() };
        let file_frames: Box<[[u8; PAGE_SIZE]]> = unsafe { transmute(file_frames) };
        let mut file_data = file_frames.into_vec().into_flattened();
        let head_node_id = file_meta.node_list_start;
        let mut current_node_id = head_node_id;
        for seg in (0..file_meta.size as usize)
            .into_iter()
            .step_by(FILE_DATA_SIZE)
        {
            let mut node: Node = unsafe { core::mem::transmute([0u8; 1024]) };
            read_node(&mut node, current_node_id);
            #[cfg(debug_assertions)]
            cprintln!(
                "Read Node {}. Next Node: {}",
                current_node_id,
                node.next_node
            );
            file_data[seg..(seg + FILE_DATA_SIZE)].copy_from_slice(&node.data);
            current_node_id = node.next_node;
        }
        core::hint::black_box(&mut file_data);
        file_data.truncate(file_meta.size as usize);
        #[cfg(debug_assertions)]
        cprintln!(
            "Done copying file {} to ram. The file is in {:#x}",
            file_name,
            file_data.as_ptr() as usize
        );
        Some(file_data.into_boxed_slice())
    }

    pub fn debug_file(&self, file_name: &str) {
        cprintln!("{:#?}", self.get_file_meta(file_name));
    }

    pub fn cat(&self, file_name: &str) {
        let file_data = self.copy_to_ram(file_name).unwrap();
        for chr in &file_data {
            cprint!(
                "{}",
                ascii::Char::from_u8(*chr).unwrap_or(ascii::Char::QuestionMark)
            );
        }
    }
}

fn read_node(buf: &mut Node, node_id: u32) {
    let node_addr = node_address(node_id);
    let node_sector = node_addr / SECTOR_SIZE;
    read_from_disk(node_sector as u64, unsafe { transmute(&mut *buf) }).unwrap();
    assert_eq!(buf.magic_number, NODE_MAGIC_NUMBER);
}

fn strcmp_ascii<const N: usize>(s: &[ascii::Char], ass: [ascii::Char; N]) -> bool {
    if N < s.len() {
        return false;
    }
    for i in 0..N {
        if *s.get(i).unwrap_or(&ascii::Char::from_u8(0).unwrap()) != ass[i] {
            return false;
        }
    }
    true
}
