use crate::arch::memlayout::UART_BASE_ADDR;
use spin::Mutex;

/// Uart 16550
/// Implementation based off [`this spec`](https://www.lammertbies.nl/comm/info/serial-uart#DLX)
pub struct Uart {
    base_addr: usize,
}

/// Uart IER register, only available when DLAB is off
const IER: u8 = 1;
const MCR: u8 = 4;
/// Uart LCR register
const LCR: u8 = 3;
/// Uart DLL register, only avaliable when dlab is set
const DLL: u8 = 0;
/// Uart DLM register, only avaliable when dlab is set
const DLM: u8 = 1;
/// Uart FCR register, write-only
const FCR: u8 = 2;
/// Uart THR register, only available when DLAB is off
pub const THR: u8 = 0;

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
        break_();
        unsafe { ((self.base_addr + REG as usize) as *mut u8).write_volatile(val) }
    }
}

fn break_() {}
