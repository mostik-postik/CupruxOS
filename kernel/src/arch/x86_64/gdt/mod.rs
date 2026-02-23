//! Global Descriptor Table (GDT) — x86_64

use core::mem::size_of;

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
    const fn new(access: u8, gran: u8) -> Self {
        Self { limit_low: 0xFFFF, base_low: 0, base_mid: 0,
               access, granularity: gran, base_high: 0 }
    }
}

#[derive(Clone, Copy, Default)]
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
    fn from_tss(tss_addr: u64, tss_size: u64) -> Self {
        let size = tss_size - 1;
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

#[repr(C, packed)]
pub struct Tss {
    reserved0:      u32,
    pub rsp0:       u64,
    pub rsp1:       u64,
    pub rsp2:       u64,
    reserved1:      u64,
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
            reserved3: 0, iomap_base: size_of::<Tss>() as u16,
        }
    }
}

#[repr(C, packed)]
struct GdtBase {
    null:        GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code:   GdtEntry,
    user_data:   GdtEntry,
    tss:         TssEntry,
}

#[repr(C, packed)]
struct GdtDescriptor {
    size:   u16,
    offset: u64,
}

static mut KERNEL_STACK: [u8; 16 * 1024] = [0; 16 * 1024];
static mut TSS: Tss = Tss::new();
static mut GDT: GdtBase = GdtBase {
    null:        GdtEntry::null(),
    kernel_code: GdtEntry::new(0x9A, 0xA0),
    kernel_data: GdtEntry::new(0x92, 0xC0),
    user_code:   GdtEntry::new(0xFA, 0xA0),
    user_data:   GdtEntry::new(0xF2, 0xC0),
    tss: TssEntry {
        limit_low: 0, base_low: 0, base_mid: 0,
        access: 0, granularity: 0, base_high: 0,
        base_upper: 0, reserved: 0,
    },
};

pub fn init() {
    unsafe {
        // &raw const — безопасный способ получить указатель на static mut
        // &raw const — safe way to get pointer to static mut
        let stack_top = (&raw const KERNEL_STACK) as *const u8;
        let stack_top = stack_top.add(KERNEL_STACK.len()) as u64;
        TSS.rsp0 = stack_top;

        let tss_addr = (&raw const TSS) as u64;
        GDT.tss = TssEntry::from_tss(tss_addr, size_of::<Tss>() as u64);

        let descriptor = GdtDescriptor {
            size:   (size_of::<GdtBase>() - 1) as u16,
            offset: (&raw const GDT) as u64,
        };

        core::arch::asm!(
            "lgdt [{desc}]",
            "mov ax, {kdata}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            "push {kcode}",
            "lea rax, [rip + 2f]",  // 2: вместо 1: — избегаем конфликта с бинарными литералами
            "push rax",
            "retfq",
            "2:",
            "ltr {tss:x}",
            desc  = in(reg) &descriptor,
            kcode = const KERNEL_CODE as u64,
            kdata = const KERNEL_DATA,
            tss   = in(reg) TSS_SEL,
            out("rax") _,
        );
    }
}

pub fn set_kernel_stack(stack_top: u64) {
    unsafe { TSS.rsp0 = stack_top; }
}
