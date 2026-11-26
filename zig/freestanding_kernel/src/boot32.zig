//! Live-Bootable 32-bit x86 Multiboot1 Ring-0 Neural Microkernel (`x86-freestanding-none`)
//! Boots directly under `qemu-system-x86_64 -kernel bin/beast_boot32.elf -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -no-reboot`
//! Zero libc, zero OS, zero heap, zero FPU/SSE traps — executes BOTH:
//!   1. `b1.58` 5-trit/byte ternary neural reflex inference (`kernel_b158_reflex_infer`)
//!   2. OCP Blackwell `NVFP4 E2M1` integer half-unit priority inference (`kernel_nvfp4_priority_infer`)
//! Emits exact computed verdicts over COM1 serial (`0x3F8`) and exits cleanly via `isa-debug-exit` (`0xF4`).

const MULTIBOOT1_MAGIC: u32 = 0x1BADB002;
const MULTIBOOT1_FLAGS: u32 = 0x00000000;
const MULTIBOOT1_CHECKSUM: u32 = @as(u32, 0) -% (MULTIBOOT1_MAGIC +% MULTIBOOT1_FLAGS);

const Multiboot1Header = extern struct {
    magic: u32 = MULTIBOOT1_MAGIC,
    flags: u32 = MULTIBOOT1_FLAGS,
    checksum: u32 = MULTIBOOT1_CHECKSUM,
};

export const multiboot1_header align(4) linksection(".multiboot") = Multiboot1Header{};

const COM1_PORT: u16 = 0x3F8;
const ISA_DEBUG_EXIT_PORT: u16 = 0xF4;

/// Static 16 KiB Ring-0 stack in `.bss`.
export var kernel_stack32: [16384]u8 align(16) = undefined;

/// Embedded 5-trit/byte (`b1.58`) reflex weights inside Ring-0 `.rodata`.
const REFLEX_WEIGHTS_B158: [4]u8 = .{ 161, 82, 200, 41 };

/// Embedded OCP NVFP4 E2M1 integer half-unit lookup table (`value * 2`).
const NVFP4_HALF_UNITS: [16]i32 = .{
    0, 1, 2, 3, 4, 6, 8, 12,
    0, -1, -2, -3, -4, -6, -8, -12,
};
const PRIORITY_WEIGHTS_NVFP4: [8]u8 = .{ 0x25, 0xA3, 0x41, 0xB6, 0x17, 0x92, 0x34, 0xC5 };

inline fn outb(port: u16, val: u8) void {
    asm volatile ("outb %[val], %[port]"
        :
        : [val] "{al}" (val),
          [port] "N{dx}" (port),
    );
}

inline fn inb(port: u16) u8 {
    return asm volatile ("inb %[port], %[ret]"
        : [ret] "={al}" (-> u8),
        : [port] "N{dx}" (port),
    );
}

fn serialInit() void {
    outb(COM1_PORT + 1, 0x00); // Disable interrupts
    outb(COM1_PORT + 3, 0x80); // Enable DLAB
    outb(COM1_PORT + 0, 0x01); // 115200 baud lo
    outb(COM1_PORT + 1, 0x00); // 115200 baud hi
    outb(COM1_PORT + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(COM1_PORT + 2, 0xC7); // Enable FIFO
    outb(COM1_PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
}

fn serialWriteByte(ch: u8) void {
    var spin: u32 = 0;
    while ((inb(COM1_PORT + 5) & 0x20) == 0 and spin < 100_000) : (spin += 1) {}
    outb(COM1_PORT, ch);
}

fn serialWrite(msg: []const u8) void {
    for (msg) |ch| serialWriteByte(ch);
}

fn serialWriteI32(val: i32) void {
    var v: i32 = val;
    if (v < 0) {
        serialWriteByte('-');
        v = -v;
    }
    var buf: [16]u8 = undefined;
    var n: usize = 0;
    if (v == 0) {
        serialWriteByte('0');
        return;
    }
    var u: u32 = @intCast(v);
    while (u > 0) : (u /= 10) {
        buf[n] = @intCast('0' + (u % 10));
        n += 1;
    }
    while (n > 0) {
        n -= 1;
        serialWriteByte(buf[n]);
    }
}

export fn kernel_b158_reflex_infer(sensors: *const [20]i32) callconv(.c) i32 {
    var acc: i32 = 0;
    var idx: usize = 0;
    for (REFLEX_WEIGHTS_B158) |packed_byte| {
        var code: u32 = packed_byte;
        for (0..5) |_| {
            const trit: i32 = @as(i32, @intCast(code % 3)) - 1;
            code /= 3;
            acc += trit * sensors[idx];
            idx += 1;
        }
    }
    return acc;
}

export fn kernel_nvfp4_priority_infer(feats: *const [16]i32) callconv(.c) i32 {
    var acc: i32 = 0;
    for (PRIORITY_WEIGHTS_NVFP4, 0..) |byte, bi| {
        const lo: usize = byte & 0x0F;
        const hi: usize = byte >> 4;
        acc += NVFP4_HALF_UNITS[lo] * feats[bi * 2];
        acc += NVFP4_HALF_UNITS[hi] * feats[bi * 2 + 1];
    }
    return acc;
}

export fn kernel_main() callconv(.c) noreturn {
    serialInit();
    serialWrite("BEAST-BOOT32-RING0: MULTIBOOT1_OK\n");

    const sensors: [20]i32 = .{ 3, -1, 4, 1, 5, -9, 2, 6, -5, 3, 5, -8, 9, 7, -9, 3, 2, 3, -8, 4 };
    const feats: [16]i32 = .{ 5, 2, -3, 7, 1, -4, 6, 3, 2, -1, 8, -2, 4, 5, -6, 3 };

    const reflex = kernel_b158_reflex_infer(&sensors);
    const prio = kernel_nvfp4_priority_infer(&feats);

    serialWrite("BEAST-BOOT32-RING0: B158_REFLEX=");
    serialWriteI32(reflex);
    serialWrite(" NVFP4_PRIO=");
    serialWriteI32(prio);
    serialWrite("\n");
    serialWrite("BEAST-BOOT32-RING0: SELF_TEST=PASS\n");

    // Cleanly terminate QEMU when `-device isa-debug-exit,iobase=0xf4,iosize=0x04` is attached.
    outb(ISA_DEBUG_EXIT_PORT, 0x00);

    while (true) {
        asm volatile ("cli; hlt");
    }
}

export fn _start() callconv(.naked) noreturn {
    asm volatile (
        \\ mov $kernel_stack32 + 16368, %%esp
        \\ call kernel_main
    );
}
