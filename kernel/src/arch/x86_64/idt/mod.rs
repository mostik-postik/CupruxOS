//! Interrupt Descriptor Table (IDT) — x86_64
//!
//! Регистрирует обработчики всех 256 прерываний/исключений.
//! Самые важные:
//!   - #PF (14) — Page Fault → наш VMM должен обработать
//!   - #GP (13) — General Protection → невалидная инструкция
//!   - IRQ0    — Timer → тик планировщика
//!
//! Registers handlers for all 256 interrupts/exceptions.
//! Most important:
//!   - #PF (14) — Page Fault → our VMM must handle
//!   - #GP (13) — General Protection → invalid instruction
//!   - IRQ0    — Timer → scheduler tick

// TODO: Этап 2 — реализация IDT
// TODO: Phase 2 — IDT implementation
pub fn init() {
    // stub
}
