#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{panic::PanicInfo, ptr::write_volatile};

use yenx_kernel::{
    apic::{apic_init, apic_timer_diag, apic_timer_init, enable_x2apic}, init, mm::{self, frame_alloc::alloc_frame}, print, println
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(_magic: u32, _mb_info_addr: u32) -> ! {
    println!("YENX Kernel");
    init();
    enable_x2apic();
    apic_init();
    x86_64::instructions::interrupts::disable();
    apic_timer_init();
    apic_timer_diag();

    unsafe {
        mm::paging::init();
    }

    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
    println!("x2APIC enabled!");

    unsafe {
       let a1 = alloc_frame();
       let a2 = alloc_frame();
       let a3 = alloc_frame();
       println!("a1: 0x{:x}, a2: 0x{:x}, a3: 0x{:x}", a1, a2, a3);
    }

    x86_64::instructions::interrupts::enable();

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}