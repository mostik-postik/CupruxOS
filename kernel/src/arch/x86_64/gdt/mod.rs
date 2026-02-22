//! Global Descriptor Table (GDT) — x86_64
//!
//! Сегментная модель x86 устарела в 64-bit режиме,
//! но GDT всё равно нужен для:
//!   - переключения кольца (ring 0 ↔ ring 3)
//!   - TSS (Task State Segment) — стек для прерываний
//!
//! The x86 segmentation model is legacy in 64-bit mode,
//! but GDT is still needed for:
//!   - ring switching (ring 0 ↔ ring 3)
//!   - TSS (Task State Segment) — interrupt stack

// TODO: Этап 2 — реализация GDT
// TODO: Phase 2 — GDT implementation
pub fn init() {
    // stub
}
