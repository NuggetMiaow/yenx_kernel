use x86_64::registers::control::Cr3;

use crate::{mm::{frame_alloc::{alloc_frame, dealloc_frame, zero_page4k}, paging::{make_pde, make_pte}}, println};

pub static mut PT_CURRENT: usize = 0;

const PG_PRESENT: u64 = 1;
const PG_WRITEABLE: u64 = 1 << 1;
const PG_USER: u64 = 1 << 2;
const PG_CACHED: u64 = 1 << 3;
const PG_EXECUTABLE: u64 = 0 << 63;
const PG_GLOBAL: u64 = 1 << 8;

static mut PDE_COUNT: usize = 0;
static mut PT_COUNT: usize = 0;

unsafe fn translate_addr(physaddr: u64) -> u64 {
    let (pml4_frame, _) = Cr3::read();
    let pml4_base = pml4_frame.start_address().as_u64();

    for pml4_idx in 0usize..512 {
        let pml4e = *((pml4_base + pml4_idx as u64 * 8) as *const u64);
        if pml4e & PG_PRESENT == 0 {
            continue;
        }
        let pdpt_base = pml4e & 0x000F_FFFF_FFFF_F000;

        for pdpt_idx in 0usize..512 {
            let pdpte = *((pdpt_base + pdpt_idx as u64 * 8) as *const u64);
            if pdpte & PG_PRESENT == 0 {
                continue;
            }

            if pdpte & (1 << 7) != 0 {
                let page_base = pdpte & 0x000F_FFFF_C000_0000;
                if physaddr >= page_base && physaddr < page_base + 0x4000_0000 {
                    return (pml4_idx as u64) << 39
                        | (pdpt_idx as u64) << 30
                        | (physaddr - page_base);
                }
                continue;
            }

            let pd_base = pdpte & 0x000F_FFFF_FFFF_F000;

            for pd_idx in 0usize..512 {
                let pde = *((pd_base + pd_idx as u64 * 8) as *const u64);
                if pde & PG_PRESENT == 0 {
                    continue;
                }

                if pde & (1 << 7) != 0 {
                    let page_base = pde & 0x000F_FFFF_FFE0_0000;
                    if physaddr >= page_base && physaddr < page_base + 0x20_0000 {
                        return (pml4_idx as u64) << 39
                            | (pdpt_idx as u64) << 30
                            | (pd_idx as u64) << 21
                            | (physaddr - page_base);
                    }
                    continue;
                }

                let pt_base = pde & 0x000F_FFFF_FFFF_F000;

                for pt_idx in 0usize..512 {
                    let pte = *((pt_base + pt_idx as u64 * 8) as *const u64);
                    if pte & PG_PRESENT == 0 {
                        continue;
                    }
                    let page_base = pte & 0x000F_FFFF_FFFF_F000;
                    if physaddr >= page_base && physaddr < page_base + 0x1000 {
                        return (pml4_idx as u64) << 39
                            | (pdpt_idx as u64) << 30
                            | (pd_idx as u64) << 21
                            | (pt_idx as u64) << 12
                            | (physaddr & 0xFFF);
                    }
                }
            }
        }
    }

    0
}

pub unsafe fn kmalloc(size: usize) -> *mut u8 {
    let mut virt_addr: u64 = 0;

    if size == 0 {
        return core::ptr::null_mut::<u8>();
    }

    let mut count = size / 4096;
    if size % 4096 != 0 {
        count += 1;
    }

    let total_size = count * 4096 + 8;

    for i in 0..count {
        let frame = alloc_frame();
        if frame == 0 {
            return core::ptr::null_mut::<u8>();
        }

        let (pml4_addr, _) = Cr3::read();
        let pml4_addr = pml4_addr.start_address().as_u64();
        let pdpte_addr: u64 = pml4_addr + 4096 + 8;
        let pdpte = *(pdpte_addr as *const u64);
        let pd_base = pdpte & 0x000FFFFFFFFFF000;
        if PT_COUNT == 512 {
            PT_COUNT = 0;
            PDE_COUNT += 1;
            PT_CURRENT += 4096;
        }
        let pde_addr = pd_base + PDE_COUNT as u64 * 8;
        let pde = *(pde_addr as *const u64);
        if pde == 0 {
            *(pde_addr as *mut u64) = make_pde(false, PT_CURRENT as u64, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
            zero_page4k(PT_CURRENT);
            *(PT_CURRENT as *mut u64) = make_pte(frame, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
            PT_COUNT += 1;
        } else {
            *((PT_CURRENT + PT_COUNT * 8) as *mut u64) = make_pte(frame, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
            PT_COUNT += 1;
        }

        if i == 0 {
            virt_addr = translate_addr(frame);
        }
    }

    let ptr = virt_addr as *mut u64;
    *ptr = total_size as u64;

    (virt_addr + 8) as *mut u8
}

pub unsafe fn kfree(virt_addr: *mut u8) {
    if virt_addr.is_null() {
        return;
    }

    let ptr = (virt_addr as u64 - 8) as *mut u64;
    let total_size = *ptr as usize;

    let mut count = total_size / 4096;

    let vaddr = virt_addr as u64 - 8;
    let (pml4_addr, _) = Cr3::read();
    let pml4_addr = pml4_addr.start_address().as_u64();

    for i in 0..count {
        let v = vaddr + (i as u64) * 4096;
        let pml4_idx = ((v >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((v >> 30) & 0x1FF) as usize;
        let pd_idx = ((v >> 21) & 0x1FF) as usize;
        let pt_idx = ((v >> 12) & 0x1FF) as usize;

        let pml4e = *((pml4_addr + pml4_idx as u64 * 8) as *const u64);
        if pml4e & PG_PRESENT == 0 {
            continue;
        }
        let pdpt_base = pml4e & 0x000FFFFFFFFFF000;

        let pdpte = *((pdpt_base + pdpt_idx as u64 * 8) as *const u64);
        if pdpte & PG_PRESENT == 0 {
            continue;
        }
        let pd_base = pdpte & 0x000FFFFFFFFFF000;

        let pde = *((pd_base + pd_idx as u64 * 8) as *const u64);
        if pde & PG_PRESENT == 0 {
            continue;
        }
        let pt_base = pde & 0x000FFFFFFFFFF000;

        let pte_addr = pt_base + pt_idx as u64 * 8;
        let pte = *(pte_addr as *const u64);
        if pte & PG_PRESENT == 0 {
            continue;
        }

        let frame = pte & 0x000FFFFFFFFFF000;
        dealloc_frame(frame);

        *(pte_addr as *mut u64) = 0;
    }
}