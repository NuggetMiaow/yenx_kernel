use core::ptr::write_volatile;

use x86_64::registers::debug;

use crate::{debug, fatal, info, logger, mm::{frame_alloc::zero_memory, malloc::translate_addr, paging::{self, make_pde, make_pdpte}}, screen};


#[derive(Debug)]
#[repr(C, packed)] 
pub struct FramebufferTag {
    pub typ: u32,
    pub size: u32,
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub fb_type: u8,
    pub reserved: u16,
}

#[repr(C, packed)] 
pub struct MultibootTagHeader {
    pub typ: u32,
    pub size: u32,
}

pub static mut FB_ADDR: u64 = 0;
pub static mut FB_WIDTH: usize = 0;
pub static mut FB_HEIGHT: usize = 0;
pub static mut FB_PITCH: usize = 0;

const PG_PRESENT: u64 = 1;
const PG_WRITEABLE: u64 = 1 << 1;
const PG_USER: u64 = 1 << 2;
const PG_CACHED: u64 = 1 << 3;
const PG_EXECUTABLE: u64 = 0 << 63;
const PG_GLOBAL: u64 = 1 << 8;

pub fn clean_screen() {
    unsafe { for i in 0..FB_WIDTH * FB_HEIGHT {
        (FB_ADDR as *mut u32).add(i).write(0x313131);
    } }
}

pub fn init(mb_info_addr: *mut u8) {
    unsafe {
        let mut tag_ptr = mb_info_addr.add(8);
        let mut fb_ptr: *mut screen::fb::FramebufferTag = core::ptr::null_mut();
        
        loop {
            let tag = &mut *(tag_ptr as *mut screen::fb::MultibootTagHeader);
            if tag.typ == 8 {
                fb_ptr = tag_ptr as *mut screen::fb::FramebufferTag;
                break;
            }
            let size = (tag.size as usize + 7) & !7;
            tag_ptr = tag_ptr.add(size);
            if tag.typ == 0 {
                break;
            }
        }
        
        if fb_ptr.is_null() {
            loop { core::arch::asm!("hlt"); }
        }
        
        let fb = &mut *fb_ptr;
        
        FB_WIDTH = fb.width as usize;
        FB_HEIGHT = fb.height as usize;
        FB_PITCH = fb.pitch as usize;
        
        if fb.bpp != 32 {
            fatal!("Framebuffer bpp must be 32");
            loop { core::arch::asm!("hlt"); }
        }

        info!("The physical address of framebuffer is 0x{:x}", fb.addr as u64);
        debug!("Mapping framebuffer......");

        let (pml4_addr, _) = x86_64::registers::control::Cr3::read();

        let pdpte_addr = pml4_addr.start_address().as_u64() + 4096 + 504;

        *((0x600000 + 496) as *mut u64) = make_pde(true, fb.addr as u64, PG_PRESENT | PG_WRITEABLE);
        *((0x600000 + 504) as *mut u64) = make_pde(true, (fb.addr + 2 * 1024 * 1024) as u64, 
            PG_PRESENT | PG_WRITEABLE);

        *(pdpte_addr as *mut u64) = make_pdpte(0x600000, PG_PRESENT | PG_WRITEABLE);

        FB_ADDR = translate_addr(fb.addr as u64);
        debug!("Mapping framebuffer done.");
        info!("The virtual address of framebuffer is 0x{:x}", FB_ADDR);

        // trying to access framebuffer
        debug!("Clean up screen");
        clean_screen();
        
        logger::ENABLE_SCREEN = true;
        debug!("Enabled Screen Logger");
        // Show the information
        info!("Framebuffer:\r\n{:#?}", fb);
    }
}

pub fn put_pixel(x: usize, y: usize, color: u32) {
    let width = unsafe { FB_WIDTH };
    let height = unsafe { FB_HEIGHT };
    let pitch = unsafe { FB_PITCH };
    
    if x >= width || y >= height {
        return;
    }

    let fb_addr = unsafe { FB_ADDR } as *mut u32;
    let pitch_u32 = pitch / 4;
    unsafe {
        let row = fb_addr.add(y * pitch_u32);
        row.add(x).write_volatile(color);
    }
}

pub fn put_pixel_rgb(x: usize, y: usize, r: u8, g: u8, b: u8) {
    put_pixel(x, y, ((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
}