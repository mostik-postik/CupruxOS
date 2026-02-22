//! Physical Memory Manager — Buddy Allocator
//!
//! Получает карту памяти от Limine и отслеживает свободные страницы.
//! Gets the memory map from Limine and tracks free physical pages.
//!
//! Алгоритм: Buddy System
//!   - order 0 = 4KB  (1 страница  / 1 page)
//!   - order 1 = 8KB  (2 страницы  / 2 pages)
//!   - ...
//!   - order N = 4KB * 2^N
//!
//! При аллокации: ищем свободный блок нужного order.
//! При освобождении: сливаем с соседом (buddy merging).

// TODO: Этап 3 — реализация Buddy Allocator
// TODO: Phase 3 — Buddy Allocator implementation

pub const PAGE_SIZE: usize = 4096;
pub const MAX_ORDER: usize = 11; // до 8MB блоков / up to 8MB blocks

pub fn init() {
    // stub — будет читать Limine memory map
    // stub — will read Limine memory map
}

/// Выделить 2^order физических страниц.
/// Allocate 2^order physical pages.
pub fn alloc(_order: usize) -> Option<super::PhysAddr> {
    None // TODO
}

/// Освободить страницы (с buddy merging).
/// Free pages (with buddy merging).
pub fn free(_addr: super::PhysAddr, _order: usize) {
    // TODO
}
