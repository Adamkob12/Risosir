use crate::{
    cprintln,
    uart::{init_uart, UART},
};
use core::ascii;
use spin::Mutex;

type KeyBoardPtr = u16;
pub type Key = u8;
const KEYBOARD_BUFF_LEN: usize = KeyBoardPtr::MAX as usize + 1;

pub struct Keyboard {
    buf: [Key; KEYBOARD_BUFF_LEN],
    pending: Option<Key>,
    r_pointer: KeyBoardPtr,
    w_pointer: KeyBoardPtr,
}

pub static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());

impl Keyboard {
    const fn new() -> Self {
        Keyboard {
            buf: [0; KEYBOARD_BUFF_LEN],
            pending: None,
            r_pointer: 0,
            w_pointer: 0,
        }
    }

    /// If there is a pending char to read, read it.
    pub fn read_next_press(&mut self) -> Option<Key> {
        (self.r_pointer != self.w_pointer).then(|| {
            let ret = self.buf[self.r_pointer as usize];
            self.r_pointer = self.r_pointer.wrapping_add(1);
            ret
        })
    }

    pub fn update_new_press(&mut self, key: Key) -> Result<(), ()> {
        if let Some(pending_key) = self.pending.take() {
            return self.update_new_press(pending_key);
        }
        if self.w_pointer.wrapping_add(1) == self.r_pointer {
            // Buffer full
            self.pending = Some(key);
            Err(())
        } else {
            self.buf[self.w_pointer as usize] = key;
            self.w_pointer = self.w_pointer.wrapping_add(1);
            Ok(())
        }
    }
}

pub fn read_recent_input() {
    let mut keyboard = KEYBOARD.lock();
    while let Some(key) = keyboard.read_next_press() {
        cprintln!("Key pressed: {}", key);
    }
}
