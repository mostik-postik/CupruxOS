//! Virtual Memory Manager
//!
//! Управляет виртуальными адресными пространствами задач.
//! Manages virtual address spaces for tasks.
//!
//! Каждая задача имеет своё AddressSpace:
//! Each task has its own AddressSpace:
//!
//!   ┌─────────────────────────────────┐
//!   │  0xFFFF_8000_0000_0000          │
//!   │       Kernel space (shared)     │  ← одинаков у всех задач
//!   │  0xFFFF_FFFF_FFFF_FFFF          │     same for all tasks
//!   ├─────────────────────────────────┤
//!   │        ... gap ...              │
//!   ├─────────────────────────────────┤
//!   │  0x0000_0000_0000_0000          │
//!   │       User space                │  ← уникален для каждой задачи
//!   │  0x0000_7FFF_FFFF_FFFF          │     unique per task
//!   └─────────────────────────────────┘

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use bitflags::bitflags;
use super::pmm::{self, PhysAddr, PAGE_SIZE};

// ── Виртуальный адрес / Virtual address ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }
    pub const fn as_u64(self)   -> u64  { self.0 }
    pub const fn as_usize(self) -> usize { self.0 as usize }
    pub const fn as_ptr<T>(self) -> *const T { self.0 as *const T }
    pub const fn as_mut_ptr<T>(self) -> *mut T { self.0 as *mut T }
}

// ── Флаги страниц / Page flags ────────────────────────────────────────────────

bitflags! {
    /// Флаги для маппинга страниц.
    /// Flags for page mapping.
    #[derive(Clone, Copy, Debug)]
    pub struct PageFlags: u64 {
        /// Страница присутствует в памяти / Page is present in memory
        const PRESENT   = 1 << 0;
        /// Страница доступна для записи / Page is writable
        const WRITABLE  = 1 << 1;
        /// Страница доступна из userspace (ring 3) / Page accessible from userspace
        const USER      = 1 << 2;
        /// Сквозная запись (write-through) / Write-through caching
        const WRITE_THROUGH = 1 << 3;
        /// Отключить кэш / Disable cache
        const NO_CACHE  = 1 << 4;
        /// Глобальная страница (не сбрасывается в TLB при смене CR3)
        /// Global page (not flushed from TLB on CR3 switch)
        const GLOBAL    = 1 << 8;
        /// Запрет выполнения (NX бит) / No-execute (NX bit)
        const NO_EXEC   = 1 << 63;

        // Комбинации / Combinations
        /// Страница ядра только для чтения / Kernel read-only page
        const KERNEL_RO = Self::PRESENT.bits() | Self::GLOBAL.bits() | Self::NO_EXEC.bits();
        /// Страница ядра для чтения и записи / Kernel read-write page
        const KERNEL_RW = Self::PRESENT.bits() | Self::WRITABLE.bits()
                        | Self::GLOBAL.bits()  | Self::NO_EXEC.bits();
        /// Страница ядра исполняемая / Kernel executable page
        const KERNEL_EX = Self::PRESENT.bits() | Self::GLOBAL.bits();
        /// Страница пользователя для чтения/записи / User read-write page
        const USER_RW   = Self::PRESENT.bits() | Self::WRITABLE.bits() | Self::USER.bits()
                        | Self::NO_EXEC.bits();
        /// Страница пользователя исполняемая / User executable page
        const USER_EX   = Self::PRESENT.bits() | Self::USER.bits();
    }
}

// ── Тип региона / VMA kind ────────────────────────────────────────────────────

/// Тип виртуального региона памяти.
/// Type of virtual memory area.
#[derive(Debug)]
pub enum VmaKind {
    /// Анонимная память — стек, куча / Anonymous memory — stack, heap
    Anonymous,
    /// Shared Memory через MemoryCap / Shared Memory via MemoryCap
    Shared(PhysAddr),
    /// Регион ядра / Kernel region
    Kernel,
}

// ── VMA — регион виртуальной памяти / Virtual Memory Area ────────────────────

/// Описывает один непрерывный регион виртуальной памяти.
/// Describes one contiguous virtual memory region.
#[derive(Debug)]
pub struct Vma {
    pub start: VirtAddr,
    pub end:   VirtAddr,
    pub flags: PageFlags,
    pub kind:  VmaKind,
}

impl Vma {
    pub fn size(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }

    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr.as_u64() >= self.start.as_u64()
        && addr.as_u64() <  self.end.as_u64()
    }
}

// ── Page Table (x86_64, 4-level) ─────────────────────────────────────────────

/// Одна запись в таблице страниц (8 байт).
/// One page table entry (8 bytes).
#[derive(Clone, Copy)]
#[repr(transparent)]
struct PageTableEntry(u64);

impl PageTableEntry {
    fn new(phys: PhysAddr, flags: PageFlags) -> Self {
        // Физический адрес выровнен по 4KB — биты 0–11 = флаги
        // Physical address aligned to 4KB — bits 0–11 = flags
        Self((phys.as_u64() & !0xFFF) | flags.bits())
    }

    fn is_present(self) -> bool {
        self.0 & PageFlags::PRESENT.bits() != 0
    }

    fn phys_addr(self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000F_FFFF_FFFF_F000)
    }

    fn flags(self) -> PageFlags {
        PageFlags::from_bits_truncate(self.0)
    }
}

/// Таблица страниц — 512 записей по 8 байт = 4KB.
/// Page table — 512 entries × 8 bytes = 4KB.
#[repr(C, align(4096))]
struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    fn zero(&mut self) {
        for e in self.entries.iter_mut() {
            *e = PageTableEntry(0);
        }
    }
}

// ── Адресное пространство / Address Space ────────────────────────────────────

/// Адресное пространство одной задачи.
/// Address space for one task.
pub struct AddressSpace {
    /// Физический адрес PML4 (корень page table) / Physical address of PML4 (page table root)
    pub pml4: PhysAddr,

    /// Список VMA регионов / List of VMA regions
    /// Используем фиксированный массив пока нет кучи / Fixed array until heap is ready
    vmas:     [Option<Vma>; 64],
    vma_count: usize,
}

impl AddressSpace {
    /// Создать новое пустое адресное пространство.
    /// Create new empty address space.
    pub fn new() -> Option<Self> {
        // Выделить страницу для PML4
        // Allocate page for PML4
        let pml4_phys = pmm::alloc_page()?;

        // Обнулить PML4
        // Zero PML4
        unsafe {
            let pml4 = phys_to_virt(pml4_phys).as_mut_ptr::<PageTable>();
            (*pml4).zero();
        }

        Some(Self {
            pml4:      pml4_phys,
            vmas:      core::array::from_fn(|_| None),
            vma_count: 0,
        })
    }

    /// Замаппировать виртуальный адрес на физический.
    /// Map virtual address to physical address.
    pub fn map(&mut self, virt: VirtAddr, phys: PhysAddr, flags: PageFlags) {
        unsafe { map_page(self.pml4, virt, phys, flags); }
    }

    /// Убрать маппинг / Unmap virtual address.
    pub fn unmap(&mut self, virt: VirtAddr) {
        unsafe { unmap_page(self.pml4, virt); }
    }

    /// Транслировать виртуальный адрес в физический.
    /// Translate virtual address to physical.
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        unsafe { translate_addr(self.pml4, virt) }
    }

    /// Загрузить это адресное пространство в CR3.
    /// Load this address space into CR3.
    pub fn activate(&self) {
        unsafe {
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) self.pml4.as_u64(),
                options(nostack)
            );
        }
    }

    /// Добавить VMA регион / Add VMA region.
    pub fn add_vma(&mut self, vma: Vma) -> bool {
        if self.vma_count >= self.vmas.len() { return false; }
        self.vmas[self.vma_count] = Some(vma);
        self.vma_count += 1;
        true
    }

    /// Найти VMA содержащий адрес / Find VMA containing address.
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        self.vmas[..self.vma_count]
            .iter()
            .filter_map(|v| v.as_ref())
            .find(|vma| vma.contains(addr))
    }

    /// Маппировать анонимный регион (lazy — страницы выделятся при page fault).
    /// Map anonymous region (lazy — pages allocated on page fault).
    pub fn map_anonymous(&mut self, start: VirtAddr, size: u64, flags: PageFlags) -> bool {
        let end = VirtAddr::new(start.as_u64() + size);
        self.add_vma(Vma { start, end, flags, kind: VmaKind::Anonymous })
        // Реальные страницы выделятся в handle_page_fault()
        // Real pages allocated in handle_page_fault()
    }

    /// Маппировать shared memory регион / Map shared memory region.
    pub fn map_shared(&mut self, start: VirtAddr, phys: PhysAddr,
                      size: u64, flags: PageFlags) -> bool {
        // Сразу маппируем физические страницы — не ленивый
        // Immediately map physical pages — not lazy
        let mut offset = 0u64;
        while offset < size {
            self.map(
                VirtAddr::new(start.as_u64() + offset),
                PhysAddr::new(phys.as_u64()  + offset),
                flags,
            );
            offset += PAGE_SIZE as u64;
        }
        let end = VirtAddr::new(start.as_u64() + size);
        self.add_vma(Vma {
            start, end, flags,
            kind: VmaKind::Shared(phys),
        })
    }
}

// ── Page Fault handler ────────────────────────────────────────────────────────

/// Обработать Page Fault — вызывается из IDT обработчика #PF.
/// Handle Page Fault — called from IDT #PF handler.
///
/// Возвращает true если fault обработан, false → SIGSEGV.
/// Returns true if fault handled, false → SIGSEGV.
pub fn handle_page_fault(
    space: &mut AddressSpace,
    fault_addr: VirtAddr,
    error: u64,
) -> bool {
    let is_write = error & 0x2 != 0;

    // Найти VMA для этого адреса
    // Find VMA for this address
    let vma = match space.find_vma(fault_addr) {
        Some(v) => v,
        None    => return false, // нет VMA → segfault
    };

    // Проверить права / Check rights
    if is_write && !vma.flags.contains(PageFlags::WRITABLE) {
        return false; // запись в read-only → segfault
    }

    let flags = vma.flags;

    // Выделить физическую страницу и замаппировать
    // Allocate physical page and map it
    match &vma.kind {
        VmaKind::Anonymous => {
            let phys = match pmm::alloc_page() {
                Some(p) => p,
                None    => return false, // OOM
            };
            // Обнулить новую страницу (безопасность!)
            // Zero new page (security!)
            unsafe {
                let ptr = phys_to_virt(phys).as_mut_ptr::<u8>();
                ptr.write_bytes(0, PAGE_SIZE);
            }
            // Выровнять адрес по границе страницы
            // Align address to page boundary
            let page_start = VirtAddr::new(
                fault_addr.as_u64() & !(PAGE_SIZE as u64 - 1)
            );
            space.map(page_start, phys, flags);
            true
        }
        VmaKind::Shared(_) | VmaKind::Kernel => {
            // Shared и Kernel маппируются сразу при создании
            // Shared and Kernel are mapped immediately on creation
            false
        }
    }
}

// ── Низкоуровневые операции с page tables / Low-level page table ops ──────────

/// Индексы в page table из виртуального адреса.
/// Page table indices from virtual address.
fn pml4_idx(addr: VirtAddr) -> usize { ((addr.as_u64() >> 39) & 0x1FF) as usize }
fn pdpt_idx(addr: VirtAddr) -> usize { ((addr.as_u64() >> 30) & 0x1FF) as usize }
fn pd_idx  (addr: VirtAddr) -> usize { ((addr.as_u64() >> 21) & 0x1FF) as usize }
fn pt_idx  (addr: VirtAddr) -> usize { ((addr.as_u64() >> 12) & 0x1FF) as usize }

/// Получить или создать следующий уровень page table.
/// Get or create next page table level.
unsafe fn get_or_create(entry: &mut PageTableEntry, flags: PageFlags) -> *mut PageTable {
    if !entry.is_present() {
        let phys = pmm::alloc_page().expect("PMM: out of memory for page table");
        let table = phys_to_virt(phys).as_mut_ptr::<PageTable>();
        (*table).zero();
        *entry = PageTableEntry::new(
            phys,
            PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER,
        );
    }
    phys_to_virt(entry.phys_addr()).as_mut_ptr::<PageTable>()
}

/// Замаппировать одну страницу / Map one page.
unsafe fn map_page(pml4_phys: PhysAddr, virt: VirtAddr,
                   phys: PhysAddr, flags: PageFlags) {
    let pml4 = phys_to_virt(pml4_phys).as_mut_ptr::<PageTable>();

    let pdpt = get_or_create(&mut (*pml4).entries[pml4_idx(virt)], flags);
    let pd   = get_or_create(&mut (*pdpt).entries[pdpt_idx(virt)], flags);
    let pt   = get_or_create(&mut (*pd  ).entries[pd_idx  (virt)], flags);

    (*pt).entries[pt_idx(virt)] = PageTableEntry::new(phys, flags);

    // Инвалидировать TLB для этого адреса
    // Invalidate TLB for this address
    core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack));
}

/// Убрать маппинг страницы / Unmap page.
unsafe fn unmap_page(pml4_phys: PhysAddr, virt: VirtAddr) {
    let pml4 = phys_to_virt(pml4_phys).as_mut_ptr::<PageTable>();
    let e0 = &mut (*pml4).entries[pml4_idx(virt)];
    if !e0.is_present() { return; }

    let pdpt = phys_to_virt(e0.phys_addr()).as_mut_ptr::<PageTable>();
    let e1 = &mut (*pdpt).entries[pdpt_idx(virt)];
    if !e1.is_present() { return; }

    let pd = phys_to_virt(e1.phys_addr()).as_mut_ptr::<PageTable>();
    let e2 = &mut (*pd).entries[pd_idx(virt)];
    if !e2.is_present() { return; }

    let pt = phys_to_virt(e2.phys_addr()).as_mut_ptr::<PageTable>();
    (*pt).entries[pt_idx(virt)] = PageTableEntry(0);

    core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack));
}

/// Транслировать виртуальный адрес / Translate virtual address.
unsafe fn translate_addr(pml4_phys: PhysAddr, virt: VirtAddr) -> Option<PhysAddr> {
    let pml4 = phys_to_virt(pml4_phys).as_ptr::<PageTable>();
    let e0 = (*pml4).entries[pml4_idx(virt)];
    if !e0.is_present() { return None; }

    let pdpt = phys_to_virt(e0.phys_addr()).as_ptr::<PageTable>();
    let e1 = (*pdpt).entries[pdpt_idx(virt)];
    if !e1.is_present() { return None; }

    let pd = phys_to_virt(e1.phys_addr()).as_ptr::<PageTable>();
    let e2 = (*pd).entries[pd_idx(virt)];
    if !e2.is_present() { return None; }

    let pt = phys_to_virt(e2.phys_addr()).as_ptr::<PageTable>();
    let e3 = (*pt).entries[pt_idx(virt)];
    if !e3.is_present() { return None; }

    let page_phys = e3.phys_addr();
    let offset    = virt.as_u64() & 0xFFF;
    Some(PhysAddr::new(page_phys.as_u64() + offset))
}

// ── Физ. адрес → виртуальный (прямой маппинг ядра) ───────────────────────────
// Physical → virtual (kernel direct map)

/// Смещение прямого маппинга ядра / Kernel direct map offset.
/// Limine маппирует всю физическую память по этому смещению.
/// Limine maps all physical memory at this offset.
pub const PHYSICAL_MAP_OFFSET: u64 = 0xFFFF_8000_0000_0000;

/// Конвертировать физический адрес в виртуальный (через прямой маппинг).
/// Convert physical address to virtual (via direct map).
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys.as_u64() + PHYSICAL_MAP_OFFSET)
}

/// Конвертировать виртуальный адрес обратно в физический.
/// Convert virtual address back to physical.
pub fn virt_to_phys(virt: VirtAddr) -> PhysAddr {
    PhysAddr::new(virt.as_u64() - PHYSICAL_MAP_OFFSET)
}

// ── Глобальное адресное пространство ядра / Kernel address space ──────────────

static KERNEL_SPACE: Mutex<Option<AddressSpace>> = Mutex::new(None);

/// Инициализировать VMM / Initialize VMM.
pub fn init() {
    let mut space = AddressSpace::new()
        .expect("VMM: failed to allocate PML4");

    // Маппировать ядро identity (физ. = вирт. - PHYSICAL_MAP_OFFSET)
    // Map kernel identity (phys = virt - PHYSICAL_MAP_OFFSET)
    // TODO: читать реальные адреса из linker script символов
    // TODO: read real addresses from linker script symbols
    // Пока маппируем первые 16MB как RW для ядра
    // For now map first 16MB as kernel RW
    let mut offset = 0u64;
    while offset < 16 * 1024 * 1024 {
        space.map(
            VirtAddr::new(PHYSICAL_MAP_OFFSET + offset),
            PhysAddr::new(offset),
            PageFlags::KERNEL_RW,
        );
        offset += PAGE_SIZE as u64;
    }

    space.activate();
    crate::kprintln!("[vmm] Kernel address space activated, PML4={:#x}", space.pml4.as_u64());

    *KERNEL_SPACE.lock() = Some(space);
}
