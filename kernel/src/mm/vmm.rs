//! Virtual Memory Manager
//!
//! Управляет адресными пространствами задач.
//! Manages per-task virtual address spaces.
//!
//! Каждая задача имеет своё AddressSpace с набором VMA регионов.
//! Each task has its own AddressSpace with a set of VMA regions.

use bitflags::bitflags;
use super::{PhysAddr, VirtAddr};

bitflags! {
    /// Флаги страницы / Page flags
    pub struct PageFlags: u64 {
        const PRESENT   = 1 << 0;
        const WRITABLE  = 1 << 1;
        const USER      = 1 << 2;
        const NO_EXEC   = 1 << 63;
    }
}

/// Тип региона виртуальной памяти / VMA kind
pub enum VmaKind {
    /// Обычная анонимная память (стек, куча) / Anonymous (stack, heap)
    Anonymous,
    /// Shared memory — маппинг через MemoryCap / Shared via MemoryCap
    Shared(PhysAddr),
    /// Регион ядра / Kernel region
    Kernel,
}

/// Регион виртуальной памяти / Virtual Memory Area
pub struct Vma {
    pub start: VirtAddr,
    pub end:   VirtAddr,
    pub flags: PageFlags,
    pub kind:  VmaKind,
}

/// Трейт для архитектурно-зависимых page tables.
/// Trait for arch-specific page table implementations.
pub trait PageTableImpl {
    fn map(&mut self, virt: VirtAddr, phys: PhysAddr, flags: PageFlags);
    fn unmap(&mut self, virt: VirtAddr);
    fn translate(&self, virt: VirtAddr) -> Option<PhysAddr>;
    /// Загрузить таблицу (CR3 / TTBR0 / SATP)
    /// Load table (CR3 / TTBR0 / SATP)
    fn activate(&self);
}

pub fn init() {
    // stub — TODO: Этап 3 / Phase 3
}
