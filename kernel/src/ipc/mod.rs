//! IPC + Capability System
//!
//! Основные примитивы / Core primitives:
//!   Port       — очередь сообщений / message queue
//!   Capability — unforgeable токен доступа / unforgeable access token
//!   Message    — сообщение (inline + capability transfer) / message

// TODO: Этап 6 — реализация IPC
// TODO: Phase 6 — IPC implementation

/// Идентификатор порта / Port identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortId(pub u64);

/// Идентификатор capability / Capability identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapId(pub u64);

/// Идентификатор задачи / Task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub u64);

pub fn init() {
    // stub
}
