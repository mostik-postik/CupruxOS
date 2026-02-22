<div align="center">

```
   ██████╗██╗   ██╗██████╗ ██████╗ ██╗   ██╗██╗  ██╗ ██████╗ ███████╗
  ██╔════╝██║   ██║██╔══██╗██╔══██╗██║   ██║╚██╗██╔╝██╔═══██╗██╔════╝
  ██║     ██║   ██║██████╔╝██████╔╝██║   ██║ ╚███╔╝ ██║   ██║███████╗
  ██║     ██║   ██║██╔═══╝ ██╔══██╗██║   ██║ ██╔██╗ ██║   ██║╚════██║
  ╚██████╗╚██████╔╝██║     ██║  ██║╚██████╔╝██╔╝ ██╗╚██████╔╝███████║
   ╚═════╝ ╚═════╝ ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
```

**Cuprum (Cu) · Unix · Rust**

*A hybrid Unix-like operating system written in Rust*  
*Гибридная Unix-подобная операционная система на Rust*

---

[![Language](https://img.shields.io/badge/language-Rust-B87333?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/kernel-Hybrid-8B5E3C?style=flat-square)](#architecture)
[![Platforms](https://img.shields.io/badge/platforms-x86__64%20·%20ARM64%20·%20RISC--V-B87333?style=flat-square)](#platforms)
[![Status](https://img.shields.io/badge/status-Design%20Phase-D4956A?style=flat-square)](#roadmap)
[![License](https://img.shields.io/badge/license-MIT-8B5E3C?style=flat-square)](LICENSE)
[![AI Assisted](https://img.shields.io/badge/docs-AI%20Assisted-6e40c9?style=flat-square&logo=claude&logoColor=white)](#)

</div>

> [!NOTE]
> **RU** — Архитектура, документация и README этого проекта разработаны при помощи искусственного интеллекта ([Claude](https://claude.ai) by Anthropic). Концепция, название и все технические решения принимались человеком; ИИ использовался как инструмент для оформления и структурирования идей.
>
> **EN** — The architecture, documentation and README of this project were developed with the assistance of AI ([Claude](https://claude.ai) by Anthropic). The concept, name and all technical decisions were made by a human; AI was used as a tool for formatting and structuring ideas.

---

## О проекте · About

**RU** — CupruxOS — это Unix-подобная операционная система с гибридным ядром, написанная на Rust с нуля. Название происходит от *Cuprum* (латинское название меди, Cu) и *Unix*. Как медь — проводник и основа электроники — CupruxOS стремится быть надёжным проводником между железом и программами.

**EN** — CupruxOS is a Unix-like operating system with a hybrid kernel, written in Rust from scratch. The name comes from *Cuprum* (Latin for copper, Cu) and *Unix*. Just as copper is a conductor and the foundation of electronics — CupruxOS aims to be a reliable conductor between hardware and software.

---

## Философия · Philosophy

| | RU | EN |
|---|---|---|
| 🛡️ | **Безопасность** через Rust и Capability | **Safety** through Rust and Capabilities |
| ⚡ | **Минимализм** — мало кода в ядре | **Minimalism** — minimal kernel code |
| 🔒 | **Изоляция** — всё общение через IPC | **Isolation** — all communication via IPC |
| 🌍 | **Мультиплатформенность** с первого дня | **Portability** from day one |

---

## Архитектура · Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Userspace                        │
│                                                     │
│  [App]    [App]    [App]    [App]    [App]          │
│    │        │        │        │        │            │
│    └────────┴────────┴────────┘        │            │
│                   │ IPC                │            │
│  ┌────────────────┼───────────────┐   │            │
│  │  VFS Server    │  Driver Mgr   │   │            │
│  │  (userspace)   │  (userspace)  │   │ Network    │
│  └────────────────┴───────────────┘   │ Stack      │
│                   │ IPC               │            │
├───────────────────┼───────────────────┼────────────┤
│              K E R N E L              │            │
│                                       │            │
│  ┌──────────┐  ┌──────────┐  ┌─────────────────┐  │
│  │  Sched   │  │   IPC    │  │  Capability     │  │
│  │  (MLFQ)  │  │  + Ports │  │  System         │  │
│  └──────────┘  └──────────┘  └─────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │           Memory Management                  │  │
│  │   PMM (Buddy)  ·  VMM  ·  Heap (Slab)       │  │
│  └──────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │     HAL — Hardware Abstraction Layer         │  │
│  │   x86_64      ·    ARM64    ·   RISC-V       │  │
│  └──────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────┤
│                   Bootloader                        │
│              Limine Protocol                        │
└─────────────────────────────────────────────────────┘
```

### Тип ядра · Kernel Type

**RU** — Гибридное ядро (как macOS XNU): драйверы и планировщик работают в ядре для производительности, часть сервисов (VFS, сеть) — в userspace для изоляции.

**EN** — Hybrid kernel (like macOS XNU): drivers and scheduler run in kernel for performance, some services (VFS, network) run in userspace for isolation.

---

## Ключевые компоненты · Key Components

### 🔗 IPC + Capability System

**RU** — Вся коммуникация между процессами строится на портах (Mach-style) и capability (seL4-style). Capability — это unforgeable токен, который даёт право на доступ к ресурсу.

**EN** — All inter-process communication is built on ports (Mach-style) and capabilities (seL4-style). A Capability is an unforgeable token that grants the right to access a resource.

```rust
enum CapKind {
    Port(PortId),              // право писать/читать порт · port read/write
    Memory(PhysAddr, usize),   // право маппить регион · map memory region  
    Task(TaskId),              // право управлять задачей · control task
}

bitflags! {
    struct Rights: u32 {
        const READ  = 0b001;
        const WRITE = 0b010;
        const GRANT = 0b100;   // передать другому · transfer to another
    }
}
```

**Передача данных · Data Transfer:**
- Малые данные `< 4KB` → копия inline в сообщении · copied inline in message
- Большие данные `≥ 4KB` → Shared Memory через MemoryCap (zero-copy)

---

### ⚙️ Планировщик · Scheduler

**RU** — MLFQ (Multilevel Feedback Queue) адаптированный под IPC. Пробуждение от IPC всегда получает наивысший приоритет.

**EN** — MLFQ (Multilevel Feedback Queue) adapted for IPC. IPC wake-ups always get the highest priority.

```
Queue 0  [ 1ms  ] ← IPC wake-ups         (highest)
Queue 1  [ 5ms  ] ← interactive tasks
Queue 2  [ 20ms ] ← normal tasks
Queue 3  [ 100ms] ← background tasks     (lowest)
```

```rust
enum TaskState {
    Running,
    Ready,
    Blocked(BlockReason),
    Dead,
}

enum BlockReason {
    WaitingForMessage(PortId),
    WaitingForReply(PortId),
    Sleeping(u64),             // наносекунды · nanoseconds
}
```

---

### 🧠 Управление памятью · Memory Management

| Слой · Layer | Алгоритм · Algorithm | Назначение · Purpose |
|---|---|---|
| **PMM** | Buddy Allocator | Физические страницы · Physical pages |
| **VMM** | Page Tables (HAL) | Виртуальные адр. пространства · Virtual address spaces |
| **Kernel Heap** | Slab Allocator | `Box<T>`, `Vec<T>` в ядре · in kernel |

**RU** — Страницы выделяются лениво — физическая аллокация происходит только при первом обращении (page fault).

**EN** — Pages are allocated lazily — physical allocation happens only on first access (page fault).

---

### 💾 Файловая система · Filesystem — CuprumFS

**RU** — Гибридная модель: Inode + Extent Tree. Вместо списка блоков хранятся непрерывные регионы `(start, length)`.

**EN** — Hybrid model: Inode + Extent Tree. Instead of a block list, contiguous regions `(start, length)` are stored.

```
Inode
  └── Extent Tree
        ├── Extent(start: 1000, len: 48)
        ├── Extent(start: 2048, len: 16)
        └── Extent(start: 5000, len: 128)
```

**Возможности · Features:**
- ✅ Journaling (Write-Ahead Log)
- ✅ HTree для больших директорий · for large directories
- ✅ Checksum везде · everywhere
- ✅ Совместимость с ext2 · ext2 compatibility
- ✅ FAT32 для флешек и UEFI · for USB and UEFI

---

### 📞 Syscall интерфейс · Syscall Interface

**RU** — Всего ~15 системных вызовов (против 300+ в Linux). Всё остальное — через IPC к userspace серверам.

**EN** — Only ~15 system calls (vs 300+ in Linux). Everything else goes through IPC to userspace servers.

| Syscall | Группа · Group | RU | EN |
|---|---|---|---|
| `ipc_call(cap, msg)` | IPC | Синхронный вызов | Sync call |
| `ipc_send(cap, msg)` | IPC | Асинхронная отправка | Async send |
| `ipc_recv(cap)` | IPC | Ждать сообщения | Wait for message |
| `cap_create_port()` | Capability | Создать порт | Create port |
| `cap_grant(cap, task)` | Capability | Передать capability | Transfer cap |
| `mem_map(cap, addr)` | Memory | Замаппить регион | Map region |
| `mem_alloc(size)` | Memory | Запросить память | Alloc memory |
| `task_spawn(bin, caps)` | Task | Создать задачу | Spawn task |
| `task_exit(code)` | Task | Завершиться | Exit |
| `time_now()` | Time | Время в нс | Time in ns |

---

## Платформы · Platforms

| Архитектура · Architecture | Загрузчик · Bootloader | Статус · Status |
|---|---|---|
| `x86_64` | Limine (UEFI/BIOS) | 🟡 Планируется · Planned |
| `aarch64` | Limine (UEFI) | 🟡 Планируется · Planned |
| `riscv64` | Limine (UEFI) | 🟡 Планируется · Planned |

---

## Структура проекта · Project Structure

```
cupruxos/
├── bootloader/              # Загрузчик · Bootloader
│   ├── x86_64/
│   ├── aarch64/
│   └── riscv64/
├── kernel/
│   └── src/
│       ├── arch/            # HAL — архитектурный код · arch-specific code
│       │   ├── x86_64/
│       │   ├── aarch64/
│       │   └── riscv64/
│       ├── mm/              # Управление памятью · Memory management
│       │   ├── pmm.rs       # Physical Memory Manager (Buddy)
│       │   ├── vmm.rs       # Virtual Memory Manager
│       │   └── heap.rs      # Kernel Heap (Slab)
│       ├── sched/           # Планировщик · Scheduler (MLFQ)
│       ├── ipc/             # IPC + Capability System
│       ├── vfs/             # Virtual Filesystem interface
│       ├── drivers/         # Kernel-space drivers
│       └── syscall/         # Syscall handler (~15 calls)
├── userland/                # Userspace серверы · Servers
│   ├── init/               # Первый процесс · First process
│   ├── vfs_server/         # Файловая система · Filesystem
│   └── driver_manager/     # Управление драйверами · Driver management
├── libcuprum/               # Userspace библиотека · Library
└── fs/
    └── cuprumfs/           # CuprumFS tools
```

---

## Дорожная карта · Roadmap

- [ ] **Этап 1** — Bootloader (Limine, все платформы · all platforms)
- [ ] **Этап 2** — Минимальное ядро · Minimal kernel (GDT/IDT, paging, UART)
- [ ] **Этап 3** — PMM + VMM (Buddy allocator, virtual address spaces)
- [ ] **Этап 4** — Kernel Heap (Slab allocator, `Box<T>` / `Vec<T>`)
- [ ] **Этап 5** — Планировщик · Scheduler (MLFQ, context switch, SMP)
- [ ] **Этап 6** — IPC + Capability (ports, messages, rights)
- [ ] **Этап 7** — Syscall interface (~15 calls) + `libcuprum`
- [ ] **Этап 8** — Init + VFS server (first userspace process)
- [ ] **Этап 9** — CuprumFS (native filesystem)
- [ ] **Этап 10** — ext2 + FAT32 compatibility

---

## Технологии · Tech Stack

| | |
|---|---|
| **Язык · Language** | Rust (`no_std`) |
| **Загрузчик · Bootloader** | [Limine Protocol](https://github.com/limine-bootloader/limine) |
| **Архитектура ядра · Kernel** | Hybrid (Monolith perf + Microkernel isolation) |
| **IPC модель · IPC model** | Mach-style ports + seL4-style capabilities |
| **Планировщик · Scheduler** | MLFQ + IPC-aware preemption |
| **Аллокатор · Allocator** | Buddy (PMM) + Slab (Heap) |
| **Файловая система · FS** | CuprumFS (Inode + Extents + WAL) |

---

## Вдохновение · Inspiration

- **[XNU](https://github.com/apple-oss-distributions/xnu)** — гибридная архитектура · hybrid architecture
- **[seL4](https://sel4.systems/)** — capability система · capability system  
- **[Mach](https://en.wikipedia.org/wiki/Mach_(kernel))** — IPC модель портов · port IPC model
- **[Redox OS](https://www.redox-os.org/)** — Rust OS inspiration
- **[OSDev Wiki](https://wiki.osdev.org/)** — незаменимый ресурс · invaluable resource

---

<div align="center">

**CupruxOS** — *Cuprum (Cu) + Unix*

Built with 🦀 Rust · Designed for safety, minimalism, and portability  
Создан на Rust · Спроектирован для безопасности, минимализма и переносимости

</div>
