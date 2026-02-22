//! Scheduler — MLFQ + IPC-aware
//!
//! Multilevel Feedback Queue адаптированный под IPC события.
//! Multilevel Feedback Queue adapted for IPC events.
//!
//! Очереди / Queues:
//!   0 →  1ms — IPC wake-ups        (highest priority)
//!   1 →  5ms — interactive tasks
//!   2 → 20ms — normal tasks
//!   3 → 100ms — background tasks   (lowest priority)
//!
//! IPC пробуждение ВСЕГДА идёт в очередь 0.
//! IPC wake-up ALWAYS goes to queue 0.

// TODO: Этап 5 — реализация планировщика
// TODO: Phase 5 — Scheduler implementation

pub fn init() {
    // stub
}

pub fn spawn_init() {
    // stub — запустить первый userspace процесс
    // stub — launch first userspace process
}

pub fn start() -> ! {
    // stub — войти в цикл планировщика
    // stub — enter scheduler loop
    loop { core::hint::spin_loop(); }
}
