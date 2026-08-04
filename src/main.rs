#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::{arch::asm, panic::PanicInfo, ptr::write_volatile};

use alloc::boxed::Box;
use yenx_kernel::{
    apic::{apic_init, apic_timer_diag, apic_timer_init, enable_x2apic}, info, init, mm::{self, frame_alloc::alloc_frame, malloc::{kfree, kmalloc}}, print, println, process, screen, sprintln
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" fn demo_a() -> ! {
    loop {
        sprintln!("A");
    }
}

extern "C" fn demo_b() -> ! {
    loop {
        sprintln!("A");
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(_magic: u32, mb_info_addr: *mut u8) -> ! {
    unsafe {
        mm::paging::init();
        screen::fb::init(mb_info_addr);
    }
    info!("YENX Kernel 0.01-dev1");
    unsafe {
        mm::frame_alloc::init_allocator();
    }
    init();
    enable_x2apic();
    apic_init();
    info!("x2APIC enabled!");
    x86_64::instructions::interrupts::disable();
    apic_timer_init();
    apic_timer_diag();

    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    info!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    process::spawn(demo_a);

    x86_64::instructions::interrupts::enable();

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}