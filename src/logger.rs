use core::fmt::{self, Write};

use crate::{sprint, sprintln};

static mut LOG_LEVEL: u64 = 0;

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
        sprint!("[DEBUG] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _info(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 1 {
        sprint!("[INFO] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _note(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 2 {
        sprint!("[NOTE] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _warn(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 3 {
        sprint!("[WARN] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _error(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 4 {
        sprint!("[ERROR] ");
        crate::serial::SENDER.lock().write_fmt(args).unwrap();
        sprintln!("");
    }
}

pub fn _fatal(args: fmt::Arguments) {
    if unsafe { LOG_LEVEL } <= 5 {
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