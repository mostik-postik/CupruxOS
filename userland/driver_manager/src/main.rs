//! Driver Manager — управление драйверами в userspace
//! Driver Manager — userspace driver management

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop { core::hint::spin_loop(); }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}
