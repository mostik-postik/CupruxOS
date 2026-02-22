//! x86_64 platform initialization

pub mod gdt;
pub mod idt;
pub mod mm;

/// x86_64 init sequence
pub fn init() {
    gdt::init();   // Global Descriptor Table
    idt::init();   // Interrupt Descriptor Table
    mm::init();    // Page tables (identity map kernel)
}
