//! CupruxOS Kernel — точка входа / entry point

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![deny(unsafe_op_in_unsafe_fn)]

// Подключить стандартный alloc крейт (Box, Vec, Arc, ...)
// Connect standard alloc crate (Box, Vec, Arc, ...)
extern crate alloc;

use core::panic::PanicInfo;

mod arch;
mod mm;
mod sched;
mod ipc;
mod vfs;
mod drivers;
mod syscall;

/// Точка входа ядра — вызывается загрузчиком Limine.
/// Kernel entry point — called by the Limine bootloader.
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // 0. UART — первым делом / first of all
    drivers::uart::init();
    kprintln!("CupruxOS booting...");

    // 1. GDT + IDT
    kprintln!("[arch] Initializing GDT + IDT...");
    arch::init();
    kprintln!("[arch] OK — interrupts enabled");

    // 2. Physical Memory Manager
    kprintln!("[mm] Initializing PMM (Buddy)...");
    mm::pmm::init();

    // 3. Virtual Memory Manager
    kprintln!("[mm] Initializing VMM...");
    mm::vmm::init();

    // 4. Kernel Heap — после этого работают Box<T>, Vec<T>!
    //    Kernel Heap — after this Box<T>, Vec<T> work!
    kprintln!("[mm] Initializing heap (Slab)...");
    mm::heap::init();

    // Тест heap — убедиться что всё работает
    // Heap test — make sure everything works
    {
        use alloc::vec::Vec;
        use alloc::boxed::Box;
        let mut v: Vec<u32> = Vec::new();
        v.push(1); v.push(2); v.push(3);
        let b = Box::new(42u64);
        kprintln!("[mm] Heap test OK: vec={:?}, box={}", v, b);
    }

    // 5. IPC + Capability
    kprintln!("[ipc] Initializing IPC + Capability...");
    ipc::init();
    kprintln!("[ipc] OK");

    // 6. Scheduler
    kprintln!("[sched] Initializing scheduler (MLFQ)...");
    sched::init();
    kprintln!("[sched] OK");

    // 7. Syscall interface
    kprintln!("[syscall] Installing handler...");
    syscall::init();
    kprintln!("[syscall] OK");

    kprintln!("");
    kprintln!("  ██████╗██╗   ██╗██████╗ ██████╗ ██╗   ██╗██╗  ██╗ ██████╗ ███████╗");
    kprintln!(" ██╔════╝██║   ██║██╔══██╗██╔══██╗██║   ██║╚██╗██╔╝██╔═══██╗██╔════╝");
    kprintln!(" ██║     ██║   ██║██████╔╝██████╔╝██║   ██║ ╚███╔╝ ██║   ██║███████╗");
    kprintln!(" ╚██████╗╚██████╔╝██║     ██║  ██║╚██████╔╝██╔╝ ██╗╚██████╔╝███████║");
    kprintln!("  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝");
    kprintln!("");
    kprintln!("  Kernel ready. Launching init...");
    kprintln!("");

    // 8. Первый userspace процесс / First userspace process
    sched::spawn_init();
    sched::start();
}

/// Panic handler — выводим в UART и halt.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("\n[KERNEL PANIC] {}", info);
    loop {
        core::hint::spin_loop();
    }
}
