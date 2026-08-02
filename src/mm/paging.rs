use core::ptr::write_volatile;

use x86_64::{PhysAddr, registers::control::Cr3, structures::paging::PhysFrame};

unsafe fn make_pml4e(p: i32, rw: i32, user: i32, pwt: i32, pcd: i32, access: i32, dirty: i32, ps: i32, physaddr: u64, xd: i32) -> u64 {
    let mut pml4e: u64 = 0;
    pml4e |= p as u64;
    pml4e |= (rw as u64) << 1;
    pml4e |= (user as u64) << 2;
    pml4e |= (pwt as u64) << 3;
    pml4e |= (pcd as u64) << 4;
    pml4e |= (access as u64) << 5;
    pml4e |= (dirty as u64) << 6;
    pml4e |= (ps as u64) << 7;
    pml4e |= physaddr & 0x000FFFFFFFFFF000;  // aligned
    pml4e |= (0 as u64) << 52;
    pml4e |= (xd as u64) << 63;
    pml4e
}

unsafe fn make_pdpte(p: i32, rw: i32, user: i32, pwt: i32, pcd: i32, access: i32, dirty: i32, ps: i32, physaddr: u64, xd: i32) -> u64 {
    let mut pdpt: u64 = 0;
    pdpt |= p as u64;
    pdpt |= (rw as u64) << 1;
    pdpt |= (user as u64) << 2;
    pdpt |= (pwt as u64) << 3;
    pdpt |= (pcd as u64) << 4;
    pdpt |= (access as u64) << 5;
    pdpt |= (dirty as u64) << 6;
    pdpt |= (ps as u64) << 7;
    pdpt |= physaddr & 0x000FFFFFFFFFF000;  // aligned
    pdpt |= (0 as u64) << 52;
    pdpt |= (xd as u64) << 63;
    pdpt
}

unsafe fn make_pde_2mb(p: i32, rw: i32, user: i32, pwt: i32, pcd: i32, access: i32, dirty: i32, ps: i32, g: i32, pat: i32, physaddr: u64, xd: i32) -> u64 {
    let mut pd: u64 = 0;
    pd |= p as u64;
    pd |= (rw as u64) << 1;
    pd |= (user as u64) << 2;
    pd |= (pwt as u64) << 3;
    pd |= (pcd as u64) << 4;
    pd |= (access as u64) << 5;
    pd |= (dirty as u64) << 6;
    pd |= (ps as u64) << 7;
    pd |= (g as u64) << 8;
    pd |= (pat as u64) << 12;
    // 2MB huge page
    pd |= physaddr & 0x000FFFFFFFFFE000;  // 2MB Aligned
    pd |= (0 as u64) << 52;
    pd |= (xd as u64) << 63;
    pd
}

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


pub unsafe fn init() -> &'static mut u8 {
    let pml4_addr: u64 = 0x10000;
    let pdpt_addr: u64 = pml4_addr + 4096;
    let pd_addr: u64 = pdpt_addr + 4096;
    let pt_addr: u64 = pd_addr + 4096;

    let mut ptr = pml4_addr as *mut u8;
    for _ in 0..(4096 * 3) {
        write_volatile(&mut *ptr, 0);
        ptr = ptr.add(1);
    }

    // 1. create PML4E always maping
    let pml4e = make_pml4e(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 0, 0, 0, 0, PG_NONHUGEPAGE, pdpt_addr, PG_EXECUTABLE);
    let pdpt = make_pdpte(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 0, 0, 0, 0, PG_NONHUGEPAGE, pd_addr, PG_EXECUTABLE);
    let pd1 = make_pde_2mb(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 0, 0, 0, 0, PG_HUGEPAGE, 0, 0, 0, PG_EXECUTABLE);
    let pd2 = make_pde_2mb(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 0, 0, 0, 0, PG_HUGEPAGE, 0, 0, 0x200000, PG_EXECUTABLE);
    let pd3 = make_pde_2mb(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 0, 0, 0, 0, PG_HUGEPAGE, 0, 0, 0x400000, PG_EXECUTABLE);    
    let pd4 = make_pde_2mb(PG_PRESENT, PG_WRITEABLE, PG_SUPERVISOR, 0, 0, 0, 0, PG_HUGEPAGE, 0, 0, 0x600000, PG_EXECUTABLE);

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