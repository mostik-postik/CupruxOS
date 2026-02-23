; CupruxOS — точка входа x86_64 / entry point x86_64
;
; Limine загружает ядро и прыгает сюда в 64-bit protected mode.
; Limine loads the kernel and jumps here in 64-bit protected mode.
;
; Наша задача / Our job:
;   1. Проверить что мы вообще запустились от Limine
;   2. Настроить стек
;   3. Обнулить .bss
;   4. Прыгнуть в kernel_main (Rust)

bits 64
section .text

global _start
extern kernel_main

_start:
    ; Отключить прерывания пока не настроили IDT
    ; Disable interrupts until IDT is ready
    cli

    ; Настроить стек — указываем на вершину нашего boot stack
    ; Set up stack — point to top of our boot stack
    lea rsp, [rel boot_stack_top]

    ; Выровнять стек по 16 байт (ABI требование)
    ; Align stack to 16 bytes (ABI requirement)
    and rsp, ~0xF

    ; Обнулить .bss — Rust ожидает что статические переменные = 0
    ; Zero .bss — Rust expects static variables = 0
    extern __bss_start
    extern __bss_end
    lea rdi, [rel __bss_start]
    lea rcx, [rel __bss_end]
    sub rcx, rdi
    xor eax, eax
    rep stosb

    ; Прыгнуть в Rust! / Jump into Rust!
    call kernel_main

    ; kernel_main никогда не возвращается (-> !)
    ; kernel_main never returns (-> !)
    ; Но на всякий случай — бесконечный цикл
    ; But just in case — infinite loop
.hang:
    cli
    hlt
    jmp .hang

section .bss
align 16
    ; Boot stack — 64KB, до инициализации GDT/TSS
    ; Boot stack — 64KB, before GDT/TSS initialization
boot_stack_bottom:
    resb 64 * 1024
boot_stack_top:
