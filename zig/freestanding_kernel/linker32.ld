/* Linker script for live-bootable 32-bit x86 Multiboot1 AI Microkernel (`x86-freestanding-none`).
 * Loads at 1 MiB (0x00100000) with `.multiboot` placed first (strictly inside first 8192 bytes),
 * enabling direct `qemu-system-x86_64 -kernel bin/beast_boot32.elf` Ring-0 execution without GRUB/ISO.
 */
ENTRY(_start)

SECTIONS {
    . = 1M;

    .text : ALIGN(4K) {
        KEEP(*(.multiboot))
        *(.text*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata*)
    }

    .data : ALIGN(4K) {
        *(.data*)
    }

    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss*)
    }
}
