#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::{panic::PanicInfo, ptr::write_volatile};

use alloc::boxed::Box;
use yenx_kernel::{
    apic::{apic_init, apic_timer_diag, apic_timer_init, enable_x2apic}, info, init, mm::{self, frame_alloc::alloc_frame, malloc::{kfree, kmalloc}, paging}, screen, serial, sprintln
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(_magic: u32, mb_info_addr: *mut u8) -> ! {
    unsafe {
        mm::paging::init();
        serial::serial_init();
    }
    screen::fb::init(mb_info_addr);
    info!("YENX Kernel 0.01-dev");
    
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

/*#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::{panic::PanicInfo, ptr::write_volatile};

use alloc::boxed::Box;
use yenx_kernel::{
    apic::{apic_init, apic_timer_diag, apic_timer_init, enable_x2apic}, init, mm::{self, frame_alloc::alloc_frame, malloc::{kfree, kmalloc}}, screen
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(_magic: u32, mb_info_addr: *mut u8) -> ! {
    unsafe {
        let fb : *mut screen::fb::FramebufferTag = mb_info_addr.add(8) as *mut _     ;

        let fb_addr = (*fb).addr as *mut u32;    // 假设 32bpp，每个像素 4 字节
        let width = (*fb).width as usize;
        let height = (*fb).height as usize;
        let pitch = (*fb).pitch as usize / 4;    // 每行的 u32 个数

        // 画一个简单的红色矩形
        for y in 0..height {
            let row = unsafe { fb_addr.add(y * pitch) };
            for x in 0..width {
                unsafe { row.add(x).write(0x00FF0000) }; // 红色
            }
        }
    }
    println!("YENX Kernel");
    init();
    enable_x2apic();
    apic_init();
    x86_64::instructions::interrupts::disable();
    apic_timer_init();
    apic_timer_diag();

    unsafe {
        mm::paging::init();
        mm::frame_alloc::init_allocator();
    }

    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
    println!("x2APIC enabled!");

    let a1 = Box::<&str>::new("Hello, World!");
    print!("a1: Box<&str> = {}", a1);

    x86_64::instructions::interrupts::enable();

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}*/