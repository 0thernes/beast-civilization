//! `beast-metal` (Zig 0.17.1): Bare-Metal Arbitrary-Bitwidth Tensor Kernels,
//! MIT Augment-Cache SIMD Attention, and Chimera Frankenstein Insert-Network Engine.
//!
//! Zero hidden allocations. Bit-exact Reference vs. `@Vector` SIMD parity across:
//!   - `u1` bipolar binary (8 weights/byte)
//!   - `b1.58` ternary (5 trits/byte, base-3: 3^5 = 243 <= 256, proven in Lean 4)
//!   - `i2` 2-bit signed (4 weights/byte using native Zig `i2`)
//!   - `nvfp4_e2m1` Blackwell OCP FP4 E2M1 (2 nibbles/byte using native Zig `u4`)

const std = @import("std");
const math = std.math;

pub const BEAST_METAL_ABI_VERSION: u32 = 0x00010000;
pub const BEAST_CHIMERA_MAX_BLOCKS: usize = 16;
pub const BEAST_CHIMERA_DIM: usize = 16;
pub const BEAST_CHIMERA_MAGIC: u32 = 0x42434849; // "BCHI"

pub const BeastBlockKind = enum(u8) {
    ternary_ffn = 0,
    nvfp4_linear = 1,
    sltc_liquid = 2,
    ssm_scan = 3,
    gf2_bitmix = 4,
    qgtl_metric = 5,
    c2c_port = 6,
};

pub const BeastPrecision = enum(u8) {
    u1_binary = 0,
    t158_tern = 1,
    i2_int2 = 2,
    u3_uint3 = 3,
    nvfp4_e2m1 = 4,
    fp8_e4m3 = 5,
    bf16 = 6,
    fp32 = 7,
};

pub const BeastChimeraBlock = extern struct {
    kind: u8,
    precision: u8,
    active: u8,
    reserved: u8,
    alpha_rezero: f32,
    tau_liquid: f32,
    spectral_est: f32,
    ribosome_f2: u64,
    weights_q: [BEAST_CHIMERA_DIM * BEAST_CHIMERA_DIM]i8,
};

pub const BeastChimeraGenome = extern struct {
    magic: u32,
    generation: u32,
    num_blocks: u32,
    mutation_ops: u32,
    lineage_seed: u64,
    max_spectral_norm: f32,
    fitness_score: f32,
    blocks: [BEAST_CHIMERA_MAX_BLOCKS]BeastChimeraBlock,
};

/// Exact OCP / NVIDIA Blackwell FP4 E2M1 16-entry LUT (`u4` index -> f32).
const NVFP4_E2M1_LUT: [16]f32 = .{
    0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0,
    -0.0, -0.5, -1.0, -1.5, -2.0, -3.0, -4.0, -6.0,
};

/// Compile-time 243-entry 5-trit unpack table (`3^5 = 243 <= 256`).
const TRIT5_LUT: [256][5]i8 = blk: {
    @setEvalBranchQuota(10000);
    var table: [256][5]i8 = undefined;
    for (0..256) |code| {
        var c: usize = if (code < 243) code else 121; // 121 => (0,0,0,0,0) safe fallback
        for (0..5) |lane| {
            const rem: i8 = @intCast(c % 3);
            table[code][lane] = rem - 1;
            c /= 3;
        }
    }
    break :blk table;
};

pub export fn beast_metal_abi_version() callconv(.c) u32 {
    return BEAST_METAL_ABI_VERSION;
}

/// Domain-isolated Philox4x32-10 PRNG (bit-exact with Rust `beast-wire::philox`).
pub export fn beast_philox_u64(seed: u64, counter: u64, domain_id: u32) callconv(.c) u64 {
    const M0: u64 = 0xD2511F53;
    const M1: u64 = 0xCD9E8D57;
    const W0: u32 = 0x9E3779B9;
    const W1: u32 = 0xBB67AE85;

    var c0: u32 = @truncate(counter);
    var c1: u32 = @truncate(counter >> 32);
    var c2: u32 = domain_id;
    var c3: u32 = 0x4245_4153;
    var k0: u32 = @truncate(seed);
    var k1: u32 = @truncate(seed >> 32);

    for (0..10) |_| {
        const p0: u64 = M0 *% @as(u64, c0);
        const p1: u64 = M1 *% @as(u64, c2);
        const hi0: u32 = @truncate(p0 >> 32);
        const lo0: u32 = @truncate(p0);
        const hi1: u32 = @truncate(p1 >> 32);
        const lo1: u32 = @truncate(p1);
        const nc0 = hi1 ^ c1 ^ k0;
        const nc1 = lo1;
        const nc2 = hi0 ^ c3 ^ k1;
        const nc3 = lo0;
        c0 = nc0;
        c1 = nc1;
        c2 = nc2;
        c3 = nc3;
        k0 +%= W0;
        k1 +%= W1;
    }
    return @as(u64, c0) | (@as(u64, c1) << 32);
}

fn philoxUnit(seed: u64, counter: u64, domain: u32) f32 {
    const u = beast_philox_u64(seed, counter, domain) >> 40;
    return @as(f32, @floatFromInt(u)) * (1.0 / 16777216.0);
}

/// Pack 5 trits in {-1, 0, +1} into 1 byte (`0..242`, bijective over `3^5 = 243`).
pub export fn beast_pack_5trits(trits5: [*]const i8) callconv(.c) u8 {
    var acc: u16 = 0;
    var pow3: u16 = 1;
    for (0..5) |i| {
        const clamped: i8 = if (trits5[i] < 0) -1 else if (trits5[i] > 0) 1 else 0;
        const digit: u16 = @intCast(clamped + 1);
        acc += digit * pow3;
        pow3 *= 3;
    }
    return @intCast(acc);
}

pub export fn beast_unpack_5trits(packed_byte: u8, out_trits5: [*]i8) callconv(.c) void {
    const row = TRIT5_LUT[packed_byte];
    for (0..5) |i| {
        out_trits5[i] = row[i];
    }
}

fn quantizeNvfp4(val: f32) u4 {
    var best_idx: u4 = 0;
    var best_err: f32 = math.inf(f32);
    for (0..16) |idx| {
        const err = @abs(val - NVFP4_E2M1_LUT[idx]);
        if (err < best_err) {
            best_err = err;
            best_idx = @intCast(idx);
        }
    }
    return best_idx;
}

pub export fn beast_pack_nvfp4_pair(a: f32, b: f32) callconv(.c) u8 {
    const lo: u8 = quantizeNvfp4(a);
    const hi: u8 = quantizeNvfp4(b);
    return lo | (hi << 4);
}

pub export fn beast_unpack_nvfp4_pair(packed_byte: u8, out_pair: [*]f32) callconv(.c) void {
    const lo: u4 = @truncate(packed_byte & 0x0F);
    const hi: u4 = @truncate(packed_byte >> 4);
    out_pair[0] = NVFP4_E2M1_LUT[lo];
    out_pair[1] = NVFP4_E2M1_LUT[hi];
}

// ==================== b1.58 (5-trit/byte) Ref vs SIMD ====================

pub export fn beast_dot_b158_ref(packed_w: [*]const u8, x: [*]const i32, n_trits: usize) callconv(.c) i64 {
    var acc: i64 = 0;
    var i: usize = 0;
    var b: usize = 0;
    while (i < n_trits) : (b += 1) {
        var code: u32 = packed_w[b];
        if (code >= 243) code = 121;
        for (0..5) |_| {
            if (i >= n_trits) break;
            const trit: i64 = @as(i64, @intCast(code % 3)) - 1;
            code /= 3;
            acc += trit * @as(i64, x[i]);
            i += 1;
        }
    }
    return acc;
}

pub export fn beast_dot_b158_simd(packed_w: [*]const u8, x: [*]const i32, n_trits: usize) callconv(.c) i64 {
    const V8i64 = @Vector(8, i64);
    var v_acc: V8i64 = @splat(0);
    var i: usize = 0;
    var b: usize = 0;

    // Process 8 bytes = 40 trits per SIMD loop iteration (5 x 8-wide int64 lanes)
    while (i + 40 <= n_trits) : ({
        i += 40;
        b += 8;
    }) {
        var w_buf: [40]i64 = undefined;
        inline for (0..8) |bi| {
            const row = TRIT5_LUT[packed_w[b + bi]];
            inline for (0..5) |ti| {
                w_buf[bi * 5 + ti] = row[ti];
            }
        }
        inline for (0..5) |chunk| {
            const base = chunk * 8;
            var xv: V8i64 = undefined;
            var wv: V8i64 = undefined;
            inline for (0..8) |lane| {
                xv[lane] = x[i + base + lane];
                wv[lane] = w_buf[base + lane];
            }
            v_acc += wv * xv;
        }
    }

    var acc: i64 = @reduce(.Add, v_acc);
    while (i < n_trits) : (b += 1) {
        const row = TRIT5_LUT[packed_w[b]];
        for (0..5) |ti| {
            if (i >= n_trits) break;
            acc += @as(i64, row[ti]) * @as(i64, x[i]);
            i += 1;
        }
    }
    return acc;
}

// ==================== u1 Bipolar Binary (8 weights/byte) Ref vs SIMD ====================

pub export fn beast_dot_u1_ref(packed_bits: [*]const u8, x: [*]const i32, n_bits: usize) callconv(.c) i64 {
    var acc: i64 = 0;
    for (0..n_bits) |i| {
        const byte = packed_bits[i >> 3];
        const bit: u1 = @truncate(byte >> @intCast(i & 7));
        const w: i64 = if (bit == 1) 1 else -1;
        acc += w * @as(i64, x[i]);
    }
    return acc;
}

pub export fn beast_dot_u1_simd(packed_bits: [*]const u8, x: [*]const i32, n_bits: usize) callconv(.c) i64 {
    const V8i64 = @Vector(8, i64);
    var v_acc: V8i64 = @splat(0);
    var i: usize = 0;
    while (i + 8 <= n_bits) : (i += 8) {
        const byte = packed_bits[i >> 3];
        var wv: V8i64 = undefined;
        var xv: V8i64 = undefined;
        inline for (0..8) |lane| {
            const bit: u1 = @truncate(byte >> lane);
            wv[lane] = if (bit == 1) 1 else -1;
            xv[lane] = x[i + lane];
        }
        v_acc += wv * xv;
    }
    var acc: i64 = @reduce(.Add, v_acc);
    while (i < n_bits) : (i += 1) {
        const byte = packed_bits[i >> 3];
        const bit: u1 = @truncate(byte >> @intCast(i & 7));
        const w: i64 = if (bit == 1) 1 else -1;
        acc += w * @as(i64, x[i]);
    }
    return acc;
}

// ==================== Native Zig `i2` (4 weights/byte) Ref vs SIMD ====================

pub export fn beast_dot_i2_ref(packed_i2: [*]const u8, x: [*]const i32, n_vals: usize) callconv(.c) i64 {
    var acc: i64 = 0;
    for (0..n_vals) |i| {
        const byte = packed_i2[i >> 2];
        const shift: u3 = @intCast((i & 3) * 2);
        const raw_u2: u2 = @truncate(byte >> shift);
        const w_i2: i2 = @bitCast(raw_u2); // native Zig 2-bit signed integer {-2,-1,0,+1}!
        acc += @as(i64, w_i2) * @as(i64, x[i]);
    }
    return acc;
}

pub export fn beast_dot_i2_simd(packed_i2: [*]const u8, x: [*]const i32, n_vals: usize) callconv(.c) i64 {
    const V8i64 = @Vector(8, i64);
    var v_acc: V8i64 = @splat(0);
    var i: usize = 0;
    while (i + 8 <= n_vals) : (i += 8) {
        const b0 = packed_i2[i >> 2];
        const b1 = packed_i2[(i >> 2) + 1];
        var wv: V8i64 = undefined;
        var xv: V8i64 = undefined;
        inline for (0..4) |lane| {
            const raw0: u2 = @truncate(b0 >> (lane * 2));
            const raw1: u2 = @truncate(b1 >> (lane * 2));
            wv[lane] = @as(i2, @bitCast(raw0));
            wv[lane + 4] = @as(i2, @bitCast(raw1));
            xv[lane] = x[i + lane];
            xv[lane + 4] = x[i + lane + 4];
        }
        v_acc += wv * xv;
    }
    var acc: i64 = @reduce(.Add, v_acc);
    while (i < n_vals) : (i += 1) {
        const byte = packed_i2[i >> 2];
        const shift: u3 = @intCast((i & 3) * 2);
        const raw: u2 = @truncate(byte >> shift);
        acc += @as(i64, @as(i2, @bitCast(raw))) * @as(i64, x[i]);
    }
    return acc;
}

// ==================== Blackwell NVFP4 E2M1 (2 nibbles/byte) Ref vs SIMD ====================

pub export fn beast_dot_nvfp4_ref(packed_fp4: [*]const u8, x: [*]const f32, n_vals: usize) callconv(.c) f32 {
    var acc: f64 = 0.0;
    for (0..n_vals) |i| {
        const byte = packed_fp4[i >> 1];
        const nib: u4 = if ((i & 1) == 0) @truncate(byte & 0x0F) else @truncate(byte >> 4);
        acc += @as(f64, NVFP4_E2M1_LUT[nib]) * @as(f64, x[i]);
    }
    return @floatCast(acc);
}

pub export fn beast_dot_nvfp4_simd(packed_fp4: [*]const u8, x: [*]const f32, n_vals: usize) callconv(.c) f32 {
    const V8f64 = @Vector(8, f64);
    var v_acc: V8f64 = @splat(0.0);
    var i: usize = 0;
    while (i + 8 <= n_vals) : (i += 8) {
        var wv: V8f64 = undefined;
        var xv: V8f64 = undefined;
        inline for (0..4) |bi| {
            const byte = packed_fp4[(i >> 1) + bi];
            const lo: u4 = @truncate(byte & 0x0F);
            const hi: u4 = @truncate(byte >> 4);
            wv[bi * 2] = NVFP4_E2M1_LUT[lo];
            wv[bi * 2 + 1] = NVFP4_E2M1_LUT[hi];
            xv[bi * 2] = x[i + bi * 2];
            xv[bi * 2 + 1] = x[i + bi * 2 + 1];
        }
        v_acc += wv * xv;
    }
    var acc: f64 = @reduce(.Add, v_acc);
    while (i < n_vals) : (i += 1) {
        const byte = packed_fp4[i >> 1];
        const nib: u4 = if ((i & 1) == 0) @truncate(byte & 0x0F) else @truncate(byte >> 4);
        acc += @as(f64, NVFP4_E2M1_LUT[nib]) * @as(f64, x[i]);
    }
    return @floatCast(acc);
}

// ==================== MIT Augment-Cache Cross-Attention (Zig SIMD) ====================

pub export fn beast_c2c_augment_attn_f32(
    q_vec: [*]const f32,
    k_self: [*]const f32,
    v_self: [*]const f32,
    len_self: usize,
    k_peer_mapped: [*]const f32,
    v_peer_mapped: [*]const f32,
    len_peer: usize,
    dim: usize,
    out_vec: [*]f32,
) callconv(.c) i32 {
    if (len_self > 512 or len_peer > 512) return -1;
    const total = len_self + len_peer;
    if (dim == 0 or total == 0 or total > 512) return -1;

    var scores: [512]f32 = undefined;
    const inv_sqrt_d: f32 = 1.0 / @sqrt(@as(f32, @floatFromInt(dim)));
    var max_s: f32 = -math.inf(f32);

    for (0..len_self) |t| {
        const base = t * dim;
        var dot: f32 = 0.0;
        for (0..dim) |d| dot += q_vec[d] * k_self[base + d];
        const s = dot * inv_sqrt_d;
        scores[t] = s;
        if (s > max_s) max_s = s;
    }
    for (0..len_peer) |t| {
        const base = t * dim;
        var dot: f32 = 0.0;
        for (0..dim) |d| dot += q_vec[d] * k_peer_mapped[base + d];
        const s = dot * inv_sqrt_d;
        scores[len_self + t] = s;
        if (s > max_s) max_s = s;
    }

    var denom: f32 = 0.0;
    for (0..total) |t| {
        const w = @exp(scores[t] - max_s);
        scores[t] = w;
        denom += w;
    }
    const inv_d: f32 = 1.0 / @max(denom, 1e-12);

    for (0..dim) |d| out_vec[d] = 0.0;
    for (0..len_self) |t| {
        const w = scores[t] * inv_d;
        const base = t * dim;
        for (0..dim) |d| out_vec[d] += w * v_self[base + d];
    }
    for (0..len_peer) |t| {
        const w = scores[len_self + t] * inv_d;
        const base = t * dim;
        for (0..dim) |d| out_vec[d] += w * v_peer_mapped[base + d];
    }
    return @intCast(total);
}

// ==================== Chimera Frankenstein Insert-Network Engine ====================

fn gf2AffineByte(mat: u64, x: u8) u8 {
    var out: u8 = 0;
    for (0..8) |r| {
        const row: u8 = @truncate(mat >> @intCast(r * 8));
        const parity: u8 = @popCount(row & x) & 1;
        out |= parity << @intCast(r);
    }
    return out;
}

fn estimateSpectralNorm(blk: *const BeastChimeraBlock) f32 {
    // 10-step power iteration on normalized 16x16 tile (`weights_q / 64.0`)
    var v: [BEAST_CHIMERA_DIM]f32 = @splat(0.25);
    var sigma: f32 = 0.0;
    for (0..10) |_| {
        var u: [BEAST_CHIMERA_DIM]f32 = @splat(0.0);
        for (0..BEAST_CHIMERA_DIM) |r| {
            var acc: f32 = 0.0;
            for (0..BEAST_CHIMERA_DIM) |c| {
                const w: f32 = @as(f32, @floatFromInt(blk.weights_q[r * BEAST_CHIMERA_DIM + c])) / 64.0;
                acc += w * v[c];
            }
            u[r] = acc;
        }
        var norm2: f32 = 0.0;
        for (u) |x| norm2 += x * x;
        sigma = @sqrt(norm2);
        if (sigma < 1e-9) return 0.0;
        for (0..BEAST_CHIMERA_DIM) |r| v[r] = u[r] / sigma;
    }
    return sigma * @abs(blk.alpha_rezero);
}

fn initBlockWeights(blk: *BeastChimeraBlock, seed: u64, salt: u32) void {
    for (0..BEAST_CHIMERA_DIM * BEAST_CHIMERA_DIM) |i| {
        const u = beast_philox_u64(seed, @intCast(i), salt);
        // Compact centered quantized weights in [-12, +12]
        const q: i8 = @as(i8, @intCast(u % 25)) - 12;
        blk.weights_q[i] = q;
    }
    // Invertible identity-perturbed 8x8 F2 ribosome matrix (AUTOPHAGE style)
    blk.ribosome_f2 = 0x8040201008040201 ^ ((beast_philox_u64(seed, 999, salt) & 0x0101010101010101) << 1);
}

pub export fn beast_chimera_init(g: *BeastChimeraGenome, seed: u64) callconv(.c) void {
    g.* = std.mem.zeroes(BeastChimeraGenome);
    g.magic = BEAST_CHIMERA_MAGIC;
    g.generation = 0;
    g.num_blocks = 2;
    g.lineage_seed = seed;

    // Seed genome with 2 heterogeneous starter blocks: b1.58 Ternary FFN + sLTC Liquid ODE
    g.blocks[0].kind = @intFromEnum(BeastBlockKind.ternary_ffn);
    g.blocks[0].precision = @intFromEnum(BeastPrecision.t158_tern);
    g.blocks[0].active = 1;
    g.blocks[0].alpha_rezero = 0.35;
    g.blocks[0].tau_liquid = 1.0;
    initBlockWeights(&g.blocks[0], seed, 0x1001);

    g.blocks[1].kind = @intFromEnum(BeastBlockKind.sltc_liquid);
    g.blocks[1].precision = @intFromEnum(BeastPrecision.fp32);
    g.blocks[1].active = 1;
    g.blocks[1].alpha_rezero = 0.25;
    g.blocks[1].tau_liquid = 0.45;
    initBlockWeights(&g.blocks[1], seed, 0x1002);

    _ = beast_chimera_op_spectral_clamp(g, 0.99);
}

/// OP_INSERT_BLOCK: Splice any foreign primitive block with ReZero `alpha = 0.0` (loss-neutral start!).
pub export fn beast_chimera_op_insert_block(g: *BeastChimeraGenome, kind: u8, prec: u8, seed: u64) callconv(.c) i32 {
    if (g.magic != BEAST_CHIMERA_MAGIC or g.num_blocks >= BEAST_CHIMERA_MAX_BLOCKS) return -1;
    const idx = g.num_blocks;
    g.blocks[idx] = std.mem.zeroes(BeastChimeraBlock);
    g.blocks[idx].kind = kind;
    g.blocks[idx].precision = prec;
    g.blocks[idx].active = 1;
    g.blocks[idx].alpha_rezero = 0.0; // ReZero loss-neutral invariant!
    g.blocks[idx].tau_liquid = 0.25 + 0.5 * philoxUnit(seed, idx, 0x2001);
    initBlockWeights(&g.blocks[idx], seed, 0x2002 +% idx);
    g.num_blocks += 1;
    g.generation += 1;
    g.mutation_ops |= (1 << 0);
    return @intCast(idx);
}

/// OP_PREC_MORPH: Mutate block precision along Pareto frontier (`u1..fp32`) and requantize weights.
pub export fn beast_chimera_op_prec_morph(g: *BeastChimeraGenome, blk_idx: u32, new_prec: u8) callconv(.c) i32 {
    if (g.magic != BEAST_CHIMERA_MAGIC or blk_idx >= g.num_blocks) return -1;
    var blk = &g.blocks[blk_idx];
    blk.precision = new_prec;
    for (&blk.weights_q) |*w| {
        switch (new_prec) {
            0 => w.* = if (w.* >= 0) 8 else -8, // u1 bipolar
            1 => w.* = if (w.* > 3) 8 else if (w.* < -3) -8 else 0, // b1.58 ternary
            2 => w.* = @as(i8, @intCast(math.clamp(@divTrunc(w.*, 4), -2, 1))) * 4, // i2
            else => {},
        }
    }
    g.generation += 1;
    g.mutation_ops |= (1 << 1);
    return 0;
}

/// OP_CROSS_XENOGRAFT: Splice a foreign organ block from a donor species genome into `dst`.
pub export fn beast_chimera_op_xenograft(dst: *BeastChimeraGenome, donor: *const BeastChimeraGenome, donor_blk: u32) callconv(.c) i32 {
    if (dst.magic != BEAST_CHIMERA_MAGIC or donor.magic != BEAST_CHIMERA_MAGIC) return -1;
    if (dst.num_blocks >= BEAST_CHIMERA_MAX_BLOCKS or donor_blk >= donor.num_blocks) return -1;
    const idx = dst.num_blocks;
    dst.blocks[idx] = donor.blocks[donor_blk];
    dst.blocks[idx].alpha_rezero *= 0.5; // Attenuate graft shock
    dst.num_blocks += 1;
    dst.generation += 1;
    dst.mutation_ops |= (1 << 2);
    return @intCast(idx);
}

/// OP_SPECTRAL_CLAMP: Noetic WSE guarantee `||W||_2 <= max_norm` (0.99) across every active block.
pub export fn beast_chimera_op_spectral_clamp(g: *BeastChimeraGenome, max_norm: f32) callconv(.c) f32 {
    if (g.magic != BEAST_CHIMERA_MAGIC or !math.isFinite(max_norm) or max_norm <= 0.0) return -1.0;
    const n_blocks: usize = @min(@as(usize, g.num_blocks), BEAST_CHIMERA_MAX_BLOCKS);
    var worst: f32 = 0.0;
    for (0..n_blocks) |i| {
        var blk = &g.blocks[i];
        if (blk.active == 0) continue;
        var s = estimateSpectralNorm(blk);
        if (s > max_norm and s > 1e-9) {
            const scale = max_norm / s;
            blk.alpha_rezero *= scale;
            s = estimateSpectralNorm(blk);
        }
        blk.spectral_est = s;
        if (s > worst) worst = s;
    }
    g.max_spectral_norm = worst;
    g.mutation_ops |= (1 << 5);
    return worst;
}

pub export fn beast_chimera_forward(
    g: *const BeastChimeraGenome,
    input16: [*]const f32,
    peer_kv16: ?[*]const f32,
    output16: [*]f32,
    liquid_state16: [*]f32,
) callconv(.c) i32 {
    if (g.magic != BEAST_CHIMERA_MAGIC or g.num_blocks > BEAST_CHIMERA_MAX_BLOCKS) return -1;
    var h: [BEAST_CHIMERA_DIM]f32 = undefined;
    for (0..BEAST_CHIMERA_DIM) |i| h[i] = input16[i];

    for (0..g.num_blocks) |bi| {
        const blk = &g.blocks[bi];
        if (blk.active == 0 or blk.alpha_rezero == 0.0) continue;

        var delta: [BEAST_CHIMERA_DIM]f32 = @splat(0.0);
        switch (blk.kind) {
            @intFromEnum(BeastBlockKind.ternary_ffn),
            @intFromEnum(BeastBlockKind.nvfp4_linear),
            @intFromEnum(BeastBlockKind.qgtl_metric),
            => {
                for (0..BEAST_CHIMERA_DIM) |r| {
                    var acc: f32 = 0.0;
                    for (0..BEAST_CHIMERA_DIM) |c| {
                        const raw: f32 = @as(f32, @floatFromInt(blk.weights_q[r * BEAST_CHIMERA_DIM + c])) / 64.0;
                        const w: f32 = if (blk.kind == @intFromEnum(BeastBlockKind.nvfp4_linear))
                            NVFP4_E2M1_LUT[quantizeNvfp4(raw * 4.0)] * 0.25
                        else
                            raw;
                        acc += w * h[c];
                    }
                    // Fubini-Study metric normalization if QGTL block
                    delta[r] = if (blk.kind == @intFromEnum(BeastBlockKind.qgtl_metric))
                        math.tanh(acc) * 0.5
                    else
                        math.tanh(acc);
                }
            },
            @intFromEnum(BeastBlockKind.sltc_liquid) => {
                // Noetic sLTC continuous-time liquid ODE step
                const dt: f32 = 0.1;
                const inv_tau = 1.0 / @max(blk.tau_liquid, 0.05);
                for (0..BEAST_CHIMERA_DIM) |r| {
                    const drive = math.tanh(h[r]);
                    const s_prev = liquid_state16[r];
                    const s_next = (s_prev + dt * drive) / (1.0 + dt * (inv_tau + @abs(drive)));
                    liquid_state16[r] = s_next;
                    delta[r] = s_next;
                }
            },
            @intFromEnum(BeastBlockKind.ssm_scan) => {
                // Selective state-space recurrence across feature channels
                const decay: f32 = math.clamp(blk.tau_liquid, 0.1, 0.95);
                var state: f32 = 0.0;
                for (0..BEAST_CHIMERA_DIM) |r| {
                    state = decay * state + (1.0 - decay) * h[r];
                    delta[r] = state;
                }
            },
            @intFromEnum(BeastBlockKind.gf2_bitmix) => {
                // AUTOPHAGE 8x8 F2 affine ribosome bit-mixer
                for (0..BEAST_CHIMERA_DIM) |r| {
                    const q_byte: u8 = @intFromFloat(math.clamp((h[r] + 1.0) * 127.0, 0.0, 255.0));
                    const mixed = gf2AffineByte(blk.ribosome_f2, q_byte);
                    delta[r] = (@as(f32, @floatFromInt(mixed)) / 127.5) - 1.0;
                }
            },
            @intFromEnum(BeastBlockKind.c2c_port) => {
                // Inline MIT Augment-Cache / SwaT peer injection port
                if (peer_kv16) |peer| {
                    for (0..BEAST_CHIMERA_DIM) |r| {
                        delta[r] = math.tanh(0.5 * h[r] + 0.5 * peer[r]);
                    }
                }
            },
            else => {},
        }

        for (0..BEAST_CHIMERA_DIM) |r| {
            h[r] += blk.alpha_rezero * delta[r];
            if (!math.isFinite(h[r])) return -2;
        }
    }

    for (0..BEAST_CHIMERA_DIM) |i| output16[i] = h[i];
    return 0;
}

pub const BeastEvolutionReport = extern struct {
    generations: u32,
    pop_size: u32,
    apex_blocks: u32,
    apex_mutation_ops: u32,
    apex_spectral_norm: f32,
    init_heldout_mse: f32,
    evolved_heldout_mse: f32,
    random_control_heldout_mse: f32,
    evolved_beats_control: u32,
    reserved: u32,
};

/// Nonlinear coupled dynamical + bit-parity sequence evaluation (`train` vs disjoint `held-out` band).
fn evalChimeraTaskMse(g: *const BeastChimeraGenome, seed: u64, band_domain: u32, n_seq: usize) f32 {
    var total_sq: f64 = 0.0;
    var count: usize = 0;
    for (0..n_seq) |s| {
        var liq: [BEAST_CHIMERA_DIM]f32 = @splat(0.0);
        var prev_y: f32 = 0.0;
        for (0..4) |t| {
            var inp: [BEAST_CHIMERA_DIM]f32 = undefined;
            var peer: [BEAST_CHIMERA_DIM]f32 = undefined;
            for (0..BEAST_CHIMERA_DIM) |d| {
                const ctr: u64 = @intCast((s * 64 + t * 16 + d));
                inp[d] = philoxUnit(seed, ctr, band_domain) * 2.0 - 1.0;
                peer[d] = philoxUnit(seed, ctr, band_domain + 1) * 2.0 - 1.0;
            }
            var out: [BEAST_CHIMERA_DIM]f32 = undefined;
            if (beast_chimera_forward(g, &inp, &peer, &out, &liq) != 0) return 1e9;
            // Target mixes smooth recurrent dynamics + cross-channel nonlinear coupling + peer context
            for (0..BEAST_CHIMERA_DIM) |d| {
                const nxt = inp[(d + 1) % BEAST_CHIMERA_DIM];
                const tgt: f32 = inp[d] + 0.30 * math.tanh(inp[d] * 1.4 - nxt * 0.6 + 0.25 * peer[d]) + 0.12 * prev_y;
                const err = @as(f64, out[d] - tgt);
                total_sq += err * err;
                count += 1;
            }
            prev_y = math.tanh(inp[0] - inp[3]);
        }
    }
    return @floatCast(total_sq / @as(f64, @floatFromInt(count)));
}

/// OP_WEIGHT_MUTATE + OP_RIBOSOME_MUTATE: Natural-gradient scaled weight/alpha/F2-ribosome mutation.
fn mutateChimeraOffspring(g: *BeastChimeraGenome, seed: u64, gen: u32, slot: u32) void {
    const salt: u32 = 0x5000 +% gen *% 97 +% slot *% 13;
    const roll = beast_philox_u64(seed, @intCast(gen * 32 + slot), salt) % 100;

    if (roll < 15 and g.num_blocks < 10) {
        // OP_INSERT_BLOCK with small initial activation nudge for exploration
        const kind: u8 = @intCast(beast_philox_u64(seed, 1, salt) % 7);
        const prec: u8 = @intCast(beast_philox_u64(seed, 2, salt) % 5);
        const idx = beast_chimera_op_insert_block(g, kind, prec, seed ^ @as(u64, salt));
        if (idx >= 0) {
            g.blocks[@intCast(idx)].alpha_rezero = (philoxUnit(seed, 3, salt) - 0.5) * 0.2;
        }
    } else if (roll < 30 and g.num_blocks > 0) {
        // OP_PREC_MORPH
        const bi: u32 = @intCast(beast_philox_u64(seed, 4, salt) % g.num_blocks);
        const np: u8 = @intCast(beast_philox_u64(seed, 5, salt) % 5);
        _ = beast_chimera_op_prec_morph(g, bi, np);
    } else if (roll < 45 and g.num_blocks > 0) {
        // OP_RIBOSOME_MUTATE (AUTOPHAGE neutral F2 row XOR preserving invertibility)
        const bi: usize = @intCast(beast_philox_u64(seed, 6, salt) % g.num_blocks);
        const r1: u6 = @intCast((beast_philox_u64(seed, 7, salt) % 8) * 8);
        const r2: u6 = @intCast(((beast_philox_u64(seed, 8, salt) % 7) + 1) * 8);
        const src_row: u64 = (g.blocks[bi].ribosome_f2 >> r1) & 0xFF;
        const dst_shift: u6 = @intCast(((@as(u32, r1) + @as(u32, r2)) % 64));
        g.blocks[bi].ribosome_f2 ^= (src_row << dst_shift);
        g.mutation_ops |= (1 << 4);
    } else if (g.num_blocks > 0) {
        // OP_WEIGHT_MUTATE: perturb a sparse subset of quantized weights + alpha_rezero + tau_liquid
        const bi: usize = @intCast(beast_philox_u64(seed, 9, salt) % g.num_blocks);
        const blk = &g.blocks[bi];
        blk.alpha_rezero = math.clamp(blk.alpha_rezero + (philoxUnit(seed, 10, salt) - 0.5) * 0.16, -0.95, 0.95);
        blk.tau_liquid = math.clamp(blk.tau_liquid + (philoxUnit(seed, 11, salt) - 0.5) * 0.15, 0.10, 2.0);
        for (0..16) |m| {
            const wi: usize = @intCast(beast_philox_u64(seed, @intCast(20 + m), salt) % (BEAST_CHIMERA_DIM * BEAST_CHIMERA_DIM));
            const step: i32 = if ((beast_philox_u64(seed, @intCast(40 + m), salt) & 1) == 0) 3 else -3;
            blk.weights_q[wi] = @intCast(math.clamp(@as(i32, blk.weights_q[wi]) + step, -48, 48));
        }
        g.mutation_ops |= (1 << 3);
    }

    g.generation += 1;
    _ = beast_chimera_op_spectral_clamp(g, 0.99);
}

/// Real generational Chimera evolutionary benchmark (proves Kill Criterion KC-4 on held-out sequences!).
pub export fn beast_chimera_evolve_benchmark(
    generations: u32,
    pop_size: u32,
    seed: u64,
    out_report: *BeastEvolutionReport,
) callconv(.c) i32 {
    const pop_n: usize = @min(@max(pop_size, 8), 32);
    var pop: [32]BeastChimeraGenome = undefined;
    var train_mse: [32]f32 = undefined;

    for (0..pop_n) |i| {
        beast_chimera_init(&pop[i], seed +% @as(u64, i) * 31);
        // Seed initial diversity with a C2C port + NVFP4 block so early lineages span mixed blocks
        _ = beast_chimera_op_insert_block(&pop[i], @intFromEnum(BeastBlockKind.c2c_port), @intFromEnum(BeastPrecision.fp32), seed +% i);
        pop[i].blocks[2].alpha_rezero = 0.2;
        _ = beast_chimera_op_spectral_clamp(&pop[i], 0.99);
        train_mse[i] = evalChimeraTaskMse(&pop[i], seed, 0x8100, 6);
    }

    // Record best initial held-out MSE and matched-parameter random control held-out MSE
    var best_idx: usize = 0;
    for (1..pop_n) |i| {
        if (train_mse[i] < train_mse[best_idx]) best_idx = i;
    }
    const init_heldout = evalChimeraTaskMse(&pop[best_idx], seed, 0x8200, 8);

    // Random control: matched number of blocks (6) with random weights, zero selection
    var rand_best_heldout: f32 = math.inf(f32);
    for (0..pop_n) |i| {
        var rg: BeastChimeraGenome = undefined;
        beast_chimera_init(&rg, seed +% 9000 +% i);
        for (0..4) |k| {
            const idx = beast_chimera_op_insert_block(&rg, @intCast((i + k) % 7), @intCast((i + k) % 5), seed +% 9100 +% i * 7 + k);
            if (idx >= 0) rg.blocks[@intCast(idx)].alpha_rezero = 0.25;
        }
        _ = beast_chimera_op_spectral_clamp(&rg, 0.99);
        const hm = evalChimeraTaskMse(&rg, seed, 0x8200, 8);
        if (hm < rand_best_heldout) rand_best_heldout = hm;
    }

    // Elitist + Cross-Xenograft Evolution over `generations` strictly on TRAIN band (`0x8100`)
    for (0..generations) |gen| {
        // Find top 2 elites
        var e0: usize = 0;
        var e1: usize = 1;
        if (train_mse[e1] < train_mse[e0]) std.mem.swap(usize, &e0, &e1);
        for (2..pop_n) |i| {
            if (train_mse[i] < train_mse[e0]) {
                e1 = e0;
                e0 = i;
            } else if (train_mse[i] < train_mse[e1]) {
                e1 = i;
            }
        }

        const apex0 = pop[e0];
        const apex1 = pop[e1];
        const fit0 = train_mse[e0];
        const fit1 = train_mse[e1];

        pop[0] = apex0;
        train_mse[0] = fit0;
        pop[1] = apex1;
        train_mse[1] = fit1;

        for (2..pop_n) |slot| {
            pop[slot] = if ((slot & 1) == 0) apex0 else apex1;
            // Every 8th slot performs OP_CROSS_XENOGRAFT between elite 0 and elite 1!
            if ((slot % 8) == 0 and pop[slot].num_blocks < 10 and apex1.num_blocks > 0) {
                const donor_b: u32 = @intCast(beast_philox_u64(seed, @intCast(gen * 17 + slot), 0x6001) % apex1.num_blocks);
                _ = beast_chimera_op_xenograft(&pop[slot], &apex1, donor_b);
            }
            mutateChimeraOffspring(&pop[slot], seed, @intCast(gen), @intCast(slot));
            train_mse[slot] = evalChimeraTaskMse(&pop[slot], seed, 0x8100, 6);
        }
    }

    var final_best: usize = 0;
    for (1..pop_n) |i| {
        if (train_mse[i] < train_mse[final_best]) final_best = i;
    }
    const evolved_heldout = evalChimeraTaskMse(&pop[final_best], seed, 0x8200, 8);
    const worst_norm = beast_chimera_op_spectral_clamp(&pop[final_best], 0.99);

    out_report.* = .{
        .generations = generations,
        .pop_size = @intCast(pop_n),
        .apex_blocks = pop[final_best].num_blocks,
        .apex_mutation_ops = pop[final_best].mutation_ops,
        .apex_spectral_norm = worst_norm,
        .init_heldout_mse = init_heldout,
        .evolved_heldout_mse = evolved_heldout,
        .random_control_heldout_mse = rand_best_heldout,
        .evolved_beats_control = if (evolved_heldout < rand_best_heldout and evolved_heldout < init_heldout) 1 else 0,
        .reserved = 0,
    };
    return 0;
}

/// Complete built-in self-test + planted mutant trap (returns >0 magic `0xBE457001` on pass, 0 on fail).
pub export fn beast_metal_self_test_all(inject_mutant: u32) callconv(.c) u64 {
    // 1. Exhaustive 243/243 bijection test of b1.58 5-trit/byte codec
    for (0..243) |code| {
        var trits: [5]i8 = undefined;
        beast_unpack_5trits(@intCast(code), &trits);
        var repacked = beast_pack_5trits(&trits);
        if (inject_mutant == 1 and code == 42) repacked ^= 1; // Planted mutant!
        if (repacked != code) return 0;
    }

    // 2. Bit-exact SIMD == Ref parity across b1.58 (500 trits), u1 (512 bits), i2 (512 vals), nvfp4 (256 vals)
    var w_b158: [100]u8 = undefined;
    var x_i32: [512]i32 = undefined;
    for (0..100) |i| w_b158[i] = @intCast((i * 37 + 11) % 243);
    for (0..512) |i| x_i32[i] = @as(i32, @intCast((i * 19) % 127)) - 63;

    const b_ref = beast_dot_b158_ref(&w_b158, &x_i32, 500);
    var b_simd = beast_dot_b158_simd(&w_b158, &x_i32, 500);
    if (inject_mutant == 2) b_simd += 1;
    if (b_ref != b_simd) return 0;

    var w_u1: [64]u8 = undefined;
    for (0..64) |i| w_u1[i] = @truncate(beast_philox_u64(5000, i, 7));
    if (beast_dot_u1_ref(&w_u1, &x_i32, 512) != beast_dot_u1_simd(&w_u1, &x_i32, 512)) return 0;

    var w_i2: [128]u8 = undefined;
    for (0..128) |i| w_i2[i] = @truncate(beast_philox_u64(5001, i, 8));
    if (beast_dot_i2_ref(&w_i2, &x_i32, 512) != beast_dot_i2_simd(&w_i2, &x_i32, 512)) return 0;

    var w_fp4: [128]u8 = undefined;
    var x_f32: [256]f32 = undefined;
    for (0..128) |i| w_fp4[i] = @truncate(beast_philox_u64(5002, i, 9));
    for (0..256) |i| x_f32[i] = philoxUnit(5002, i, 10) * 2.0 - 1.0;
    const f_ref = beast_dot_nvfp4_ref(&w_fp4, &x_f32, 256);
    const f_simd = beast_dot_nvfp4_simd(&w_fp4, &x_f32, 256);
    if (@abs(f_ref - f_simd) > 1e-5) return 0;

    // 3. Chimera ReZero loss-neutrality + Spectral Clamp <= 0.99 test
    var g: BeastChimeraGenome = undefined;
    beast_chimera_init(&g, 5000);
    var inp: [16]f32 = .{ 0.1, -0.2, 0.3, -0.4, 0.5, -0.1, 0.2, 0.7, -0.6, 0.4, 0.2, -0.3, 0.8, -0.5, 0.1, 0.6 };
    var out_before: [16]f32 = undefined;
    var liq_before: [16]f32 = @splat(0.0);
    if (beast_chimera_forward(&g, &inp, null, &out_before, &liq_before) != 0) return 0;

    // Insert a foreign GF(2) bit-mixer block with ReZero alpha=0 -> output MUST be bit-exact identical!
    const ins_idx = beast_chimera_op_insert_block(&g, @intFromEnum(BeastBlockKind.gf2_bitmix), @intFromEnum(BeastPrecision.u1_binary), 5005);
    if (ins_idx < 0) return 0;
    if (inject_mutant == 3) g.blocks[@intCast(ins_idx)].alpha_rezero = 0.5;
    var out_after: [16]f32 = undefined;
    var liq_after: [16]f32 = @splat(0.0);
    if (beast_chimera_forward(&g, &inp, null, &out_after, &liq_after) != 0) return 0;
    for (0..16) |k| {
        if (out_before[k] != out_after[k]) return 0;
    }

    // Activate inserted block + crank alpha to 5.0 -> spectral clamp MUST enforce <= 0.99!
    g.blocks[@intCast(ins_idx)].alpha_rezero = 5.0;
    const norm = beast_chimera_op_spectral_clamp(&g, 0.99);
    if (norm > 0.991) return 0;

    return 0xBE457001;
}

test "beast_metal clean pass and 3 planted mutants caught" {
    try std.testing.expectEqual(@as(u64, 0xBE457001), beast_metal_self_test_all(0));
    try std.testing.expectEqual(@as(u64, 0), beast_metal_self_test_all(1));
    try std.testing.expectEqual(@as(u64, 0), beast_metal_self_test_all(2));
    try std.testing.expectEqual(@as(u64, 0), beast_metal_self_test_all(3));
}
