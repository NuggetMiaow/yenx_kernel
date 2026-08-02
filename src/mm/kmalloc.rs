// Frame Allocator

use lazy_static::lazy_static;

static mut FRAME_BITMAP: [u8; 4194304] = [0; 4194304];

#[allow(deref_nullptr)]
pub unsafe fn kmalloc() -> &'static mut u8 {
    // 1. get a frame

    &mut *(0 as *mut u8)
}
