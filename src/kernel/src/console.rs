use crate::uart::{init_uart, UART};
use core::ascii;
use riscv::interrupt::free;
use spin::Mutex;

pub const CONSOLE_DEV_ID: usize = 1;
type ConsolePtr = u16;
const CONSOLE_BUFF_LEN: usize = ConsolePtr::MAX as usize + 1;

pub struct Console {
    buf: [ascii::Char; CONSOLE_BUFF_LEN],
    r_pointer: ConsolePtr,
    w_pointer: ConsolePtr,
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub unsafe fn init_console() {
    unsafe { init_uart() };
}

impl Console {
    const fn new() -> Self {
        Console {
            buf: [ascii::Char::CapitalM; CONSOLE_BUFF_LEN],
            r_pointer: 0,
            w_pointer: 0,
        }
    }

    /// If there is a pending char to read, read it.
    pub fn read_next(&mut self) -> Option<ascii::Char> {
        (self.r_pointer != self.w_pointer).then(|| {
            let ret = self.buf[self.r_pointer as usize];
            self.r_pointer = self.r_pointer.wrapping_add(1);
            ret
        })
    }

    pub fn write_char(&mut self, c: ascii::Char) -> Result<(), ()> {
        if self.w_pointer.wrapping_add(1) == self.r_pointer {
            // Buffer full
            Err(())
        } else {
            self.buf[self.w_pointer as usize] = c;
            self.w_pointer = self.w_pointer.wrapping_add(1);
            Ok(())
        }
    }

    /// return how many chars have been written
    pub fn write_str(&mut self, s: &str) -> usize {
        let mut chars_written: usize = 0;
        for c in s.chars() {
            let _ = self
                .write_char(c.as_ascii().unwrap_or(ascii::Char::SmallM))
                .map_err(|_| return chars_written);
            chars_written += 1;
        }
        chars_written
    }
}

impl core::fmt::Write for Console {
    fn write_str(&mut self, mut s: &str) -> core::fmt::Result {
        let mut read = 0;
        while read < s.len() {
            s = &s[read..];
            read += self.write_str(s);
            UART.lock().sync_send_pending(self);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! cprint {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! cprintln {
    () => ($crate::cprint!("\n"));
    ($($arg:tt)*) => ($crate::cprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    free(|| {
        CONSOLE
            .lock()
            .write_fmt(args)
            .expect("Couldn't write to console");
    });
}
