extern crate alloc;

use core::{any::Any, arch::global_asm};
use alloc::vec::Vec;
use x86_64::{registers::{self, segmentation::{CS, SS, Segment}}, structures::idt::InterruptStackFrame};

use crate::{debug, info, mm::malloc, process::ProcessState::{Ready, Running}};

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct Context {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub ss: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub cs: u64,
    pub rip: u64,
}

impl Context {
    pub fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            ss: 0,
            rsp: 0,
            rflags: 0,
            cs: 0,
            rip: 0
        }
    }
}

#[derive(Clone, Copy)]
pub enum ProcessState {
    Running,
    Ready
}

#[derive(Clone)]
pub struct Process {
    pub state: ProcessState,
    pub context: Context,
    pub stack: *mut u8
}

global_asm!(
    ".global switch_to",
    "switch_to:",
    "mov rax, [rdi]",
    "mov rbx, [rdi + 8]",
    "mov rcx, [rdi + 16]",
    "mov rdx, [rdi + 24]",
    "mov r8, [rdi + 32]",
    "mov r9, [rdi + 40]",
    "mov r10, [rdi + 48]",
    "mov r11, [rdi + 56]",
    "mov r12, [rdi + 64]",
    "mov r13, [rdi + 72]",
    "mov r14, [rdi + 80]",
    "mov r15, [rdi + 88]",
    "push [rdi + 96]",
    "push [rdi + 104]",
    "push [rdi + 112]",
    "push [rdi + 120]",
    "push [rdi + 128]",
    "iretq"
);

pub static mut PROCESSES: Vec<Process> = Vec::<Process>::new();

pub fn spawn(entry: extern "C" fn()->!) {
    debug!("spawn a process");
    let mut ctx = Context::new();

    ctx.ss = SS::get_reg().0 as u64;
    let stack_size = 64000usize;
    let stack = unsafe { malloc::kmalloc(64000) };

    let stack_top = (stack as u64) + stack_size as u64;
    ctx.rsp = stack_top;
    ctx.rflags = registers::rflags::read_raw();
    ctx.cs = CS::get_reg().0 as u64;
    ctx.rip = entry as u64;


    unsafe {
        PROCESSES.push(Process {
            state: ProcessState::Ready,
            context: ctx,
            stack: stack
        });
    }
}

pub static mut CURRENT_PID: usize = 0;

extern "C" {
    fn switch_to(ctx: Context) -> !;
}

pub unsafe fn schedule(regs: Context, frame: InterruptStackFrame) {
    let mut old_ctx = regs;
    old_ctx.ss = frame.stack_segment.0 as u64;
    old_ctx.rsp = frame.stack_pointer.as_u64();
    old_ctx.rflags = frame.cpu_flags.bits();
    old_ctx.cs = frame.code_segment.0 as u64;
    old_ctx.rip = frame.instruction_pointer.as_u64();

    if CURRENT_PID >= PROCESSES.len() {
        CURRENT_PID = 0;
    }

    let len = PROCESSES.len();

    if CURRENT_PID != 0 {
        let old_p = &mut PROCESSES[CURRENT_PID - 1];
        old_p.state = Ready;
        old_p.context = old_ctx;
    } else {
        let old_p = &mut PROCESSES[len - 1];
        old_p.state = Ready;      
        old_p.context = old_ctx;  
    }

    let p: &mut Process = &mut PROCESSES[CURRENT_PID];
    p.state = Running;

    info!("hooray");
    switch_to(p.context);
}