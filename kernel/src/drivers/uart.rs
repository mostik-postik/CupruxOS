//! UART Serial driver — COM1 (0x3F8)

use core::fmt;
use spin::Mutex;

const COM1: u16 = 0x3F8;

unsafe fn outb(port: u16, val: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") val); }
}

unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") val, in("dx") port); }
    val
}

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
}

fn send_byte(byte: u8) {
    unsafe {
        while inb(COM1 + 5) & 0x20 == 0 {}
        outb(COM1, byte);
    }
}

pub fn print(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' { send_byte(b'\r'); }
        send_byte(byte);
    }
}

struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s);
        Ok(())
    }
}

static UART_LOCK: Mutex<UartWriter> = Mutex::new(UartWriter);

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    UART_LOCK.lock().write_fmt(args).ok();
}
