use core::ascii;

use crate::{memlayout::UART_BASE_ADDR, Console, CONSOLE};
use spin::Mutex;

/// Uart 16550
/// Implementation based off [`this spec`](https://www.lammertbies.nl/comm/info/serial-uart#DLX)
pub struct Uart {
    base_addr: usize,
}

/// Uart IER register, only available when DLAB is off
pub const IER: u8 = 1;
const _MCR: u8 = 4;
/// Uart LSR register for line status
const LSR: u8 = 5;
const LSR_THR_EMPTY_BIT: u8 = 1 << 5;
const LSR_RHR_READY_BIT: u8 = 1 << 0;
/// Uart LCR register
const LCR: u8 = 3;
/// Uart DLL register, only avaliable when dlab is set
const DLL: u8 = 0;
/// Uart DLM register, only avaliable when dlab is set
const DLM: u8 = 1;
/// Uart FCR register, write-only
const FCR: u8 = 2;
/// Uart ISR register, read-only
/// Uart THR register, only available when DLAB is off
pub const THR: u8 = 0;
/// Uart RHR register, only available when DLAB is off
pub const RHR: u8 = 0;

pub static UART: Mutex<Uart> = Mutex::new(Uart {
    base_addr: UART_BASE_ADDR,
});

pub unsafe fn init_uart() {
    unsafe { UART.lock().init() }
}

impl Uart {
    // Init the UART, get ready to recieve interrupts
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn init(&mut self) {
        // Disable interrupts
        self.write_to_register::<IER>(0x00);
        // Enable DLAB so we can access DLL and DLM
        self.write_to_register::<LCR>(1 << 7);
        // Set baud-rate to 38.4K
        self.write_to_register::<DLL>(0x03);
        self.write_to_register::<DLM>(0x00);
        // Disable DLAB so we can write to the uart, 8-bit data mode
        self.write_to_register::<LCR>(3 << 0);
        // Enable FIFO and clear the rx and tx FIFO buffers
        self.write_to_register::<FCR>((1 << 0) | (3 << 1));
        // Ready to recieve and send
        self.write_to_register::<IER>((1 << 0) | (1 << 1));
    }

    pub unsafe fn write_to_register<const REG: u8>(&mut self, val: u8) {
        ((self.base_addr + REG as usize) as *mut u8).write_volatile(val)
    }

    pub unsafe fn put_char(&mut self, char: u8) {
        self.write_to_register::<THR>(char)
    }

    pub unsafe fn write_chars(&mut self, chars: &[u8]) {
        for char in chars {
            self.put_char(*char)
        }
    }

    pub unsafe fn read_register<const REG: u8>(&mut self) -> u8 {
        ((self.base_addr + REG as usize) as *mut u8).read_volatile()
    }

    pub unsafe fn get_next(&mut self) -> Option<u8> {
        (self.read_register::<LSR>() & LSR_RHR_READY_BIT == 1)
            .then_some(self.read_register::<RHR>())
    }

    /// Read any pending data from the console, if the buffer ever becomes full, this function will wait until its free.
    /// As opposed to [`Self::async_send_pending`] which will send all the pending data from the console untill the uart buffer becomes full -
    /// then it returns and *only* continues after the uart interrupts and requests more data.
    pub fn sync_send_pending(&mut self, console: &mut Console) {
        unsafe {
            while let Some(char) = console.read_next() {
                while (self.read_register::<LSR>() & LSR_THR_EMPTY_BIT) == 0 {}
                self.put_char(char as u8);
            }
        }
    }

    /// Will send all the pending data from the console untill the uart buffer becomes full -
    /// then it returns and *only* continues after the uart interrupts and requests more data.
    pub fn async_send_pending(&mut self, console: &mut Console) {
        unsafe {
            while (self.read_register::<LSR>() & LSR_THR_EMPTY_BIT) != 0 {
                if let Some(char) = console.read_next() {
                    self.write_to_register::<THR>(char as u8);
                } else {
                    break;
                }
            }
        }
    }
}

pub fn uart_interrupt() {
    let mut console = CONSOLE.lock();
    let mut uart = UART.lock();
    // let mut kb = KEYBOARD.lock();

    while let Some(key) = unsafe { uart.get_next() } {
        console
            .write_char(ascii::Char::from_u8(key).unwrap())
            .unwrap();
    }
    uart.async_send_pending(&mut *console);
}
