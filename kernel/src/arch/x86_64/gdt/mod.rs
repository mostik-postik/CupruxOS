//! Global Descriptor Table (GDT) — x86_64
//!
//! Структура GDT для CupruxOS / CupruxOS GDT layout:
//!
//!  Индекс / Index  Сегмент / Segment
//!  ─────────────────────────────────
//!  0               Null descriptor (обязателен / required)
//!  1               Kernel Code  (ring 0, execute)
//!  2               Kernel Data  (ring 0, read/write)
//!  3               User Code    (ring 3, execute)
//!  4               User Data    (ring 3, read/write)
//!  5               TSS          (Task State Segment)

use core::mem::size_of;

// ── Селекторы сегментов / Segment selectors ───────────────────────────────────
pub const KERNEL_CODE: u16 = 0x08;
pub const KERNEL_DATA: u16 = 0x10;
pub const USER_CODE:   u16 = 0x1B;
pub const USER_DATA:   u16 = 0x23;
pub const TSS_SEL:     u16 = 0x28;

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low:   u16,
    base_low:    u16,
    base_mid:    u8,
    access:      u8,
    granularity: u8,
    base_high:   u8,
}

impl GdtEntry {
    const fn null() -> Self {
        Self { limit_low: 0, base_low: 0, base_mid: 0,
               access: 0, granularity: 0, base_high: 0 }
    }
    const fn new(access: u8, granularity: u8) -> Self {
        Self { limit_low: 0xFFFF, base_low: 0, base_mid: 0,
               access, granularity, base_high: 0 }
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct TssEntry {
    limit_low:   u16,
    base_low:    u16,
    base_mid:    u8,
    access:      u8,
    granularity: u8,
    base_high:   u8,
    base_upper:  u32,
    reserved:    u32,
}

impl TssEntry {
    fn new(tss_addr: u64) -> Self {
        let size = (size_of::<Tss>() - 1) as u64;
        Self {
            limit_low:   (size & 0xFFFF) as u16,
            base_low:    (tss_addr & 0xFFFF) as u16,
            base_mid:    ((tss_addr >> 16) & 0xFF) as u8,
            access:      0x89,
            granularity: ((size >> 16) & 0x0F) as u8,
            base_high:   ((tss_addr >> 24) & 0xFF) as u8,
            base_upper:  (tss_addr >> 32) as u32,
            reserved:    0,
        }
    }
}

/// TSS — хранит стеки для переключения привилегий.
/// TSS — stores stacks for privilege level switches.
#[repr(C, packed)]
pub struct Tss {
    reserved0:      u32,
    /// Стек ядра для прерываний из ring 3 / Kernel stack for ring 3 interrupts
    pub rsp0:       u64,
    pub rsp1:       u64,
    pub rsp2:       u64,
    reserved1:      u64,
    /// IST — 7 стеков для критических прерываний / 7 stacks for critical interrupts
    pub ist:        [u64; 7],
    reserved2:      u64,
    reserved3:      u16,
    pub iomap_base: u16,
}

impl Tss {
    const fn new() -> Self {
        Self {
            reserved0: 0, rsp0: 0, rsp1: 0, rsp2: 0,
            reserved1: 0, ist: [0; 7], reserved2: 0,
            reserved3: 0,
            iomap_base: size_of::<Tss>() as u16,
        }
    }
}

#[repr(C, packed)]
struct Gdt {
    null:        GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code:   GdtEntry,
    user_data:   GdtEntry,
    tss:         TssEntry,
}

impl Gdt {
    const fn new(tss_addr: u64) -> Self {
        Self {
            null:        GdtEntry::null(),
            kernel_code: GdtEntry::new(0x9A, 0xA0), // ring 0, code, 64-bit
            kernel_data: GdtEntry::new(0x92, 0xC0), // ring 0, data
            user_code:   GdtEntry::new(0xFA, 0xA0), // ring 3, code, 64-bit
            user_data:   GdtEntry::new(0xF2, 0xC0), // ring 3, data
            tss:         TssEntry::new(tss_addr),
        }
    }
}

#[repr(C, packed)]
struct GdtDescriptor {
    size:   u16,
    offset: u64,
}

/// Стек ядра для прерываний из userspace (16KB).
/// Kernel interrupt stack for userspace interrupts (16KB).
static mut KERNEL_STACK: [u8; 16 * 1024] = [0; 16 * 1024];

static mut TSS: Tss = Tss::new();
static mut GDT: Gdt = Gdt::new(0);

/// Инициализировать и загрузить GDT + TSS.
/// Initialize and load GDT + TSS.
pub fn init() {
    unsafe {
        // 1. Вершина стека ядра → TSS.rsp0
        let stack_top = KERNEL_STACK.as_ptr().add(KERNEL_STACK.len()) as u64;
        TSS.rsp0 = stack_top;

        // 2. GDT с правильным адресом TSS
        GDT = Gdt::new(&TSS as *const Tss as u64);

        // 3. Загрузить GDTR
        let descriptor = GdtDescriptor {
            size:   (size_of::<Gdt>() - 1) as u16,
            offset: &GDT as *const Gdt as u64,
        };

        core::arch::asm!(
            "lgdt [{desc}]",
            // Обновить регистры данных / Update data registers
            "mov ax, {kdata}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            // Обновить CS через far return / Update CS via far return
            "push {kcode}",
            "lea rax, [rip + 1f]",
            "push rax",
            "retfq",
            "1:",
            // Загрузить TSS / Load TSS
            "ltr {tss:x}",
            desc  = in(reg) &descriptor,
            kcode = const KERNEL_CODE as u64,
            kdata = const KERNEL_DATA,
            tss   = in(reg) TSS_SEL,
            out("rax") _,
        );
    }
}

/// Обновить RSP0 в TSS при смене задачи.
/// Called by scheduler on every context switch.
pub fn set_kernel_stack(stack_top: u64) {
    unsafe { TSS.rsp0 = stack_top; }
}
