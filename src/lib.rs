#![feature(abi_x86_interrupt)]
#![no_std]

pub mod interrupts;
pub mod vga_buffer;
pub mod gdt;
pub mod apic;
pub mod mm;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}