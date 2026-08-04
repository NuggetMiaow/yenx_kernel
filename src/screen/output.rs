extern crate alloc;
use core::fmt;
use alloc::vec::Vec;
use crate::{debug, screen::{self, fb::{clean_screen, put_pixel_rgb}}};
use lazy_static::lazy_static;
use spin::Mutex;

pub fn print_char_to_screen(c: u8, x: u64, y: u64, r: u8, g: u8, b: u8) {
    if c >= 128 {
        debug!("Unregioned char 0x{:x}", c);
        return;
    }
    
    for i in 0usize..16 {
        let mut bits: [u8; 8] = [0; 8];

        bits[0] = screen::font8x16::FONT8X16[c as usize][i] >> 7;
        bits[1] = screen::font8x16::FONT8X16[c as usize][i] >> 6 & 1;
        bits[2] = screen::font8x16::FONT8X16[c as usize][i] >> 5 & 1;
        bits[3] = screen::font8x16::FONT8X16[c as usize][i] >> 4 & 1;
        bits[4] = screen::font8x16::FONT8X16[c as usize][i] >> 3 & 1;
        bits[5] = screen::font8x16::FONT8X16[c as usize][i] >> 2 & 1;
        bits[6] = screen::font8x16::FONT8X16[c as usize][i] >> 1 & 1;
        bits[7] = screen::font8x16::FONT8X16[c as usize][i];

        for j in 0u64..8 {
            if bits[j as usize] == 1 {
                put_pixel_rgb((j + x) as usize, ((i as u64) + y) as usize, r, g, b);
            }
        }
    }
}

pub struct Writer {
    row: usize,
    line: usize,
    size: usize,
    buffer: Vec<u8>,
    index: usize,
}

impl Writer {
    pub fn new(x: usize, y: usize) -> Self {
        let row = x / 9;
        let line = y / 17;
        let size = row * line;
        Self {
            row,
            line,
            size,
            buffer: alloc::vec![0; size],
            index: 0,
        }
    }

    pub fn print_char(&mut self, c: u8) {
        if self.index == self.size {
            self.row_buffer();
            self.index -= self.row;
        }

        self.buffer[self.index] = c;
        self.index += 1;

        if c == b'\n' {
            self.flush_buffer();
        }

        if self.index == self.size {
            self.row_buffer();
            self.flush_buffer();
            self.index -= self.row;
        }
    }

    pub fn row_buffer(&mut self) {
        for i in self.row..self.size {
            self.buffer[i - self.row] = self.buffer[i];
        }

        for i in (self.size - self.row)..self.size {
            self.buffer[i] = 0;
        }
    }

    pub fn flush_buffer(&mut self) {
        clean_screen();
        let mut cx = 0;
        let mut cy = 0;

        for &c in self.buffer.iter() {
            if c == b'\n' {
                cx = 0;
                cy += 1;
                if cy >= self.line as u64 {
                    break;
                }
                continue;
            }
            if c != 0 {
                print_char_to_screen(c, cx * 9, 1 + cy * 16, 255, 255, 255);
            }
            cx += 1;
            if cx == self.row as u64 {
                cx = 0;
                cy += 1;
                if cy >= self.line as u64 {
                    break;
                }
            }
        }
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(1024, 768));
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.print_char(c);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::screen::output::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
    WRITER.lock().flush_buffer();
}