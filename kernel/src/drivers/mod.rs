//! Kernel-space drivers
//!
//! Только минимально необходимые драйверы в ядре:
//!   - UART/Serial  — отладочный вывод / debug output
//!   - Framebuffer  — вывод на экран   / screen output
//!   - PCI          — обнаружение устройств / device discovery
//!
//! Все остальные драйверы — в userspace через Driver Manager.
//! All other drivers live in userspace via Driver Manager.

// TODO: Этап 2 — UART + Framebuffer для отладки
// TODO: Phase 2 — UART + Framebuffer for debug
