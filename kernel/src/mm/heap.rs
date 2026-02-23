//! Kernel Heap — Slab Allocator

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use spin::Mutex;
use super::pmm::{self, PAGE_SIZE};
use super::vmm::phys_to_virt;

const SLAB_SIZES: [usize; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const NUM_SLABS:  usize = SLAB_SIZES.len();

struct FreeNode {
    next: Option<NonNull<FreeNode>>,
}

// NonNull не Send по умолчанию — оборачиваем
// NonNull is not Send by default — wrap it
struct FreeList(Option<NonNull<FreeNode>>);
unsafe impl Send for FreeList {}

struct SlabCache {
    obj_size: usize,
    free:     FreeList,
}

impl SlabCache {
    const fn new(obj_size: usize) -> Self {
        Self { obj_size, free: FreeList(None) }
    }

    fn grow(&mut self) {
        let phys = match pmm::alloc_page() { Some(p) => p, None => return };
        let virt  = phys_to_virt(phys);
        let start = virt.as_u64() as usize;
        let count = PAGE_SIZE / self.obj_size;

        for i in (0..count).rev() {
            let ptr = (start + i * self.obj_size) as *mut FreeNode;
            unsafe {
                (*ptr).next = self.free.0;
                self.free.0 = NonNull::new(ptr);
            }
        }
    }

    fn alloc(&mut self) -> Option<*mut u8> {
        if self.free.0.is_none() { self.grow(); }
        let node = self.free.0?;
        unsafe { self.free.0 = (*node.as_ptr()).next; }
        Some(node.as_ptr() as *mut u8)
    }

    fn free(&mut self, ptr: *mut u8) {
        let node = ptr as *mut FreeNode;
        unsafe {
            (*node).next = self.free.0;
            self.free.0  = NonNull::new(node);
        }
    }
}

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

    fn slab_index(size: usize) -> Option<usize> {
        SLAB_SIZES.iter().position(|&s| s >= size)
    }
}

unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());
        match Self::slab_index(size) {
            Some(idx) => self.slabs[idx].lock().alloc().unwrap_or(core::ptr::null_mut()),
            None => {
                let order = ((size + PAGE_SIZE - 1) / PAGE_SIZE)
                    .next_power_of_two().trailing_zeros() as usize;
                match pmm::alloc_pages(order) {
                    Some(phys) => phys_to_virt(phys).as_u64() as *mut u8,
                    None       => core::ptr::null_mut(),
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align());
        match Self::slab_index(size) {
            Some(idx) => self.slabs[idx].lock().free(ptr),
            None => {
                let virt = super::vmm::VirtAddr::new(ptr as u64);
                let phys = super::vmm::virt_to_phys(virt);
                let order = ((size + PAGE_SIZE - 1) / PAGE_SIZE)
                    .next_power_of_two().trailing_zeros() as usize;
                pmm::free_pages(phys, order);
            }
        }
    }
}

#[global_allocator]
static HEAP: KernelHeap = KernelHeap::new();

pub fn init() {
    for slab in HEAP.slabs.iter() { slab.lock().grow(); }
    crate::kprintln!("[heap] Slab allocator ready ({} caches)", NUM_SLABS);
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Kernel OOM: size={} align={}", layout.size(), layout.align());
}
