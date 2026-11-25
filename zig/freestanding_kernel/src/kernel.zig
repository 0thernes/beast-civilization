//! Freestanding Bare-Metal x86-64 Multiboot2 Microkernel v2 (`x86_64-freestanding-none`)
//! Zero libc, zero OS, zero heap — embeds BOTH:
//!   1. `b1.58` 5-trit/byte ternary neural reflex core (`kernel_b158_reflex_infer`)
//!   2. OCP Blackwell `NVFP4 E2M1` 4-bit neural priority core (`kernel_nvfp4_priority_infer`)
//!   3. Ring-0 self-check telemetry framed over COM1 serial (0x3F8) + VGA text buffer (0xB8000)

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

/// Static 16 KiB Ring-0 kernel stack in `.bss` (zero dynamic allocation).
var kernel_stack: [16384]u8 align(16) = undefined;

/// Embedded 5-trit/byte (`b1.58`) reflex weights inside Ring-0 `.rodata`.
const REFLEX_WEIGHTS_B158: [4]u8 = .{ 161, 82, 200, 41 };

/// Embedded fixed-point scaled OCP NVFP4 E2M1 table (`value * 2` -> integer half-units, zero FPU/SSE needed in Ring-0!).
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

/// Pure integer Ring-0 NVFP4 E2M1 neural priority inference (no x87/SSE required in kernel mode!).
export fn kernel_nvfp4_priority_infer(feats: *const [16]i32) callconv(.c) i64 {
    var acc: i64 = 0;
    for (PRIORITY_WEIGHTS_NVFP4, 0..) |byte, bi| {
        const lo: usize = byte & 0x0F;
        const hi: usize = byte >> 4;
        acc += @as(i64, NVFP4_HALF_UNITS[lo]) * @as(i64, feats[bi * 2]);
        acc += @as(i64, NVFP4_HALF_UNITS[hi]) * @as(i64, feats[bi * 2 + 1]);
    }
    return acc;
}

export fn kernel_main() callconv(.c) noreturn {
    vgaWriteRow(0, "BEAST CIVILIZATION RING-0 MICROKERNEL v0.2 (ZIG FREESTANDING x86_64)", 0x0A);
    serialWrite("BEAST-KERNEL-V2: BOOT OK\n");

    const sensors: [20]i32 = .{ 3, -1, 4, 1, 5, -9, 2, 6, -5, 3, 5, -8, 9, 7, -9, 3, 2, 3, -8, 4 };
    const feats: [16]i32 = .{ 5, 2, -3, 7, 1, -4, 6, 3, 2, -1, 8, -2, 4, 5, -6, 3 };

    const reflex = kernel_b158_reflex_infer(&sensors);
    const prio = kernel_nvfp4_priority_infer(&feats);

    if (reflex >= 0 and prio != 0) {
        vgaWriteRow(1, "DUAL NEURAL RING-0 CORE (B1.58 + NVFP4): ARMED & ACTIVE", 0x0F);
        serialWrite("BEAST-KERNEL-V2: B158+NVFP4 SELF-TEST PASS\n");
    } else {
        vgaWriteRow(1, "DUAL NEURAL RING-0 CORE: DEFENSIVE CLAMP ENGAGED", 0x0E);
        serialWrite("BEAST-KERNEL-V2: DEFENSIVE CLAMP\n");
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
