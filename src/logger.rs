use core::fmt::{self, Write};

use crate::{print, println, sprint, sprintln};

static mut LOG_LEVEL: u64 = 0;

pub static mut ENABLE_SCREEN: bool = false;

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::logger::_debug(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::logger::_info(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! note {
    ($($arg:tt)*) => ($crate::logger::_note(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ($crate::logger::_warn(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::logger::_error(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => ($crate::logger::_fatal(format_args!($($arg)*)));
}

pub fn _debug(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } == 0 {
        if unsafe { ENABLE_SCREEN } == true {
           //crate::screen::output::WRITER.lock().write_fmt(format_args!("[DEBUG] {}\n", args)).unwrap();
        }
        sprint!("[DEBUG] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _info(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 1 {
        if unsafe { ENABLE_SCREEN } == true {
            crate::screen::output::WRITER.lock().write_fmt(format_args!("[INFO] {}\n", args)).unwrap();
        }
        sprint!("[INFO] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _note(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 2 {
        if unsafe { ENABLE_SCREEN } == true {
            crate::screen::output::WRITER.lock().write_fmt(format_args!("[NOTE] {}\n", args)).unwrap();
        }
        sprint!("[NOTE] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _warn(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 3 {
        if unsafe { ENABLE_SCREEN } == true {
            crate::screen::output::WRITER.lock().write_fmt(format_args!("[WARN] {}\n", args)).unwrap();
        }
        sprint!("[WARN] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _error(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 4 {
        if unsafe { ENABLE_SCREEN } == true {
            crate::screen::output::WRITER.lock().write_fmt(format_args!("[ERROR] {}\n", args)).unwrap();
        }
        sprint!("[ERROR] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _fatal(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 5 {
        if unsafe { ENABLE_SCREEN } == true {
            crate::screen::output::WRITER.lock().write_fmt(format_args!("[FATAL] {}\n", args)).unwrap();
            //crate::screen::output::WRITER.lock().flush_buffer();
        }
        sprint!("[FATAL] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn set_log_level(level: u64) {
    unsafe {
        if level > 5 {
            LOG_LEVEL = 0;
        } LOG_LEVEL = level;
    }
}