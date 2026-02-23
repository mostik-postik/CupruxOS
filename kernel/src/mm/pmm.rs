//! Physical Memory Manager — Buddy Allocator
//!
//! Получает карту памяти от Limine и управляет физическими страницами.
//! Gets memory map from Limine and manages physical pages.
//!
//! Алгоритм Buddy System / Buddy System algorithm:
//!
//!   order 0 →    4 KB  (1 страница  / 1 page)
//!   order 1 →    8 KB  (2 страницы  / 2 pages)
//!   order 2 →   16 KB
//!   ...
//!   order 10 → 4096 KB (1024 страниц / 1024 pages)
//!
//! Аллокация: ищем свободный блок нужного order, если нет —
//! берём больший и делим пополам (splitting).
//!
//! Освобождение: освобождаем блок и сливаем с соседом если
//! тот тоже свободен (merging/coalescing).

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

// ── Константы / Constants ─────────────────────────────────────────────────────

pub const PAGE_SIZE:  usize = 4096;       // 4 KB
pub const MAX_ORDER:  usize = 11;         // до / up to 4096 * 2^10 = 4MB
pub const MAX_PAGES:  usize = 1024 * 1024; // поддерживаем до 4GB / support up to 4GB

// ── Физический адрес / Physical address ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }
    pub const fn as_u64(self)   -> u64  { self.0 }
    pub const fn as_usize(self) -> usize { self.0 as usize }

    /// Номер страницы / Page frame number
    pub const fn pfn(self) -> usize { self.0 as usize / PAGE_SIZE }
}

// ── Bitmap — битовая карта свободных страниц ──────────────────────────────────

/// Простая битовая карта для отслеживания свободных блоков каждого order.
/// Simple bitmap for tracking free blocks per order.
struct Bitmap {
    bits: [u64; MAX_PAGES / 64],
    len:  usize, // сколько бит реально используется
}

impl Bitmap {
    const fn new() -> Self {
        Self { bits: [0; MAX_PAGES / 64], len: 0 }
    }

    fn get(&self, idx: usize) -> bool {
        self.bits[idx / 64] & (1 << (idx % 64)) != 0
    }

    fn set(&mut self, idx: usize, val: bool) {
        if val {
            self.bits[idx / 64] |=   1 << (idx % 64);
        } else {
            self.bits[idx / 64] &= !(1 << (idx % 64));
        }
    }

    /// Найти первый свободный блок / Find first free block
    fn find_free(&self) -> Option<usize> {
        for (i, &word) in self.bits.iter().enumerate() {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let idx = i * 64 + bit;
                if idx < self.len { return Some(idx); }
            }
        }
        None
    }
}

// ── Buddy Allocator ───────────────────────────────────────────────────────────

/// Buddy Allocator — сердце PMM.
/// Buddy Allocator — the heart of PMM.
struct BuddyAllocator {
    /// Битовые карты свободных блоков для каждого order.
    /// Free block bitmaps for each order.
    /// free[n][i] = 1 означает что блок i*2^n*PAGE_SIZE свободен.
    /// free[n][i] = 1 means block at i*2^n*PAGE_SIZE is free.
    free:        [Bitmap; MAX_ORDER],

    /// Общее число страниц / Total page count
    total_pages: usize,

    /// Число свободных страниц / Free page count
    free_pages:  usize,

    /// Начало физической памяти (обычно 0x100000 / 1MB на x86)
    /// Start of physical memory (usually 0x100000 / 1MB on x86)
    mem_start:   u64,
}

impl BuddyAllocator {
    const fn new() -> Self {
        // Не можем использовать [Bitmap::new(); N] — Bitmap не Copy
        // Cannot use [Bitmap::new(); N] — Bitmap is not Copy
        Self {
            free: [
                Bitmap::new(), Bitmap::new(), Bitmap::new(),
                Bitmap::new(), Bitmap::new(), Bitmap::new(),
                Bitmap::new(), Bitmap::new(), Bitmap::new(),
                Bitmap::new(), Bitmap::new(),
            ],
            total_pages: 0,
            free_pages:  0,
            mem_start:   0,
        }
    }

    /// Добавить свободный регион памяти (от Limine).
    /// Add free memory region (from Limine).
    fn add_region(&mut self, start: u64, size: u64) {
        // Выровнять начало вверх, конец вниз по PAGE_SIZE
        // Align start up, end down to PAGE_SIZE
        let start = align_up(start, PAGE_SIZE as u64);
        let end   = align_down(start + size, PAGE_SIZE as u64);

        if start >= end { return; }

        if self.mem_start == 0 { self.mem_start = start; }

        // Добавляем страницы по одной в order 0
        // Add pages one by one at order 0
        let mut addr = start;
        while addr + PAGE_SIZE as u64 <= end {
            let pfn = ((addr - self.mem_start) / PAGE_SIZE as u64) as usize;
            self.free[0].set(pfn, true);
            self.free[0].len = self.free[0].len.max(pfn + 1);
            self.free_pages  += 1;
            self.total_pages += 1;
            addr += PAGE_SIZE as u64;
        }

        // Попробовать слить соседние блоки в блоки большего order
        // Try to merge adjacent blocks into higher order blocks
        self.merge_all();
    }

    /// Слить все возможные блоки снизу вверх.
    /// Merge all possible blocks bottom-up.
    fn merge_all(&mut self) {
        for order in 0..MAX_ORDER - 1 {
            let block_pages = 1 << order;
            let mut i = 0;
            while i + 1 < self.free[order].len {
                if self.free[order].get(i) && self.free[order].get(i + 1)
                   && i % 2 == 0
                {
                    // Оба buddy свободны — сливаем
                    // Both buddies free — merge
                    self.free[order].set(i,     false);
                    self.free[order].set(i + 1, false);
                    let parent = i / 2;
                    self.free[order + 1].set(parent, true);
                    self.free[order + 1].len =
                        self.free[order + 1].len.max(parent + 1);
                }
                i += 2;
                let _ = block_pages; // подавить предупреждение / suppress warning
            }
        }
    }

    /// Выделить 2^order страниц / Allocate 2^order pages.
    ///
    /// Возвращает физический адрес начала блока.
    /// Returns physical address of block start.
    fn alloc(&mut self, order: usize) -> Option<PhysAddr> {
        // Ищем свободный блок начиная с нужного order и выше
        // Search for free block starting at requested order and above
        let found_order = (order..MAX_ORDER)
            .find(|&o| self.free[o].find_free().is_some())?;

        // Берём блок
        let idx = self.free[found_order].find_free().unwrap();
        self.free[found_order].set(idx, false);

        // Разбиваем (split) до нужного order
        // Split down to requested order
        let mut current_order = found_order;
        let mut current_idx   = idx;

        while current_order > order {
            current_order -= 1;
            // Левый buddy — занят нами, правый — свободен
            // Left buddy — ours, right buddy — free
            let left  = current_idx * 2;
            let right = left + 1;
            self.free[current_order].set(right, true);
            self.free[current_order].len =
                self.free[current_order].len.max(right + 1);
            current_idx = left;
        }

        // Вычислить физический адрес
        // Calculate physical address
        let pages_offset = current_idx * (1 << order);
        let phys = self.mem_start + (pages_offset * PAGE_SIZE) as u64;

        self.free_pages -= 1 << order;

        Some(PhysAddr::new(phys))
    }

    /// Освободить блок / Free block.
    fn free(&mut self, addr: PhysAddr, order: usize) {
        let pages_offset = ((addr.as_u64() - self.mem_start) / PAGE_SIZE as u64)
            as usize;
        let mut idx   = pages_offset / (1 << order);
        let mut order = order;

        self.free[order].set(idx, true);
        self.free_pages += 1 << order;

        // Попробовать слить с buddy / Try to merge with buddy
        while order < MAX_ORDER - 1 {
            let buddy = idx ^ 1; // XOR 1 — получить индекс buddy
            if buddy < self.free[order].len && self.free[order].get(buddy) {
                // Buddy свободен — сливаем / Buddy is free — merge
                self.free[order].set(idx,   false);
                self.free[order].set(buddy, false);
                idx   = idx / 2;
                order += 1;
                self.free[order].set(idx, true);
                self.free[order].len = self.free[order].len.max(idx + 1);
            } else {
                break;
            }
        }
    }
}

// ── Выравнивание / Alignment helpers ─────────────────────────────────────────

const fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

const fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}

// ── Глобальный PMM / Global PMM ───────────────────────────────────────────────

static PMM: Mutex<BuddyAllocator> = Mutex::new(BuddyAllocator::new());

/// Статистика памяти / Memory statistics
static TOTAL_BYTES: AtomicU64 = AtomicU64::new(0);
static FREE_BYTES:  AtomicU64 = AtomicU64::new(0);

/// Инициализировать PMM — вызывается из kernel_main.
/// Initialize PMM — called from kernel_main.
///
/// Читает карту памяти от Limine и регистрирует свободные регионы.
/// Reads memory map from Limine and registers free regions.
pub fn init() {
    // TODO: получить реальную карту памяти от Limine
    // TODO: get real memory map from Limine
    //
    // Когда подключим Limine protocol, будет примерно так:
    // When we connect Limine protocol, it will look like:
    //
    // let mmap = MMAP_REQUEST.get_response().unwrap();
    // for entry in mmap.entries() {
    //     if entry.entry_type == EntryType::USABLE {
    //         pmm.add_region(entry.base, entry.length);
    //     }
    // }
    //
    // Пока — заглушка с 16MB тестовой памяти для разработки
    // For now — stub with 16MB test memory for development
    let mut pmm = PMM.lock();
    pmm.add_region(0x100000, 16 * 1024 * 1024); // 1MB..17MB

    TOTAL_BYTES.store(pmm.total_pages as u64 * PAGE_SIZE as u64, Ordering::Relaxed);
    FREE_BYTES.store(pmm.free_pages   as u64 * PAGE_SIZE as u64, Ordering::Relaxed);

    crate::kprintln!(
        "[pmm] Total: {} MB, Free: {} MB",
        TOTAL_BYTES.load(Ordering::Relaxed) / 1024 / 1024,
        FREE_BYTES.load(Ordering::Relaxed)  / 1024 / 1024,
    );
}

// ── Публичный API / Public API ────────────────────────────────────────────────

/// Выделить одну страницу (4KB) / Allocate one page (4KB).
pub fn alloc_page() -> Option<PhysAddr> {
    alloc_pages(0)
}

/// Выделить 2^order страниц / Allocate 2^order pages.
pub fn alloc_pages(order: usize) -> Option<PhysAddr> {
    let addr = PMM.lock().alloc(order)?;
    FREE_BYTES.fetch_sub((PAGE_SIZE << order) as u64, Ordering::Relaxed);
    Some(addr)
}

/// Освободить одну страницу / Free one page.
pub fn free_page(addr: PhysAddr) {
    free_pages(addr, 0);
}

/// Освободить 2^order страниц / Free 2^order pages.
pub fn free_pages(addr: PhysAddr, order: usize) {
    PMM.lock().free(addr, order);
    FREE_BYTES.fetch_add((PAGE_SIZE << order) as u64, Ordering::Relaxed);
}

/// Статистика / Statistics
pub fn free_memory()  -> u64 { FREE_BYTES.load(Ordering::Relaxed) }
pub fn total_memory() -> u64 { TOTAL_BYTES.load(Ordering::Relaxed) }
