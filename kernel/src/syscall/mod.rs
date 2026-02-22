//! Syscall handler — ~15 system calls
//!
//! Номера / Numbers:
//!   0  ipc_call(cap, msg)      — синхронный IPC вызов
//!   1  ipc_send(cap, msg)      — асинхронная отправка
//!   2  ipc_recv(cap)           — ждать сообщения
//!   3  ipc_reply(msg)          — ответить на вызов
//!   4  cap_create_port()       — создать порт
//!   5  cap_grant(cap, task)    — передать capability
//!   6  cap_revoke(cap)         — отозвать capability
//!   7  mem_map(cap, addr)      — замаппить регион
//!   8  mem_unmap(addr)         — размаппить
//!   9  mem_alloc(size)         — запросить анонимную память
//!   10 task_spawn(bin, caps)   — создать задачу
//!   11 task_exit(code)         — завершиться
//!   12 task_yield()            — отдать CPU
//!   13 time_now()              — текущее время (нс)
//!   14 time_sleep(ns)          — заснуть

// TODO: Этап 7 — реализация syscall handler
// TODO: Phase 7 — syscall handler implementation

pub fn init() {
    // stub — установить обработчик (syscall/svc/ecall)
    // stub — install handler (syscall/svc/ecall)
}

#[no_mangle]
pub extern "C" fn syscall_handler(
    number: usize,
    _arg0: usize,
    _arg1: usize,
    _arg2: usize,
) -> isize {
    match number {
        0..=14 => -1, // TODO: реализовать / implement
        _      => -38, // ENOSYS
    }
}
