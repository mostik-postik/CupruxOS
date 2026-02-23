//! Kernel Heap — Slab Allocator
//!
//! Пул заранее выделенных объектов фиксированного размера.
//! Pool of pre-allocated fixed-size objects.
//!
//! Размеры слабов / Slab sizes:
//!   8, 16, 32, 64, 128, 256, 512, 1024, 2048 байт
//!
//! Объекты больше 2048 байт → напрямую через PMM (buddy).
//! Objects larger than 2048 bytes → directly via PMM (buddy).
//!
//! После init() работают:
//! After init():
//!   Box<T>, Vec<T>, Arc<T>, BTreeMap<K,V> — всё что использует alloc

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};
use spin::Mutex;
use super::pmm::{self, PhysAddr, PAGE_SIZE};
use super::vmm::{phys_to_virt, VirtAddr};

// ── Размеры слабов / Slab sizes ───────────────────────────────────────────────

const SLAB_SIZES: [usize; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const NUM_SLABS:  usize = SLAB_SIZES.len();

// ── Свободный объект в slab / Free object in slab ────────────────────────────

/// Связный список свободных объектов.
/// Linked list of free objects.
struct FreeNode {
    next: Option<NonNull<FreeNode>>,
}

// ── Slab кэш / Slab cache ─────────────────────────────────────────────────────

/// Кэш объектов одного размера.
/// Cache for objects of one size.
struct SlabCache {
    /// Размер объекта / Object size
    obj_size: usize,
    /// Список свободных объектов / Free object list
    free:     Option<NonNull<FreeNode>>,
    /// Статистика / Stats
    total:    usize,
    used:     usize,
}

impl SlabCache {
    const fn new(obj_size: usize) -> Self {
        Self { obj_size, free: None, total: 0, used: 0 }
    }

    /// Расширить slab — выделить новую страницу и нарезать на объекты.
    /// Grow slab — allocate new page and slice into objects.
    fn grow(&mut self) {
        let phys = match pmm::alloc_page() {
            Some(p) => p,
            None    => return, // OOM
        };

        // Конвертировать в виртуальный адрес
        let virt = phys_to_virt(phys);
        let start = virt.as_u64() as usize;
        let obj_size = self.obj_size;

        // Нарезать страницу на объекты и связать в список
        // Slice page into objects and link into free list
        let count = PAGE_SIZE / obj_size;
        for i in (0..count).rev() {
            let ptr = (start + i * obj_size) as *mut FreeNode;
            unsafe {
                (*ptr).next = self.free;
                self.free = NonNull::new(ptr);
            }
        }

        self.total += count;
    }

    /// Выделить объект / Allocate object.
    fn alloc(&mut self) -> Option<*mut u8> {
        if self.free.is_none() {
            self.grow();
        }

        let node = self.free?;
        unsafe {
            self.free = (*node.as_ptr()).next;
        }
        self.used += 1;
        Some(node.as_ptr() as *mut u8)
    }

    /// Освободить объект / Free object.
    fn free(&mut self, ptr: *mut u8) {
        let node = ptr as *mut FreeNode;
        unsafe {
            (*node).next = self.free;
            self.free = NonNull::new(node);
        }
        self.used -= 1;
    }
}

// ── Глобальный аллокатор / Global allocator ───────────────────────────────────

/// Heap аллокатор ядра.
/// Kernel heap allocator.
pub struct KernelHeap {
    slabs: [Mutex<SlabCache>; NUM_SLABS],
}

impl KernelHeap {
    const fn new() -> Self {
        Self {
            slabs: [
                Mutex::new(SlabCache::new(SLAB_SIZES[0])),
                Mutex::new(SlabCache::new(SLAB_SIZES[1])),
                Mutex::new(SlabCache::new(SLAB_SIZES[2])),
                Mutex::new(SlabCache::new(SLAB_SIZES[3])),
                Mutex::new(SlabCache::new(SLAB_SIZES[4])),
                Mutex::new(SlabCache::new(SLAB_SIZES[5])),
                Mutex::new(SlabCache::new(SLAB_SIZES[6])),
                Mutex::new(SlabCache::new(SLAB_SIZES[7])),
                Mutex::new(SlabCache::new(SLAB_SIZES[8])),
            ],
        }
    }

    /// Найти подходящий slab для размера / Find suitable slab for size.
    fn slab_index(size: usize) -> Option<usize> {
        SLAB_SIZES.iter().position(|&s| s >= size)
    }
}

// ── GlobalAlloc — интеграция с Rust alloc ─────────────────────────────────────

unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());

        HEAP_USED.fetch_add(size, Ordering::Relaxed);

        match Self::slab_index(size) {
            Some(idx) => {
                // Маленький объект → slab
                self.slabs[idx].lock().alloc()
                    .unwrap_or(core::ptr::null_mut())
            }
            None => {
                // Большой объект → прямо из PMM
                // Large object → directly from PMM
                let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
                let order = pages.next_power_of_two().trailing_zeros() as usize;
                match pmm::alloc_pages(order) {
                    Some(phys) => phys_to_virt(phys).as_u64() as *mut u8,
                    None       => core::ptr::null_mut(),
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align());

        HEAP_USED.fetch_sub(size, Ordering::Relaxed);

        match Self::slab_index(size) {
            Some(idx) => {
                self.slabs[idx].lock().free(ptr);
            }
            None => {
                // Вернуть в PMM / Return to PMM
                let virt = VirtAddr::new(ptr as u64);
                let phys = super::vmm::virt_to_phys(virt);
                let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
                let order = pages.next_power_of_two().trailing_zeros() as usize;
                pmm::free_pages(phys, order);
            }
        }
    }
}

// ── Регистрация глобального аллокатора / Register global allocator ────────────

#[global_allocator]
static HEAP: KernelHeap = KernelHeap::new();

static HEAP_USED: AtomicUsize = AtomicUsize::new(0);

/// Инициализировать heap / Initialize heap.
///
/// После этого вызова работают Box<T>, Vec<T>, Arc<T>.
/// After this call Box<T>, Vec<T>, Arc<T> work.
pub fn init() {
    // Прогреть каждый slab — выделить по одной странице заранее
    // Warm up each slab — pre-allocate one page per slab
    for slab in HEAP.slabs.iter() {
        slab.lock().grow();
    }

    crate::kprintln!(
        "[heap] Slab allocator ready, {} slab caches ({} sizes)",
        NUM_SLABS,
        NUM_SLABS,
    );
}

/// Сколько байт heap занято / How many heap bytes are used.
pub fn used() -> usize {
    HEAP_USED.load(Ordering::Relaxed)
}

// ── OOM handler ───────────────────────────────────────────────────────────────

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!(
        "Kernel heap OOM: size={}, align={}",
        layout.size(), layout.align()
    );
}
