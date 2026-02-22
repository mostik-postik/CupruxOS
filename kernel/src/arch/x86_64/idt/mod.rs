//! Interrupt Descriptor Table (IDT) — x86_64
//!
//! Регистрирует обработчики всех 256 прерываний/исключений.
//! Registers handlers for all 256 interrupts/exceptions.
//!
//! Важные векторы / Important vectors:
//!   0x00 #DE  — Division Error
//!   0x06 #UD  — Invalid Opcode
//!   0x08 #DF  — Double Fault     (критический / critical)
//!   0x0D #GP  — General Protection Fault
//!   0x0E #PF  — Page Fault       → наш VMM / our VMM
//!   0x20      — Timer IRQ        → тик планировщика / scheduler tick
//!   0x21      — Keyboard IRQ

use core::arch::asm;

// ── Дескриптор прерывания / Interrupt descriptor (16 байт) ───────────────────

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low:  u16, // биты 0–15 обработчика  / handler bits 0–15
    selector:    u16, // сегмент кода / code segment (KERNEL_CODE)
    ist:         u8,  // Interrupt Stack Table index (0 = не использовать)
    type_attr:   u8,  // тип + атрибуты / type + attributes
    offset_mid:  u16, // биты 16–31 обработчика / handler bits 16–31
    offset_high: u32, // биты 32–63 обработчика / handler bits 32–63
    reserved:    u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0, selector: 0, ist: 0,
            type_attr: 0, offset_mid: 0,
            offset_high: 0, reserved: 0,
        }
    }

    /// Создать дескриптор прерывания.
    /// Create interrupt gate descriptor.
    ///
    /// type_attr:
    ///   0x8E = Present | ring 0 | Interrupt Gate (IF сбрасывается / IF cleared)
    ///   0xEE = Present | ring 3 | Interrupt Gate (для syscall trap)
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

// ── IDTR ─────────────────────────────────────────────────────────────────────

#[repr(C, packed)]
struct IdtDescriptor {
    size:   u16,
    offset: u64,
}

// ── Таблица IDT / IDT table ───────────────────────────────────────────────────

const IDT_SIZE: usize = 256;
static mut IDT: [IdtEntry; IDT_SIZE] = [IdtEntry::missing(); IDT_SIZE];

// ── Контекст прерывания / Interrupt frame ─────────────────────────────────────

/// CPU автоматически помещает на стек при прерывании.
/// CPU automatically pushes on the stack on interrupt.
#[repr(C)]
pub struct InterruptFrame {
    pub rip:    u64,
    pub cs:     u64,
    pub rflags: u64,
    pub rsp:    u64,
    pub ss:     u64,
}

// ── Макрос для обработчиков / Handler macro ───────────────────────────────────

/// Создать naked обработчик прерывания без error code.
macro_rules! isr_handler {
    ($name:ident, $handler:expr) => {
        #[naked]
        unsafe extern "C" fn $name() {
            asm!(
                "push 0",          // фиктивный error code / dummy error code
                "call {handler}",
                "add rsp, 8",
                "iretq",
                handler = sym $handler,
                options(noreturn)
            );
        }
    };
}

/// Создать обработчик с error code.
macro_rules! isr_handler_err {
    ($name:ident, $handler:expr) => {
        #[naked]
        unsafe extern "C" fn $name() {
            asm!(
                // error code уже на стеке / error code already on stack
                "call {handler}",
                "add rsp, 8",
                "iretq",
                handler = sym $handler,
                options(noreturn)
            );
        }
    };
}

// ── Обработчики исключений / Exception handlers ───────────────────────────────

extern "C" fn handle_divide_error(frame: &InterruptFrame, _error: u64) {
    panic!("Division Error at RIP={:#x}", frame.rip);
}

extern "C" fn handle_invalid_opcode(frame: &InterruptFrame, _error: u64) {
    panic!("Invalid Opcode at RIP={:#x}", frame.rip);
}

extern "C" fn handle_double_fault(frame: &InterruptFrame, error: u64) {
    panic!("Double Fault (error={:#x}) at RIP={:#x}", error, frame.rip);
}

extern "C" fn handle_general_protection(frame: &InterruptFrame, error: u64) {
    panic!("General Protection Fault (error={:#x}) at RIP={:#x}", error, frame.rip);
}

/// Page Fault — передаём в VMM.
/// Page Fault — forward to VMM.
extern "C" fn handle_page_fault(frame: &InterruptFrame, error: u64) {
    // Виновный виртуальный адрес — в регистре CR2
    // Faulting virtual address is in CR2
    let cr2: u64;
    unsafe { asm!("mov {}, cr2", out(reg) cr2) };

    // TODO: Этап 3 — передать в mm::vmm::handle_page_fault(cr2, error)
    // TODO: Phase 3 — forward to mm::vmm::handle_page_fault(cr2, error)
    panic!(
        "Page Fault at RIP={:#x} addr={:#x} error={:#x}",
        frame.rip, cr2, error
    );
}

/// Timer IRQ — тик планировщика.
/// Timer IRQ — scheduler tick.
extern "C" fn handle_timer(_frame: &InterruptFrame, _error: u64) {
    // Сообщить PIC что прерывание обработано (EOI)
    // Tell PIC interrupt is handled (EOI)
    unsafe { pic_eoi(0x20); }

    // TODO: Этап 5 — вызвать sched::tick()
    // TODO: Phase 5 — call sched::tick()
}

extern "C" fn handle_spurious(_frame: &InterruptFrame, _error: u64) {
    // Ложное прерывание от PIC — игнорируем / Spurious IRQ — ignore
}

// ── Naked заглушки / Naked stubs ──────────────────────────────────────────────

isr_handler!(isr_divide_error,      handle_divide_error);
isr_handler!(isr_invalid_opcode,    handle_invalid_opcode);
isr_handler_err!(isr_double_fault,  handle_double_fault);
isr_handler_err!(isr_gp_fault,      handle_general_protection);
isr_handler_err!(isr_page_fault,    handle_page_fault);
isr_handler!(isr_timer,             handle_timer);
isr_handler!(isr_spurious,          handle_spurious);

// ── PIC (8259) ────────────────────────────────────────────────────────────────

const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val);
}

/// Сообщить PIC об окончании обработки прерывания.
/// Send End-Of-Interrupt to PIC.
unsafe fn pic_eoi(irq: u8) {
    if irq >= 8 { outb(PIC2_CMD, 0x20); }
    outb(PIC1_CMD, 0x20);
}

/// Инициализировать PIC и ремаппировать IRQ 0–15 на векторы 0x20–0x2F.
/// Initialize PIC and remap IRQ 0–15 to vectors 0x20–0x2F.
///
/// По умолчанию IRQ 0–7 → векторы 0x08–0x0F — конфликт с исключениями!
/// By default IRQ 0–7 → vectors 0x08–0x0F — conflicts with exceptions!
unsafe fn pic_init() {
    // ICW1: начало инициализации / start initialization
    outb(PIC1_CMD,  0x11);
    outb(PIC2_CMD,  0x11);
    // ICW2: векторные смещения / vector offsets
    outb(PIC1_DATA, 0x20); // IRQ 0–7  → 0x20–0x27
    outb(PIC2_DATA, 0x28); // IRQ 8–15 → 0x28–0x2F
    // ICW3: каскадирование / cascade
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);
    // ICW4: режим 8086 / 8086 mode
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);
    // Маска: разрешить только Timer (IRQ0) и Keyboard (IRQ1)
    // Mask: allow only Timer (IRQ0) and Keyboard (IRQ1)
    outb(PIC1_DATA, 0b11111100);
    outb(PIC2_DATA, 0b11111111);
}

// ── Инициализация / Initialization ───────────────────────────────────────────

/// Инициализировать IDT и загрузить IDTR.
/// Initialize IDT and load IDTR.
pub fn init() {
    use super::gdt::KERNEL_CODE;

    unsafe {
        // Заполнить записи IDT
        // Fill IDT entries
        let set = |vec: usize, handler: u64, ist: u8, attr: u8| {
            IDT[vec] = IdtEntry::new(handler, KERNEL_CODE, ist, attr);
        };

        // Исключения процессора / CPU exceptions
        set(0x00, isr_divide_error   as u64, 0, 0x8E);
        set(0x06, isr_invalid_opcode as u64, 0, 0x8E);
        set(0x08, isr_double_fault   as u64, 1, 0x8E); // IST 1 — отдельный стек
        set(0x0D, isr_gp_fault       as u64, 0, 0x8E);
        set(0x0E, isr_page_fault     as u64, 0, 0x8E);

        // IRQ после ремаппинга / IRQ after remapping
        set(0x20, isr_timer          as u64, 0, 0x8E); // Timer
        set(0x27, isr_spurious       as u64, 0, 0x8E); // Spurious

        // Инициализировать PIC / Initialize PIC
        pic_init();

        // Загрузить IDTR / Load IDTR
        let descriptor = IdtDescriptor {
            size:   (core::mem::size_of::<[IdtEntry; IDT_SIZE]>() - 1) as u16,
            offset: IDT.as_ptr() as u64,
        };
        asm!("lidt [{desc}]", desc = in(reg) &descriptor);

        // Разрешить прерывания / Enable interrupts
        asm!("sti");
    }
}
