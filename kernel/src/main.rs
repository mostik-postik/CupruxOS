//! CupruxOS Kernel — точка входа / entry point
//!
//! Загрузчик (Limine) передаёт управление сюда после инициализации
//! железа и передачи карты памяти.
//!
//! The bootloader (Limine) hands control here after hardware init
//! and providing the memory map.

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::panic::PanicInfo;

mod arch;    // HAL — Hardware Abstraction Layer
mod mm;      // Memory Management (PMM + VMM + Heap)
mod sched;   // Scheduler (MLFQ + IPC-aware)
mod ipc;     // IPC + Capability System
mod vfs;     // Virtual Filesystem interface
mod drivers; // Kernel-space drivers
mod syscall; // Syscall handler

/// Точка входа ядра — вызывается загрузчиком Limine.
/// Kernel entry point — called by the Limine bootloader.
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // 1. Инициализация архитектурно-зависимого кода (GDT, IDT, ...)
    //    Architecture-specific init (GDT, IDT, ...)
    arch::init();

    // 2. Инициализация физической памяти (карта от загрузчика)
    //    Physical memory init (map from bootloader)
    mm::pmm::init();

    // 3. Виртуальная память — маппинг ядра
    //    Virtual memory — map the kernel
    mm::vmm::init();

    // 4. Kernel heap — slab allocator
    mm::heap::init();

    // 5. IPC подсистема — порты и capability
    //    IPC subsystem — ports and capabilities
    ipc::init();

    // 6. Планировщик — MLFQ
    //    Scheduler — MLFQ
    sched::init();

    // 7. Syscall интерфейс
    //    Syscall interface
    syscall::init();

    // 8. Запуск первого userspace процесса (init)
    //    Launch first userspace process (init)
    sched::spawn_init();

    // Передаём управление планировщику — он уже не вернётся
    // Hand off to scheduler — this never returns
    sched::start();
}

/// Обработчик паники — halt навсегда.
/// Panic handler — halt forever.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: вывести info на экран / UART
    // TODO: print info to screen / UART
    loop {
        core::hint::spin_loop();
    }
}
