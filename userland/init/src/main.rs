//! CupruxOS Init — первый userspace процесс / first userspace process
//!
//! Запускает все системные серверы и передаёт управление shell/display.
//! Launches all system servers and hands off to shell/display.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // TODO: запустить VFS сервер, Driver Manager, Network стек
    // TODO: launch VFS server, Driver Manager, Network stack
    loop { core::hint::spin_loop(); }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}
