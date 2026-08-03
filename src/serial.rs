use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub unsafe fn serial_init() {
    // 1. Start DLAB
    Port::write(&mut Port::new(0x3fb), 
        0x80u16);

    // 2. Set bps = 115200
    Port::write(&mut Port::new(0x3f8), 1u8);

    // 3. DLH
    Port::write(&mut Port::new(0x3f9), 0u8);

    // 4. 8N1
    Port::write(&mut Port::new(0x3fb), 0x03u8);

    // 5. DTR/RTS
    Port::write(&mut Port::new(0x3fc), 0x0bu8);
}

pub fn send_char(c: u8) {
    unsafe {
        loop {
            // write until thre = 1 (bit 5)
            let lsr: u8 = Port::read(&mut Port::<u8>::new(0x3fd));

            if (lsr & 0x20)!= 0 {
                break;
            }
        }

        Port::write(&mut Port::new(0x3f8), c);
    }
}

pub struct SerialSender {}

impl fmt::Write for SerialSender {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            send_char(c);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref SENDER: Mutex<SerialSender> = Mutex::new(SerialSender {});
}

#[macro_export]
macro_rules! sprint {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! sprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::sprint!("{}\r\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SENDER.lock().write_fmt(args).unwrap();
}
