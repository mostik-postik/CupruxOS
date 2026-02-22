//! Kernel-space drivers
//!
//! Минимально необходимые для отладки / Minimum required for debugging:
//!   - UART/Serial  — отладочный вывод в терминал QEMU
//!   - Framebuffer  — вывод на экран (TODO: Этап 2)

pub mod uart;

/// Вывести строку в UART (для отладки).
/// Print string to UART (for debugging).
pub fn print(s: &str) {
    uart::print(s);
}

/// Макрос для отладочного вывода.
/// Debug print macro.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::drivers::uart::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! kprintln {
    ()           => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}
