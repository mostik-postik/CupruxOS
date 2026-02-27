OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text   PT_LOAD FLAGS(0x5); /* R+X = Execute + Read */
    rodata PT_LOAD FLAGS(0x4); /* R   = Read only      */
    data   PT_LOAD FLAGS(0x6); /* R+W = Read + Write   */
}

SECTIONS
{
    /* Virtual address base for kernel (high memory on x86-64) */
    . = 0xFFFFFFFF80000000;

    /* Text section: executable code */
    .text : ALIGN(4K)
    {
        *(.text .text.*)
    } :text

    /* Read-only data (constants, string literals, rodata) */
    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    } :rodata

    /* Initialized data (global variables with initial values) */
    .data : ALIGN(4K)
    {
        *(.data .data.*)
    } :data

    /* BSS section: uninitialized zero-initialized data */
    .bss (NOLOAD) : ALIGN(4K)
    {
        __bss_start = .;
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(8);
        __bss_end = .;
    } :data

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.debug*)
        *(.note*)
    }
}

