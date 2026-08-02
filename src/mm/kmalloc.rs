// Frame Allocator

use lazy_static::lazy_static;
use x86_64::{registers::control::Cr3, structures::paging::PageTable};

use crate::{mm::paging::{make_pde, make_pte}, println};

const MAX_PAGE_SIZE_DIV8: usize = 4194304;

static mut FRAME_BITMAP: [u8; MAX_PAGE_SIZE_DIV8] = [0; MAX_PAGE_SIZE_DIV8];

const PG_PRESENT: i32 = 1;
const PG_UNPERSENT: i32 = 0;
const PG_READONLY: i32 = 0;
const PG_WRITEABLE: i32 = 1;
const PG_USER: i32 = 1;
const PG_SUPERVISOR: i32 = 0;
const PG_UNEXECUTABLE: i32 = 1;
const PG_EXECUTABLE: i32 = 0;
const PG_HUGEPAGE: i32 = 1;
const PG_NONHUGEPAGE: i32 = 0;


#[allow(deref_nullptr)]
pub unsafe fn kmalloc(size: u64) -> &'static mut u8 {
    // 1. get a frame
    let mut frame_index = 0;
    let left_frame_src = size / 4096;
    let mut left_frame = left_frame_src;
    let mut selected = false;
    let mut got = false;

    if size < 4096 { left_frame = 1; }
    if size % 4096 != 0 { left_frame += 1; }

    'outer: for i in 2048..MAX_PAGE_SIZE_DIV8 {
        let mut bits: [u8; 8] = [0; 8];
        bits[0] = FRAME_BITMAP[i] >> 7;
        bits[1] = (FRAME_BITMAP[i] >> 6) & 1;
        bits[2] = (FRAME_BITMAP[i] >> 5) & 1;
        bits[3] = (FRAME_BITMAP[i] >> 4) & 1;
        bits[4] = (FRAME_BITMAP[i] >> 3) & 1;
        bits[5] = (FRAME_BITMAP[i] >> 2) & 1;
        bits[6] = (FRAME_BITMAP[i] >> 1) & 1;
        bits[7] = FRAME_BITMAP[i] & 1;

        for j in 0..8 {
            if bits[j] == 0 {
                if selected == false {
                    frame_index = (i - 2048) * 8 + j;
                    selected = true;

                }
                left_frame -= 1;
                if left_frame == 0 {
                    got = true;
                    break 'outer;
                }
            } else {
                selected = false;
                left_frame = left_frame_src;
            }
        }
    }

    if got {
        // TODO: Process PDPTE and PML4E
        // 1. Get the position
     let pml4_addr = 0x10000;
        let pdpt_addr = pml4_addr + 4096;
        let pd_addr = pdpt_addr + 4096;
        let pt_addr = pd_addr + 4096;

        // 2. Get the address of physic memory
        let mem_physaddr = 0x800000 + (frame_index as u64) * 4096;

        // 3. get the index of PT
        let pt_index = frame_index % 512;

        // 4. Get the address pf PTE
        let pte_addr = pt_addr + (pt_index as u64) * 8;

        // 5. Initialize PTE
        let pte = make_pte(PG_PRESENT, PG_WRITEABLE, PG_USER, 
            0, 0, 0, 0, 0, 1, mem_physaddr, PG_EXECUTABLE);
        unsafe { *(pte_addr as *mut u64) = pte; }
        
        // 6. Set PDE
        let pde_addr = pd_addr + 8; // PD[1]
        let pde_content = *(pde_addr as *const u64);

        if pde_content == 0 {
            let pde = make_pde(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 
                0, 0, 0, PG_NONHUGEPAGE, pt_addr, PG_EXECUTABLE);
            *(pde_addr as *mut u64) = pde;
        }

        // 7. Flush TLB
        x86_64::instructions::tlb::flush_all();

        return &mut *(mem_physaddr as *mut u8);
    } else {
        println!("ERROR: Cannot malloc memory!");
    }

    &mut *(0 as *mut u8)
}
