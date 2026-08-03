use core::char::MAX;

use crate::{print, println};

const MAX_PAGE: usize = 131702;

static mut FRAME_BITMAP: [u8; MAX_PAGE / 8] = [0; MAX_PAGE / 8];

pub fn init_bitmap() {
    unsafe { FRAME_BITMAP.fill(0); }
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