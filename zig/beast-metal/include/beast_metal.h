#ifndef BEAST_METAL_H
#define BEAST_METAL_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define BEAST_METAL_ABI_VERSION 0x00010000u

/* Precision / Quantization tags for arbitrary mixed-precision tensors */
typedef enum BeastPrecision {
    BEAST_PREC_U1_BINARY   = 0, /* 1-bit bipolar {-1, +1} packed 8 per byte */
    BEAST_PREC_T158_TERN   = 1, /* b1.58 ternary {-1, 0, +1} packed 5 trits per byte (3^5=243 <= 256) */
    BEAST_PREC_I2_INT2     = 2, /* 2-bit signed {-2, -1, 0, +1} packed 4 per byte */
    BEAST_PREC_U3_UINT3    = 3, /* 3-bit centered {-3..+4} packed bit-stream */
    BEAST_PREC_NVFP4_E2M1  = 4, /* Blackwell native FP4 E2M1 (16 values, 2 per byte) */
    BEAST_PREC_FP8_E4M3    = 5, /* FP8 E4M3fn (1 byte per weight) */
    BEAST_PREC_BF16        = 6, /* Brain Float 16 */
    BEAST_PREC_FP32        = 7  /* IEEE-754 float32 */
} BeastPrecision;

/* Chimera Insert-Network primitive block kinds */
typedef enum BeastBlockKind {
    BEAST_BLK_TERNARY_FFN  = 0, /* b1.58 5-trit/byte ternary FFN */
    BEAST_BLK_NVFP4_LINEAR = 1, /* Blackwell E2M1 FP4 linear projection */
    BEAST_BLK_SLTC_LIQUID  = 2, /* Continuous-time liquid ODE neuron (Noetic sLTC) */
    BEAST_BLK_SSM_SCAN     = 3, /* Linear-time Mamba-style selective state-space scan */
    BEAST_BLK_GF2_BITMIX   = 4, /* AUTOPHAGE 8x8 F2 affine bit-mixer */
    BEAST_BLK_QGTL_METRIC  = 5, /* Fubini-Study natural-gradient metric projection */
    BEAST_BLK_C2C_PORT     = 6  /* MIT Augment-Cache / SwaT cross-attention graft port */
} BeastBlockKind;

/* Compact Chimera Insert-Network Genome (flat POD, zero pointers, 64B aligned) */
#define BEAST_CHIMERA_MAX_BLOCKS 16
#define BEAST_CHIMERA_DIM        16

typedef struct BeastChimeraBlock {
    uint8_t  kind;         /* BeastBlockKind */
    uint8_t  precision;    /* BeastPrecision */
    uint8_t  active;       /* 1 = active, 0 = dormant/spliced out */
    uint8_t  reserved;
    float    alpha_rezero; /* ReZero residual gate (starts at 0.0 on OP_INSERT_BLOCK!) */
    float    tau_liquid;   /* Adaptive time constant for sLTC / SSM decay */
    float    spectral_est; /* Power-iteration ||W||_2 estimate (enforced <= 0.99) */
    uint64_t ribosome_f2;  /* 8x8 F2 affine matrix for GF(2) bit-mixer */
    int8_t   weights_q[BEAST_CHIMERA_DIM * BEAST_CHIMERA_DIM]; /* quantized tile */
} BeastChimeraBlock;

typedef struct BeastChimeraGenome {
    uint32_t magic;        /* 0x42434849 "BCHI" */
    uint32_t generation;
    uint32_t num_blocks;
    uint32_t mutation_ops; /* bitmask of applied operators */
    uint64_t lineage_seed;
    float    max_spectral_norm;
    float    fitness_score;
    BeastChimeraBlock blocks[BEAST_CHIMERA_MAX_BLOCKS];
} BeastChimeraGenome;

/* ABI & Version */
uint32_t beast_metal_abi_version(void);

/* Deterministic Philox4x32-10 PRNG (Asmodensius P4 compatible) */
uint64_t beast_philox_u64(uint64_t key, uint64_t counter, uint32_t domain_id);

/* b1.58 5-Trit/Byte Packing (3^5 = 243 <= 256, proven bijective in Lean 4) */
uint8_t  beast_pack_5trits(const int8_t trits5[5]);
void     beast_unpack_5trits(uint8_t packed, int8_t out_trits5[5]);

/* NVFP4 E2M1 exact codec (2 nibbles per byte) */
uint8_t  beast_pack_nvfp4_pair(float a, float b);
void     beast_unpack_nvfp4_pair(uint8_t packed, float out_pair[2]);

/* Bit-Exact Reference vs. SIMD Vectorized Mixed-Precision Dot Products */
int64_t  beast_dot_b158_ref(const uint8_t* packed_w, const int32_t* x, size_t n_trits);
int64_t  beast_dot_b158_simd(const uint8_t* packed_w, const int32_t* x, size_t n_trits);

int64_t  beast_dot_u1_ref(const uint8_t* packed_bits, const int32_t* x, size_t n_bits);
int64_t  beast_dot_u1_simd(const uint8_t* packed_bits, const int32_t* x, size_t n_bits);

int64_t  beast_dot_i2_ref(const uint8_t* packed_i2, const int32_t* x, size_t n_vals);
int64_t  beast_dot_i2_simd(const uint8_t* packed_i2, const int32_t* x, size_t n_vals);

float    beast_dot_nvfp4_ref(const uint8_t* packed_fp4, const float* x, size_t n_vals);
float    beast_dot_nvfp4_simd(const uint8_t* packed_fp4, const float* x, size_t n_vals);

/* C2C Augment-Cache Cross-Attention Kernel (MIT Augment-Cache law: preserves self KV!) */
/* Computes Attention(Q_adapted, [K_self; K_peer], [V_self; V_peer]) in Zig SIMD */
int32_t  beast_c2c_augment_attn_f32(
    const float* q_vec,          /* [dim] */
    const float* k_self,         /* [len_self * dim] */
    const float* v_self,         /* [len_self * dim] */
    size_t       len_self,
    const float* k_peer_mapped,  /* [len_peer * dim] */
    const float* v_peer_mapped,  /* [len_peer * dim] */
    size_t       len_peer,
    size_t       dim,
    float*       out_vec         /* [dim] */
);

/* Chimera Insert-Network Evolution & Execution */
void     beast_chimera_init(BeastChimeraGenome* g, uint64_t seed);
int32_t  beast_chimera_op_insert_block(BeastChimeraGenome* g, uint8_t kind, uint8_t prec, uint64_t seed);
int32_t  beast_chimera_op_prec_morph(BeastChimeraGenome* g, uint32_t blk_idx, uint8_t new_prec);
int32_t  beast_chimera_op_xenograft(BeastChimeraGenome* dst, const BeastChimeraGenome* donor, uint32_t donor_blk);
float    beast_chimera_op_spectral_clamp(BeastChimeraGenome* g, float max_norm);
int32_t  beast_chimera_forward(
    const BeastChimeraGenome* g,
    const float* input16,
    const float* peer_kv16, /* optional C2C peer vector for BEAST_BLK_C2C_PORT */
    float*       output16,
    float*       liquid_state16
);

/* Benchmark helper returning nanoseconds per 10,000 b1.58 1024-dim dot products */
uint64_t beast_metal_self_test_all(uint32_t inject_mutant);

#ifdef __cplusplus
}
#endif

#endif /* BEAST_METAL_H */
