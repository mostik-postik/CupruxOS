//! IPC — Inter-Process Communication API
//!
//! Обёртки над ipc_* syscall'ами.
//! Wrappers over ipc_* syscalls.

use crate::Result;

/// Capability на порт / Port capability
#[derive(Clone, Copy)]
pub struct PortCap(pub u64);

/// Сообщение / Message (inline payload + capability slots)
pub struct Message {
    pub payload: [u8; 512],
    pub payload_len: usize,
}

/// Синхронный вызов — отправить и ждать ответа.
/// Synchronous call — send and wait for reply.
pub fn call(_port: PortCap, _msg: &Message) -> Result<Message> {
    // TODO: arch::syscall(0, ...)
    Err(crate::Error::Unknown(-1))
}

/// Асинхронная отправка — не ждать ответа.
/// Async send — don't wait for reply.
pub fn send(_port: PortCap, _msg: &Message) -> Result<()> {
    // TODO: arch::syscall(1, ...)
    Err(crate::Error::Unknown(-1))
}

/// Ждать входящего сообщения.
/// Wait for incoming message.
pub fn recv(_port: PortCap) -> Result<Message> {
    // TODO: arch::syscall(2, ...)
    Err(crate::Error::Unknown(-1))
}
