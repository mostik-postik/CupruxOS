//! Interrupt Descriptor Table (IDT) — x86_64

use core::arch::{asm, naked_asm};

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low:  u16,
    selector:    u16,
    ist:         u8,
    type_attr:   u8,
    offset_mid:  u16,
    offset_high: u32,
    reserved:    u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self { offset_low: 0, selector: 0, ist: 0,
               type_attr: 0, offset_mid: 0,
               offset_high: 0, reserved: 0 }
    }

    fn new(handler: u64, selector: u16, ist: u8, type_attr: u8) -> Self {
        Self {
            offset_low:  (handler & 0xFFFF) as u16,
            selector,
            ist,
            type_attr,
            offset_mid:  ((handler >> 16) & 0xFFFF) as u16,
            offset_high: (handler >> 32) as u32,
            reserved:    0,
        }
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    size:   u16,
    offset: u64,
}

const IDT_SIZE: usize = 256;
static mut IDT: [IdtEntry; IDT_SIZE] = [IdtEntry::missing(); IDT_SIZE];

#[repr(C)]
pub struct InterruptFrame {
    pub rip:    u64,
    pub cs:     u64,
    pub rflags: u64,
    pub rsp:    u64,
    pub ss:     u64,
}

// ── Макросы для обработчиков / Handler macros ─────────────────────────────────

macro_rules! isr_handler {
    ($name:ident, $handler:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            naked_asm!(
                "push 0",
                "call {handler}",
                "add rsp, 8",
                "iretq",
                handler = sym $handler,
            );
        }
    };
}

macro_rules! isr_handler_err {
    ($name:ident, $handler:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            naked_asm!(
                "call {handler}",
                "add rsp, 8",
                "iretq",
                handler = sym $handler,
            );
        }
    };
}

// ── Обработчики / Handlers ────────────────────────────────────────────────────

extern "C" fn handle_divide_error(frame: &InterruptFrame, _e: u64) {
    panic!("Division Error at RIP={:#x}", frame.rip);
}

extern "C" fn handle_invalid_opcode(frame: &InterruptFrame, _e: u64) {
    panic!("Invalid Opcode at RIP={:#x}", frame.rip);
}

extern "C" fn handle_double_fault(frame: &InterruptFrame, e: u64) {
    panic!("Double Fault (err={:#x}) at RIP={:#x}", e, frame.rip);
}

extern "C" fn handle_general_protection(frame: &InterruptFrame, e: u64) {
    panic!("General Protection Fault (err={:#x}) at RIP={:#x}", e, frame.rip);
}

extern "C" fn handle_page_fault(frame: &InterruptFrame, e: u64) {
    let cr2: u64;
    unsafe { asm!("mov {}, cr2", out(reg) cr2) };
    panic!("Page Fault at RIP={:#x} addr={:#x} err={:#x}", frame.rip, cr2, e);
}

extern "C" fn handle_timer(_frame: &InterruptFrame, _e: u64) {
    unsafe { pic_eoi(0x20); }
    // TODO: sched::tick()
}

extern "C" fn handle_spurious(_frame: &InterruptFrame, _e: u64) {}

isr_handler!(isr_divide_error,   handle_divide_error);
isr_handler!(isr_invalid_opcode, handle_invalid_opcode);
isr_handler_err!(isr_double_fault,  handle_double_fault);
isr_handler_err!(isr_gp_fault,      handle_general_protection);
isr_handler_err!(isr_page_fault,    handle_page_fault);
isr_handler!(isr_timer,    handle_timer);
isr_handler!(isr_spurious, handle_spurious);

// ── PIC ───────────────────────────────────────────────────────────────────────

const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

unsafe fn outb(port: u16, val: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") val); }
}

unsafe fn pic_eoi(irq: u8) {
    unsafe {
        if irq >= 8 { outb(PIC2_CMD, 0x20); }
        outb(PIC1_CMD, 0x20);
    }
}

unsafe fn pic_init() {
    unsafe {
        outb(PIC1_CMD,  0x11);
        outb(PIC2_CMD,  0x11);
        outb(PIC1_DATA, 0x20);
        outb(PIC2_DATA, 0x28);
        outb(PIC1_DATA, 0x04);
        outb(PIC2_DATA, 0x02);
        outb(PIC1_DATA, 0x01);
        outb(PIC2_DATA, 0x01);
        outb(PIC1_DATA, 0b11111100);
        outb(PIC2_DATA, 0b11111111);
    }
}

// ── Init ──────────────────────────────────────────────────────────────────────

pub fn init() {
    use super::gdt::KERNEL_CODE;

    unsafe {
        let set = |vec: usize, handler: u64, ist: u8, attr: u8| {
            IDT[vec] = IdtEntry::new(handler, KERNEL_CODE, ist, attr);
        };

        set(0x00, isr_divide_error   as *const () as u64, 0, 0x8E);
        set(0x06, isr_invalid_opcode as *const () as u64, 0, 0x8E);
        set(0x08, isr_double_fault   as *const () as u64, 1, 0x8E);
        set(0x0D, isr_gp_fault       as *const () as u64, 0, 0x8E);
        set(0x0E, isr_page_fault     as *const () as u64, 0, 0x8E);
        set(0x20, isr_timer          as *const () as u64, 0, 0x8E);
        set(0x27, isr_spurious       as *const () as u64, 0, 0x8E);

        pic_init();

        let descriptor = IdtDescriptor {
            size:   (core::mem::size_of::<[IdtEntry; IDT_SIZE]>() - 1) as u16,
            offset: (&raw const IDT) as u64,
        };
        asm!("lidt [{desc}]", desc = in(reg) &descriptor);
        asm!("sti");
    }
}
