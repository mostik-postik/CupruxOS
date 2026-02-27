//! x86_64 boot entry point — replaces entry.asm (no NASM required)
//!
//! Limine jumps here in 64-bit long mode with interrupts disabled.
//! We set up the boot stack, zero BSS, then call kernel_main.

use core::arch::global_asm;

global_asm!(
    r#"
.section .text
.global _start
_start:
    cli

    /* Switch to our 64KB boot stack */
    leaq boot_stack_top(%rip), %rsp
    andq $-16, %rsp

    /* Zero BSS: rdi = __bss_start, rcx = byte count, al = 0 */
    leaq __bss_start(%rip), %rdi
    leaq __bss_end(%rip),   %rcx
    subq %rdi, %rcx
    xorl %eax, %eax
    rep stosb

    callq kernel_main

    /* kernel_main never returns — halt just in case */
.hang:
    cli
    hlt
    jmp .hang

.section .bss
.balign 16
boot_stack_bottom:
    .skip 65536
boot_stack_top:
"#,
    options(att_syntax)
);
