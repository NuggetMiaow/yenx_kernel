use core::arch::x86_64::__cpuid;
use x86_64::registers::model_specific::{ApicBase, ApicBaseFlags};

use x86_64::instructions::port::PortWriteOnly;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;
const ICW4_8086: u8 = 0x01;

pub fn disable_8259pic() {
    let mut cmd1 = PortWriteOnly::<u8>::new(PIC1_COMMAND);
    let mut data1 = PortWriteOnly::<u8>::new(PIC1_DATA);
    let mut cmd2 = PortWriteOnly::<u8>::new(PIC2_COMMAND);
    let mut data2 = PortWriteOnly::<u8>::new(PIC2_DATA);

    unsafe {
        cmd1.write(ICW1_INIT | ICW1_ICW4);
        cmd2.write(ICW1_INIT | ICW1_ICW4);
        data1.write(0x20);
        data2.write(0x28);
        data1.write(4);
        data2.write(2);
        data1.write(ICW4_8086);
        data2.write(ICW4_8086);
        data1.write(0xFF);
        data2.write(0xFF);
    }
}

pub fn check_apic() -> bool {
    __cpuid(1).edx & (1 << 9) != 0
}

pub fn check_x2apic() -> bool {
    __cpuid(1).ecx & (1 << 21) != 0
}

pub fn enable_x2apic() {
    if !check_x2apic() {
        //println!("x2APIC not supported!");
        return;
    }

    disable_8259pic();

    let (frame, _) = ApicBase::read_raw();
    unsafe {
        ApicBase::write_raw(
            frame,
            ApicBaseFlags::LAPIC_ENABLE.bits() | ApicBaseFlags::X2APIC_ENABLE.bits(),
        );
    }
}

const X2APIC_BASE: u32        = 0x800;
const X2APIC_ID: u32          = X2APIC_BASE + 0x02;
const X2APIC_VERSION: u32     = X2APIC_BASE + 0x03;
const X2APIC_SPURIOUS: u32    = X2APIC_BASE + 0x0F;
const X2APIC_EOI: u32         = X2APIC_BASE + 0x0B;
const X2APIC_LVT_TIMER: u32   = X2APIC_BASE + 0x32;
const X2APIC_LVT_THERMAL: u32 = X2APIC_BASE + 0x34;
const X2APIC_LVT_PERF: u32    = X2APIC_BASE + 0x35;
const X2APIC_LVT_LINT0: u32   = X2APIC_BASE + 0x36;
const X2APIC_LVT_LINT1: u32   = X2APIC_BASE + 0x37;
const X2APIC_LVT_ERROR: u32   = X2APIC_BASE + 0x30;
const X2APIC_TIMER_INIT: u32  = X2APIC_BASE + 0x38;
const X2APIC_TIMER_CURRENT: u32 = X2APIC_BASE + 0x39;
const X2APIC_TIMER_DIV: u32   = X2APIC_BASE + 0x3E;

pub const TIMER_VECTOR: u8 = 0x20;

fn x2apic_write(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        core::arch::asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high);
    }
}

fn x2apic_read(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        core::arch::asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high);
    }
    (high as u64) << 32 | (low as u64)
}

pub fn apic_init() {
    debug!("Ready to init x2apic");
    let ver = x2apic_read(X2APIC_VERSION);
    let id = x2apic_read(X2APIC_ID);
    info!("APIC ID: {}, Version: {}", id, (ver >> 16) & 0xFF);

    x2apic_write(X2APIC_SPURIOUS, 0xFF | (1 << 8));

    let masked = 1u64 << 16;
    x2apic_write(X2APIC_LVT_TIMER, masked);
    x2apic_write(X2APIC_LVT_THERMAL, masked);
    x2apic_write(X2APIC_LVT_PERF, masked);
    x2apic_write(X2APIC_LVT_LINT0, masked);
    x2apic_write(X2APIC_LVT_LINT1, masked);
    x2apic_write(X2APIC_LVT_ERROR, masked);
}

pub fn apic_timer_init() {
    x2apic_write(X2APIC_TIMER_DIV, 0);
    // One-shot 模式：不设 bit 17
    x2apic_write(X2APIC_LVT_TIMER, TIMER_VECTOR as u64);
    x2apic_write(X2APIC_TIMER_INIT, 0x1);
}

pub fn x2apic_eoi() {
    x2apic_write(X2APIC_EOI, 0);
}

pub fn apic_timer_diag() {
    // 读回刚写入的寄存器，确认写入成功
    let div = x2apic_read(X2APIC_TIMER_DIV);
    let lvt = x2apic_read(X2APIC_LVT_TIMER);
    let init = x2apic_read(X2APIC_TIMER_INIT);
    let cur = x2apic_read(X2APIC_TIMER_CURRENT);
    debug!("APIC Timer: DIV: 0x{:x}, LVT: 0x{:x}, INIT: 0x{:x}, CURRENT: 0x{:x}", div, lvt, init, cur);
}

pub fn x2apic_timer_rearm() {
    x2apic_write(X2APIC_TIMER_INIT, 0x100000);
}

use x86_64::instructions::port::PortReadOnly;

use crate::{debug, info};

const KEYBOARD_STATUS: u16 = 0x64;
const KEYBOARD_DATA: u16   = 0x60;

/// 扫描码集 1 → ASCII 映射（无 Shift）
static SCANCODE_MAP: [Option<char>; 128] = {
    let mut map: [Option<char>; 128] = [None; 128];
    // 数字行
    map[0x02] = Some('1'); map[0x03] = Some('2'); map[0x04] = Some('3');
    map[0x05] = Some('4'); map[0x06] = Some('5'); map[0x07] = Some('6');
    map[0x08] = Some('7'); map[0x09] = Some('8'); map[0x0A] = Some('9');
    map[0x0B] = Some('0');
    // 字母行
    map[0x10] = Some('q'); map[0x11] = Some('w'); map[0x12] = Some('e');
    map[0x13] = Some('r'); map[0x14] = Some('t'); map[0x15] = Some('y');
    map[0x16] = Some('u'); map[0x17] = Some('i'); map[0x18] = Some('o');
    map[0x19] = Some('p');
    map[0x1E] = Some('a'); map[0x1F] = Some('s'); map[0x20] = Some('d');
    map[0x21] = Some('f'); map[0x22] = Some('g'); map[0x23] = Some('h');
    map[0x24] = Some('j'); map[0x25] = Some('k'); map[0x26] = Some('l');
    map[0x2C] = Some('z'); map[0x2D] = Some('x'); map[0x2E] = Some('c');
    map[0x2F] = Some('v'); map[0x30] = Some('b'); map[0x31] = Some('n');
    map[0x32] = Some('m');
    // 符号
    map[0x0C] = Some('-'); map[0x0D] = Some('=');
    map[0x1A] = Some('['); map[0x1B] = Some(']');
    map[0x27] = Some(';'); map[0x28] = Some('\'');
    map[0x29] = Some('`');
    map[0x33] = Some(','); map[0x34] = Some('.'); map[0x35] = Some('/');
    map[0x39] = Some(' ');
    // 特殊键
    map[0x0E] = Some('\x08'); // Backspace
    map[0x0F] = Some('\t');   // Tab
    map[0x1C] = Some('\n');   // Enter
    map[0x2B] = Some('\\');
    map
};
/* pub fn read_keyboard() {
    let mut status = PortReadOnly::<u8>::new(KEYBOARD_STATUS);
    let mut data   = PortReadOnly::<u8>::new(KEYBOARD_DATA);

    unsafe {
        if status.read() & 1 != 0 {
            let scancode = data.read();
            // 只处理按下事件（bit 7 = 0）
            if scancode < 128 {
                if let Some(c) = SCANCODE_MAP.get(scancode as usize).and_then(|&c| c) {
                    print!("{}", c);
                }
            }
        }
    }
}*/