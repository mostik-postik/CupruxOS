//! VFS Server — файловая система в userspace / filesystem in userspace
//!
//! Принимает IPC запросы от приложений и делегирует конкретной ФС.
//! Accepts IPC requests from apps and delegates to concrete FS driver.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // TODO: Этап 8 — реализация VFS сервера
    // TODO: Phase 8 — VFS server implementation
    loop { core::hint::spin_loop(); }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}
