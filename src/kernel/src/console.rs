use spin::Mutex;

use crate::uart::init_uart;

pub const CONSOLE_DEV_ID: usize = 1;

pub struct Console {
    /// Make sure this struct can't be constructed elsewhere
    _priv: (),
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console { _priv: () });

pub unsafe fn init_console() {
    unsafe { init_uart() };
    CONSOLE.lock().init()
}

impl Console {
    pub fn init(&mut self) {
        todo!()
    }
}
