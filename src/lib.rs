#![feature(abi_x86_interrupt)]
#![no_std]

pub mod interrupts;

pub mod gdt;
pub mod apic;
pub mod mm;
pub mod screen;
pub mod serial;
pub mod logger;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}