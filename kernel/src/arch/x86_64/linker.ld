OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(kernel_main)

PHDRS
{
    text   PT_LOAD FLAGS(0x5); /* R+X = 1 (Execute) + 4 (Read) */
    data   PT_LOAD FLAGS(0x6); /* R+W = 2 (Write) + 4 (Read) */
}

SECTIONS
{
    /* Virtual address base for kernel (high memory on x86-64) */
    . = 0xFFFFFFFF80000000;

    /* Text section: executable code */
    .text : ALIGN(4K) AT(ADDR(.text))
    {
        *(.multiboot_header)     /* Optional: for Multiboot/Multiboot2 */
        *(.text .text.*)
    } :text

    /* Read-only data (constants, string literals, rodata) */
    .rodata : ALIGN(4K) AT(ADDR(.rodata))
    {
        *(.rodata .rodata.*)
    } :data   /* Note: often mapped R+X or R only; adjust PHDR if needed */

    /* Initialized data (global variables with initial values) */
    .data : ALIGN(4K) AT(ADDR(.data))
    {
        *(.data .data.*)
    } :data

    /* BSS section: uninitialized zero-initialized data */
    .bss (NOLOAD) : ALIGN(4K) AT(ADDR(.bss))
    {
        __bss_start = .;
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(8);            /* Ensure 8-byte alignment for safety */
        __bss_end = .;
    } :data

    /DISCARD/ :
    {
        *(.eh_frame)             /* Avoid C++ exception tables in kernel */
        *(.debug*)
        *(.note*)
    }
}

