//! CupruxOS Kernel — точка входа / entry point

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![deny(unsafe_op_in_unsafe_fn)]

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
    kprintln!("[mm] PMM OK");

    // 3. Virtual Memory Manager
    kprintln!("[mm] Initializing VMM...");
    mm::vmm::init();
    kprintln!("[mm] VMM OK");

    // 4. Kernel Heap
    kprintln!("[mm] Initializing heap (Slab)...");
    mm::heap::init();
    kprintln!("[mm] Heap OK");

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

    // 8. Первый userspace процесс
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
