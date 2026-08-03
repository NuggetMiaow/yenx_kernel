use core::char::MAX;
use crate::mm::{malloc::PT_CURRENT, paging::{make_pde, make_pte}};

use x86_64::registers::control::Cr3;

use crate::{mm::paging::make_pdpte, print, println};

const MAX_PAGE: usize = 131702;

const PG_PRESENT: u64 = 1;
const PG_WRITEABLE: u64 = 1 << 1;
const PG_USER: u64 = 1 << 2;
const PG_CACHED: u64 = 1 << 3;
const PG_EXECUTABLE: u64 = 0 << 63;
const PG_GLOBAL: u64 = 1 << 8;

static mut FRAME_BITMAP: [u8; MAX_PAGE / 8] = [0; MAX_PAGE / 8];

pub fn zero_page4k(page_addr: usize) {
    unsafe {
        for i in 0..4096 {
            *((page_addr + i) as *mut u8) = 0;
        }
    }
}

pub fn init_allocator() {
    unsafe { FRAME_BITMAP.fill(0); }
    unsafe {
        // 1. Create New PDPTE, PD, PT
        let (pml4_addr, _) = Cr3::read();
        let pml4_addr = pml4_addr.start_address().as_u64();
        let pdpt_addr: u64 = pml4_addr + 4096;
        PT_CURRENT = (pdpt_addr as usize) + 4096 + 4096; // pt_addr
        // here are a PD table
        zero_page4k(PT_CURRENT);
        *((pdpt_addr + 8) as *mut u64) = make_pdpte(PT_CURRENT as u64, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
        PT_CURRENT += 4096;
    }
}

pub unsafe fn alloc_frame() -> u64 {  
    // 1. find a free frame
    let mut frame_index = 0;
    let mut bitmap_index = 0;
    let mut bit_index = 0;
    'outer: for i in 0..(MAX_PAGE / 8) {
        let mut bits: [u8; 8] = [0; 8];
        bits[0] = FRAME_BITMAP[i] >> 7 & 1;
        bits[1] = FRAME_BITMAP[i] >> 6 & 1;
        bits[2] = FRAME_BITMAP[i] >> 5 & 1;
        bits[3] = FRAME_BITMAP[i] >> 4 & 1;
        bits[4] = FRAME_BITMAP[i] >> 3 & 1;
        bits[5] = FRAME_BITMAP[i] >> 2 & 1;
        bits[6] = FRAME_BITMAP[i] >> 1 & 1;
        bits[7] = FRAME_BITMAP[i] & 1;

        for j in 0..8 {
            frame_index += 1;
            if bits[j] == 0 {
                bitmap_index = i;
                bit_index = j;
                break 'outer;
            }
        }
    }

    if frame_index == 0 || frame_index == MAX_PAGE {
        println!("No free frame found");
        return 0;
    }
    // 2. set the frame to used
    FRAME_BITMAP[bitmap_index] |= (1 << (7 - bit_index));

    // 3. return the frame (real address)

    0x800000 + ((frame_index as u64) - 1 as u64) * 4096
}