//! Virtual Memory Manager — x86_64 4-level paging

use bitflags::bitflags;
use spin::Mutex;
use super::pmm::{self, PhysAddr, PAGE_SIZE};

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

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PageFlags: u64 {
        const PRESENT      = 1 << 0;
        const WRITABLE     = 1 << 1;
        const USER         = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE     = 1 << 4;
        const GLOBAL       = 1 << 8;
        const NO_EXEC      = 1 << 63;

        const KERNEL_RO = Self::PRESENT.bits() | Self::GLOBAL.bits() | Self::NO_EXEC.bits();
        const KERNEL_RW = Self::PRESENT.bits() | Self::WRITABLE.bits()
                        | Self::GLOBAL.bits()  | Self::NO_EXEC.bits();
        const KERNEL_EX = Self::PRESENT.bits() | Self::GLOBAL.bits();
        const USER_RW   = Self::PRESENT.bits() | Self::WRITABLE.bits()
                        | Self::USER.bits()    | Self::NO_EXEC.bits();
        const USER_EX   = Self::PRESENT.bits() | Self::USER.bits();
    }
}

pub enum VmaKind {
    Anonymous,
    Shared(PhysAddr),
    Kernel,
}

pub struct Vma {
    pub start: VirtAddr,
    pub end:   VirtAddr,
    pub flags: PageFlags,
    pub kind:  VmaKind,
}

impl Vma {
    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr.as_u64() >= self.start.as_u64() && addr.as_u64() < self.end.as_u64()
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct PageTableEntry(u64);

impl PageTableEntry {
    fn new(phys: PhysAddr, flags: PageFlags) -> Self {
        Self((phys.as_u64() & !0xFFF) | flags.bits())
    }
    fn is_present(self) -> bool { self.0 & PageFlags::PRESENT.bits() != 0 }
    fn phys_addr(self)  -> PhysAddr { PhysAddr::new(self.0 & 0x000F_FFFF_FFFF_F000) }
}

#[repr(C, align(4096))]
struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    fn zero(&mut self) {
        for e in self.entries.iter_mut() { *e = PageTableEntry(0); }
    }
}

pub struct AddressSpace {
    pub pml4:  PhysAddr,
    vmas:      [Option<Vma>; 64],
    vma_count: usize,
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let pml4_phys = pmm::alloc_page()?;
        unsafe {
            let pml4 = phys_to_virt(pml4_phys).as_mut_ptr::<PageTable>();
            (*pml4).zero();
        }
        Some(Self { pml4: pml4_phys, vmas: core::array::from_fn(|_| None), vma_count: 0 })
    }

    pub fn map(&mut self, virt: VirtAddr, phys: PhysAddr, flags: PageFlags) {
        unsafe { map_page(self.pml4, virt, phys, flags); }
    }

    pub fn unmap(&mut self, virt: VirtAddr) {
        unsafe { unmap_page(self.pml4, virt); }
    }

    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        unsafe { translate_addr(self.pml4, virt) }
    }

    pub fn activate(&self) {
        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) self.pml4.as_u64(), options(nostack));
        }
    }

    pub fn add_vma(&mut self, vma: Vma) -> bool {
        if self.vma_count >= self.vmas.len() { return false; }
        self.vmas[self.vma_count] = Some(vma);
        self.vma_count += 1;
        true
    }

    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        self.vmas[..self.vma_count]
            .iter().filter_map(|v| v.as_ref())
            .find(|vma| vma.contains(addr))
    }

    pub fn map_anonymous(&mut self, start: VirtAddr, size: u64, flags: PageFlags) -> bool {
        let end = VirtAddr::new(start.as_u64() + size);
        self.add_vma(Vma { start, end, flags, kind: VmaKind::Anonymous })
    }
}

pub fn handle_page_fault(space: &mut AddressSpace, fault_addr: VirtAddr, error: u64) -> bool {
    let is_write = error & 0x2 != 0;
    let vma = match space.find_vma(fault_addr) { Some(v) => v, None => return false };
    if is_write && !vma.flags.contains(PageFlags::WRITABLE) { return false; }
    let flags = vma.flags;
    match &vma.kind {
        VmaKind::Anonymous => {
            let phys = match pmm::alloc_page() { Some(p) => p, None => return false };
            unsafe {
                let ptr = phys_to_virt(phys).as_mut_ptr::<u8>();
                ptr.write_bytes(0, PAGE_SIZE);
            }
            let page_start = VirtAddr::new(fault_addr.as_u64() & !(PAGE_SIZE as u64 - 1));
            space.map(page_start, phys, flags);
            true
        }
        _ => false,
    }
}

fn pml4_idx(addr: VirtAddr) -> usize { ((addr.as_u64() >> 39) & 0x1FF) as usize }
fn pdpt_idx(addr: VirtAddr) -> usize { ((addr.as_u64() >> 30) & 0x1FF) as usize }
fn pd_idx  (addr: VirtAddr) -> usize { ((addr.as_u64() >> 21) & 0x1FF) as usize }
fn pt_idx  (addr: VirtAddr) -> usize { ((addr.as_u64() >> 12) & 0x1FF) as usize }

unsafe fn get_or_create(entry: &mut PageTableEntry) -> *mut PageTable {
    unsafe {
        if !entry.is_present() {
            let phys = pmm::alloc_page().expect("PMM OOM for page table");
            let table = phys_to_virt(phys).as_mut_ptr::<PageTable>();
            (*table).zero();
            *entry = PageTableEntry::new(
                phys,
                PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER,
            );
        }
        phys_to_virt(entry.phys_addr()).as_mut_ptr::<PageTable>()
    }
}

unsafe fn map_page(pml4_phys: PhysAddr, virt: VirtAddr, phys: PhysAddr, flags: PageFlags) {
    unsafe {
        let pml4 = phys_to_virt(pml4_phys).as_mut_ptr::<PageTable>();
        let pdpt = get_or_create(&mut (*pml4).entries[pml4_idx(virt)]);
        let pd   = get_or_create(&mut (*pdpt).entries[pdpt_idx(virt)]);
        let pt   = get_or_create(&mut (*pd  ).entries[pd_idx  (virt)]);
        (*pt).entries[pt_idx(virt)] = PageTableEntry::new(phys, flags);
        core::arch::asm!("invlpg [{}]", in(reg) virt.as_u64(), options(nostack));
    }
}

unsafe fn unmap_page(pml4_phys: PhysAddr, virt: VirtAddr) {
    unsafe {
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
}

unsafe fn translate_addr(pml4_phys: PhysAddr, virt: VirtAddr) -> Option<PhysAddr> {
    unsafe {
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
        Some(PhysAddr::new(e3.phys_addr().as_u64() + (virt.as_u64() & 0xFFF)))
    }
}

pub const PHYSICAL_MAP_OFFSET: u64 = 0xFFFF_8000_0000_0000;

pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys.as_u64() + PHYSICAL_MAP_OFFSET)
}

pub fn virt_to_phys(virt: VirtAddr) -> PhysAddr {
    PhysAddr::new(virt.as_u64() - PHYSICAL_MAP_OFFSET)
}

static KERNEL_SPACE: Mutex<Option<AddressSpace>> = Mutex::new(None);

pub fn init() {
    let mut space = AddressSpace::new().expect("VMM: failed to allocate PML4");
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
    crate::kprintln!("[vmm] PML4={:#x}", space.pml4.as_u64());
    *KERNEL_SPACE.lock() = Some(space);
}
