//! UART Serial driver — COM1 (0x3F8)
//!
//! Используется для отладочного вывода в QEMU.
//! Used for debug output in QEMU: -serial stdio
//!
//! Запуск / Run:
//!   qemu-system-x86_64 -serial stdio ...

use core::fmt;
use spin::Mutex;

const COM1: u16 = 0x3F8;

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port);
    val
}

/// Инициализировать COM1 на 115200 baud.
/// Initialize COM1 at 115200 baud.
pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00); // Отключить прерывания / Disable interrupts
        outb(COM1 + 3, 0x80); // Включить DLAB (Divisor Latch)
        outb(COM1 + 0, 0x01); // Делитель 1 → 115200 baud
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03); // 8 бит, нет чётности, 1 стоп-бит
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear, 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQ включён, RTS/DSR
    }
}

/// Ждать пока буфер передачи свободен и отправить байт.
/// Wait for transmit buffer empty and send byte.
fn send_byte(byte: u8) {
    unsafe {
        // Ждём пока регистр пуст (бит 5 LSR) / Wait until THR empty (LSR bit 5)
        while inb(COM1 + 5) & 0x20 == 0 {}
        outb(COM1, byte);
    }
}

/// Отправить строку в COM1.
/// Send string to COM1.
pub fn print(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' {
            send_byte(b'\r'); // Windows-совместимость / Windows compat
        }
        send_byte(byte);
    }
}

// ── fmt::Write для использования с format_args! ───────────────────────────────

struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s);
        Ok(())
    }
}

static UART_LOCK: Mutex<UartWriter> = Mutex::new(UartWriter);

/// Внутренняя функция для макроса kprint!
/// Internal function for kprint! macro
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    UART_LOCK.lock().write_fmt(args).ok();
}
