use core::ptr::write_volatile;

use x86_64::{PhysAddr, registers::control::Cr3, structures::paging::PhysFrame};

const PG_PRESENT: u64 = 1;
const PG_WRITEABLE: u64 = 1 << 1;
const PG_USER: u64 = 1 << 2;
const PG_CACHED: u64 = 1 << 3;
const PG_EXECUTABLE: u64 = 0 << 63;
const PG_GLOBAL: u64 = 1 << 8;

pub unsafe fn make_pml4e(physaddr: u64, flags: u64) -> u64 {
    0 | flags | (physaddr & 0x000FFFFFFFFFF000)
}

pub unsafe fn make_pdpte(physaddr: u64, flags: u64) -> u64 {
    0 | flags | (physaddr & 0x000FFFFFFFFFF000)
}

pub unsafe fn make_pde(ps: bool, physaddr: u64, flags: u64) -> u64 {
    if ps { 0 | 1 << 7 | flags | (physaddr & 0x000FFFFFFFFFE000) } 
    else { 0 | flags | (physaddr & 0x000FFFFFFFFFF000) }
}

pub unsafe fn make_pte(physaddr: u64, flags: u64) -> u64 {
    0 | flags | (physaddr & 0x000FFFFFFFFFF000)
}

pub unsafe fn init() -> &'static mut u8 {
    let pml4_addr: u64 = 0x10000;
    let pdpt_addr: u64 = pml4_addr + 4096;
    let pd_addr: u64 = pdpt_addr + 4096;
    //let pt_addr: u64 = pd_addr + 4096;

    let mut ptr = pml4_addr as *mut u8;
    for _ in 0..(4096 * 3) {
        write_volatile(&mut *ptr, 0);
        ptr = ptr.add(1);
    }

    // 1. create PML4E always maping
    let pml4e = make_pml4e(pdpt_addr, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
    let pdpt = make_pdpte(pd_addr, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
    let pd1 = make_pde(true, 0, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
    let pd2 = make_pde(true, 0x200000, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);
    let pd3 = make_pde(true, 0x400000, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);    
    let pd4 = make_pde(true, 0x600000, PG_PRESENT | PG_WRITEABLE | PG_EXECUTABLE);

    write_volatile(&mut *(pml4_addr as *mut u64), pml4e);
    write_volatile(&mut *(pdpt_addr as *mut u64), pdpt);
    write_volatile(&mut *(pd_addr as *mut u64), pd1);
    write_volatile(&mut *((pd_addr + 8) as *mut u64), pd2);
    write_volatile(&mut *((pd_addr + 16) as *mut u64), pd3);
    write_volatile(&mut *((pd_addr + 24) as *mut u64), pd4);

    let (_, cr3_flags) = Cr3::read();
    Cr3::write(PhysFrame::containing_address(PhysAddr::new(pml4_addr)), cr3_flags);

    &mut *(pml4_addr as *mut u8)
}