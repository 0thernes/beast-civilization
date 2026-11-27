module

public import BeastProofs.Spec

/-!
# BeastProofs.Quantization — Group I: Arbitrary-Bitwidth Packing & Compression (7 Theorems)

Machine-checked Pure Lean 4.35.0 proofs for BitNet `b1.58`, `NVFP4 E2M1`, `u1`, `i2`,
and standard `llama.cpp` GGUF v3 `Q4_0` compression (`B01`–`B05`, `B61`, `B64`).

Copyright (c) 2026 Alexander Stephen Donahue <0_0@0thernes.art>.
SPDX-License-Identifier: Apache-2.0
-/

namespace BeastProofs.Quantization

public theorem B01_b158_five_trits_per_byte : 3 ^ 5 = 243 ∧ 243 ≤ 256 := by decide

public theorem B02_b158_pack_unpack_exact :
    ∀ (d0 d1 d2 d3 d4 : Fin 3),
      pack5Trits d0 d1 d2 d3 d4 < 243 ∧
      unpack5Trits (pack5Trits d0 d1 d2 d3 d4) = (d0.val, d1.val, d2.val, d3.val, d4.val) := by
  decide

public theorem B03_nvfp4_two_nibbles_per_byte : 16 * 16 = 256 := by decide
public theorem B04_u1_eight_bits_per_byte : 2 ^ 8 = 256 := by decide
public theorem B05_i2_four_vals_per_byte : 4 ^ 4 = 256 := by decide

public theorem B61_gguf_q4_0_block_bytes : 2 + 16 = 18 ∧ 16 * 2 = 32 := by decide

public theorem B64_q4_0_compression_vs_f32 (n : Nat) (hn : n > 0) : 18 * n < 128 * n := by
  omega

end BeastProofs.Quantization
