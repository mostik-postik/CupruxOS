//! Kernel Heap — Slab Allocator
//!
//! Пул заранее выделенных объектов фиксированного размера.
//! Pool of pre-allocated fixed-size objects.
//!
//! Размеры слабов / Slab sizes: 8, 16, 32, 64, 128, 256, 512 байт.
//! Большие объекты → Buddy Allocator.
//! Large objects → Buddy Allocator.
//!
//! После init() работают: Box<T>, Vec<T>, Arc<T> в ядре.
//! After init(): Box<T>, Vec<T>, Arc<T> work in kernel.

// TODO: Этап 4 — реализация Slab Allocator + глобальный аллокатор
// TODO: Phase 4 — Slab Allocator + global allocator implementation

pub fn init() {
    // stub
}
