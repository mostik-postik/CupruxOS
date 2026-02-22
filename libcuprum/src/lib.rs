//! libcuprum — CupruxOS userspace library
//!
//! Тонкая обёртка над syscall'ами + удобный IPC API.
//! Thin wrapper over syscalls + convenient IPC API.
//!
//! Использование / Usage:
//!   use libcuprum::ipc;
//!   let reply = ipc::call(port_cap, &msg)?;

#![no_std]

pub mod ipc;
pub mod cap;
pub mod mem;
pub mod task;
pub mod time;

/// Ошибки syscall / Syscall errors
#[derive(Debug)]
pub enum Error {
    InvalidCap,
    NoPermission,
    InvalidArg,
    NoMemory,
    NotFound,
    Unknown(isize),
}

pub type Result<T> = core::result::Result<T, Error>;
