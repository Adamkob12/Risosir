#![feature(asm_const)]
#![allow(static_mut_refs)]
#![feature(naked_functions)]
#![feature(fn_align)]
#![feature(panic_info_message)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![no_std]
#![no_main]

use core::ascii;

pub const SECTOR_SIZE: usize = 512;
pub const NODE_MAGIC_NUMBER: u32 = 102030069;
pub const FILE_MAGIC_NUMBER: u32 = 900000111;
pub const MAX_FILES: usize = NODE_SIZE;
pub const NODES_OFFSET: usize = size_of::<FileMeta>() * MAX_FILES;
pub const FILE_NAME_LEN: usize = 18;
pub const NODE_SIZE: usize = 1024;
pub const FILE_DATA_SIZE: usize = NODE_SIZE - 16;
/// The address of a node with some NodeId is: (NODES_OFFSET + size_of::<Node>() * NodeId)
pub type NodeId = u32;
pub type FileId = u16;
// Must be 32 bytes
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FileMeta {
    pub magic_number: u32,                  // 4 bytes, Always =FILE_MAGIC_NUMBER
    pub node_list_start: NodeId,            // 4 bytes, the index of the node
    pub file_id: FileId,                    // 2 bytes
    pub name: [ascii::Char; FILE_NAME_LEN], // 18 bytes
    pub size: u32,                          // 4 bytes, size in bytes
}

pub type FileData = [u8; FILE_DATA_SIZE];

// Must be 1 KB exactly
#[repr(C)]
pub struct Node {
    pub magic_number: u32, // 4 bytes, Always =NODE_MAGIC_NUMBER
    pub file_id: FileId,   // 2 byte
    pub flags: u16,        // 2 byte
    pub next_node: NodeId, // 4 bytes
    pub prev_node: NodeId, // 4 bytes
    // metadata = 16 bytes
    pub data: FileData,
}

const _: () = {
    if core::mem::size_of::<Node>() != NODE_SIZE {
        panic!()
    }
    if core::mem::size_of::<FileMeta>() != 32 {
        panic!()
    }
};

pub const fn node_address(node_id: NodeId) -> usize {
    NODES_OFFSET + size_of::<Node>() * node_id as usize
}
