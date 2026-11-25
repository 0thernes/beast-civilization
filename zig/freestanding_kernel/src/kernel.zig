//! Freestanding Bare-Metal x86-64 Multiboot2 Microkernel (`x86_64-freestanding-none`)
//! Zero libc, zero OS, zero heap — embeds a live `b1.58` 5-trit/byte ternary neural reflex
//! core directly inside Ring-0 kernel space and writes telemetry to COM1 serial + VGA 0xB8000.

const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const MULTIBOOT2_ARCH_X86: u32 = 0;
const MULTIBOOT2_HEADER_LEN: u32 = 24;
const MULTIBOOT2_CHECKSUM: u32 = @as(u32, 0) -% (MULTIBOOT2_MAGIC +% MULTIBOOT2_ARCH_X86 +% MULTIBOOT2_HEADER_LEN);

const Multiboot2Header = extern struct {
    magic: u32 = MULTIBOOT2_MAGIC,
    arch: u32 = MULTIBOOT2_ARCH_X86,
    header_len: u32 = MULTIBOOT2_HEADER_LEN,
    checksum: u32 = MULTIBOOT2_CHECKSUM,
    end_tag_type: u16 = 0,
    end_tag_flags: u16 = 0,
    end_tag_size: u32 = 8,
};

export const multiboot2_header align(8) linksection(".multiboot") = Multiboot2Header{};

const VGA_TEXT_BUF: [*]volatile u16 = @ptrFromInt(0xB8000);
const COM1_PORT: u16 = 0x3F8;

/// Static 16 KiB kernel stack in `.bss` (zero dynamic memory).
var kernel_stack: [16384]u8 align(16) = undefined;

/// Embedded 5-trit/byte (`b1.58`) reflex weights inside Ring-0 kernel `.rodata`.
const REFLEX_WEIGHTS_B158: [4]u8 = .{ 161, 82, 200, 41 };

inline fn outb(port: u16, val: u8) void {
    asm volatile ("outb %[val], %[port]"
        :
        : [val] "{al}" (val),
          [port] "N{dx}" (port),
    );
}

fn serialWrite(msg: []const u8) void {
    for (msg) |ch| outb(COM1_PORT, ch);
}

fn vgaWriteRow(row: usize, msg: []const u8, attr: u8) void {
    const base = row * 80;
    for (msg, 0..) |ch, i| {
        if (i >= 80) break;
        VGA_TEXT_BUF[base + i] = (@as(u16, attr) << 8) | ch;
    }
}

/// Pure freestanding `b1.58` 5-trit/byte ternary neural reflex inference inside Ring-0.
export fn kernel_b158_reflex_infer(sensors: *const [20]i32) callconv(.c) i64 {
    var acc: i64 = 0;
    var idx: usize = 0;
    for (REFLEX_WEIGHTS_B158) |packed_byte| {
        var code: u32 = packed_byte;
        for (0..5) |_| {
            const trit: i64 = @as(i64, @intCast(code % 3)) - 1;
            code /= 3;
            acc += trit * @as(i64, sensors[idx]);
            idx += 1;
        }
    }
    return acc;
}

export fn kernel_main() callconv(.c) noreturn {
    vgaWriteRow(0, "BEAST CIVILIZATION RING-0 MICROKERNEL (ZIG FREESTANDING x86_64)", 0x0A);
    serialWrite("BEAST-KERNEL: BOOT OK\n");

    const sensors: [20]i32 = .{ 3, -1, 4, 1, 5, -9, 2, 6, -5, 3, 5, -8, 9, 7, -9, 3, 2, 3, -8, 4 };
    const verdict = kernel_b158_reflex_infer(&sensors);
    if (verdict >= 0) {
        vgaWriteRow(1, "B1.58 TERNARY REFLEX CORE: ARMED & ACTIVE", 0x0F);
        serialWrite("BEAST-KERNEL: B158 REFLEX POSITIVE\n");
    } else {
        vgaWriteRow(1, "B1.58 TERNARY REFLEX CORE: DEFENSIVE CLAMP", 0x0E);
        serialWrite("BEAST-KERNEL: B158 REFLEX DEFENSIVE\n");
    }

    while (true) {
        asm volatile ("hlt");
    }
}

export fn _start() callconv(.naked) noreturn {
    asm volatile (
        \\ lea %[stack_top], %%rsp
        \\ call kernel_main
        :
        : [stack_top] "m" (kernel_stack[kernel_stack.len - 16]),
    );
}
