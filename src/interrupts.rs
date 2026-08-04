

use core::arch::global_asm;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use lazy_static::lazy_static;


use crate::process::{Context, schedule};
use crate::{debug, error, fatal, gdt, note};
use crate::apic;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        debug!("Loaded Exception handler");
        idt[apic::TIMER_VECTOR].set_handler_fn(timer_handler);
        debug!("APIC Timer has been registered");
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn divide_error_handler(
    stack_frame: InterruptStackFrame)
{
    fatal!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn debug_handler(
    stack_frame: InterruptStackFrame)
{
    note!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(
    stack_frame: InterruptStackFrame)
{
    fatal!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    debug!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(
    stack_frame: InterruptStackFrame)
{
    error!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: InterruptStackFrame)
{
    fatal!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame)
{
    fatal!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn device_not_available_handler(
    stack_frame: InterruptStackFrame)
{
    error!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    fatal!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    fatal!("Error code: {}", error_code);
    loop {}
}

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
)
{
    fatal!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
    fatal!("Error code: {}", error_code);
    loop {}
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
)
{
    fatal!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
    fatal!("Error code: {}", error_code);
    loop {}
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
)
{
    fatal!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
    fatal!("Error code: {}", error_code);
    loop {}
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
)
{
    fatal!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
    fatal!("Error code: {}", error_code);
    loop {}
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    error!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
    error!("Accessed Address: {:?}", Cr2::read());
    error!("Error Code: {:?}", error_code);
    loop {}
}

extern "x86-interrupt" fn x87_floating_point_handler(
    stack_frame: InterruptStackFrame)
{
    note!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
)
{
    fatal!("EXCEPTION: ALIGNMENT CHECK\n{:#?}", stack_frame);
    fatal!("Error code: {}", error_code);
    loop {}
}

extern "x86-interrupt" fn machine_check_handler(
    stack_frame: InterruptStackFrame)
-> ! {
    fatal!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn simd_floating_point_handler(
    stack_frame: InterruptStackFrame)
{
    note!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn virtualization_handler(
    stack_frame: InterruptStackFrame)
{
    note!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
}

global_asm!(
    ".global save_register",
    "save_register:",
    "mov [rdi], rax",
    "mov [rdi + 8], rbx",
    "mov [rdi + 16], rcx",
    "mov [rdi + 24], rdx",
    "mov [rdi + 32], r8",
    "mov [rdi + 40], r9",
    "mov [rdi + 48], r10",
    "mov [rdi + 56], r11",
    "mov [rdi + 64], r12",
    "mov [rdi + 72], r13",
    "mov [rdi + 80], r14",
    "mov [rdi + 88], r15",
    "ret"
);

extern "C" {
    fn save_register(context: *mut Context);
}

extern "x86-interrupt" fn timer_handler(
    stack_frame: InterruptStackFrame)
{
    let mut context: Context = Context::new();
    unsafe {
        save_register(&mut context); 
        schedule(context, stack_frame);
    }
    apic::x2apic_eoi();
    apic::x2apic_timer_rearm();
}