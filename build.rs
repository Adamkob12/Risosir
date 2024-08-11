#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(path_file_prefix)]

use ::fs::*;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::{
    ascii,
    fs::{read_dir, File, OpenOptions},
    io::{ErrorKind, Seek, SeekFrom, Write},
    slice,
};
use walkdir::WalkDir;

const SHARED_FILES: &str = "shared_files";
const ELF_SOURCE: &str = "target/riscv64gc-unknown-none-elf/debug";

fn is_elf_file(file_path: &Path) -> io::Result<bool> {
    let mut file = fs::File::open(file_path)?;
    let mut buffer = [0u8; 4];
    file.read_exact(&mut buffer)?;
    Ok(&buffer == b"\x7fELF")
}

fn copy_elf_files(src_dir: &Path, dst_dir: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if path.is_file() && is_elf_file(path).unwrap_or(false) {
            let relative_path = path.strip_prefix(src_dir).unwrap();
            let dest_path = dst_dir.join(relative_path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(path, &dest_path)?;
            println!("Copied: {:?} to {:?}", path, dest_path);
        }
    }
    Ok(())
}

fn main() {
    println!("cargo:rustc-link-arg-bin=risosir=--script=src/kernel/kernel.ld");

    copy_elf_files(&Path::new(ELF_SOURCE), &Path::new(SHARED_FILES)).unwrap();

    let mut img = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open("fs.img")
        .unwrap();
    let shared_files = Path::new(SHARED_FILES);
    let dir = read_dir(shared_files).unwrap();
    let mut current_file_id: FileId = 1;
    let mut current_node_id: NodeId = 1;
    for entry in dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|e| e.is_file())
    {
        let mut file_data_buff: FileDataSeg = [0; 1024 - 16];
        let mut shared_file = File::open(&entry).unwrap();
        let file_name = String::from(entry.file_name().unwrap().to_str().unwrap());

        let file_meta = get_file_meta(&shared_file, file_name, current_file_id, current_node_id);
        // The file meta is ready, now copy its data to the disk img
        img.seek(SeekFrom::Start(
            (current_file_id as usize * size_of::<FileMeta>()) as u64,
        ))
        .unwrap();

        let file_meta_buff = as_byte_slice(&file_meta);
        assert_eq!(file_meta_buff.len(), 32);
        img.write_all(file_meta_buff).unwrap();
        // The file meta has been written to the file meta section
        // Now we need to write the actuall data of the file into the disk img

        let first_node_id = current_node_id;
        let last_node_id = first_node_id + file_meta.size / FILE_DATA_SIZE as u32;
        loop {
            let end = read_max(&mut shared_file, &mut file_data_buff).unwrap();
            let node = Node {
                magic_number: NODE_MAGIC_NUMBER,
                data: file_data_buff,
                flags: 1,
                next_node: if current_node_id == last_node_id {
                    first_node_id
                } else {
                    current_node_id + 1
                },
                prev_node: if current_node_id == first_node_id {
                    last_node_id
                } else {
                    current_node_id - 1
                },
                file_id: current_file_id,
            };

            img.seek(SeekFrom::Start(
                (NODES_OFFSET + current_node_id as usize * size_of::<Node>()) as u64,
            ))
            .unwrap();

            let node_buff = as_byte_slice(&node);
            assert_eq!(node_buff.len(), NODE_SIZE);
            img.write_all(node_buff).unwrap();

            current_node_id += 1;

            if end {
                break;
            }
        }

        current_file_id += 1;
    }
}

/// Return Ok(true) if the source has no more information to give
fn read_max<R: Read + ?Sized>(this: &mut R, mut buf: &mut [u8]) -> std::io::Result<bool> {
    while !buf.is_empty() {
        match this.read(buf) {
            Ok(0) => return Ok(true),
            Ok(n) => {
                buf = &mut buf[n..];
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}

fn as_byte_slice<'a, T>(t: &'a T) -> &'a [u8] {
    unsafe { slice::from_raw_parts(t as *const T as *const u8, size_of::<T>()) }
}

fn get_file_meta(
    file: &File,
    file_name: String,
    file_id: FileId,
    current_node_id: NodeId,
) -> FileMeta {
    let mut file_name_buff: [ascii::Char; FILE_NAME_LEN] = [ascii::Char::Null; FILE_NAME_LEN];
    if file_name.len() > FILE_NAME_LEN {
        panic!(
            "File name {} too long, must be shorter than {}",
            file_name, FILE_NAME_LEN
        );
    }
    for (i, char) in file_name
        .as_ascii()
        .expect("file name needs to be ascii compatible")
        .into_iter()
        .enumerate()
    {
        file_name_buff[i] = *char;
    }
    let file_size = file.metadata().unwrap().len() as u32;

    FileMeta {
        magic_number: FILE_MAGIC_NUMBER,
        file_id,
        size: file_size,
        name: file_name_buff,
        node_list_start: current_node_id,
    }
}
