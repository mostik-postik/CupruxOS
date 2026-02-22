//! Memory Management subsystem
//!
//! Три уровня / Three layers:
//!   pmm  — Physical Memory Manager (Buddy Allocator)
//!   vmm  — Virtual Memory Manager (Page Tables + VMA)
//!   heap — Kernel Heap (Slab Allocator)

pub mod pmm;
pub mod vmm;
pub mod heap;

/// Физический адрес / Physical address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

/// Виртуальный адрес / Virtual address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }
    pub const fn as_u64(self) -> u64   { self.0 }
}

impl VirtAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }
    pub const fn as_u64(self) -> u64   { self.0 }
}
