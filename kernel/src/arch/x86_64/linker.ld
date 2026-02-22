/* CupruxOS x86_64 linker script */
/* Limine загружает ядро по этому адресу / Limine loads kernel at this address */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(kernel_main)

SECTIONS {
    . = 0xFFFFFFFF80000000;  /* Kernel virtual base (высокая память / high mem) */

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    }
}
