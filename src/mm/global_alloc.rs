use core::{alloc::{GlobalAlloc, Layout}, ptr::null_mut};

use crate::mm::malloc;
 
struct KernelAllocator {}
 
#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator {};
 
unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = malloc::kmalloc(layout.size());
        if ptr.is_null() {
            return null_mut();
        }
        ptr as *mut u8
    }
 
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        malloc::kfree(ptr);
    }
}
 