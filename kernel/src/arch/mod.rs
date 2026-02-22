//! HAL — Hardware Abstraction Layer
//!
//! Выбираем реализацию в зависимости от целевой архитектуры.
//! Select implementation based on target architecture.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64 as current;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64 as current;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use riscv64 as current;

/// Инициализация платформы — вызывается первой из kernel_main.
/// Platform initialization — called first from kernel_main.
pub fn init() {
    current::init();
}
