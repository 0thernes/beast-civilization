<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Glossary (`A44`)

- **Augment-Cache (MIT C2C Law #1):** Non-destructive cross-attention over concatenated `[K_self || K_peer]` (`L_s + L_p`), preserving 100% of the receiver's local KV context while incorporating peer latent states.
- **b1.58 (BitNet Ternary):** Weight representation in `{-1, 0, +1}` ($\log_2 3 \approx 1.585$ bits/weight) packed 5 trits per byte via base-3 encoding ($3^5 = 243 \le 256$).
- **BCG (`.bcg` CacheGraft Bank):** SHA-256-authenticated offline repository KV-cache binary produced by `beast-c2c::graft_io`.
- **Chimera Genome:** Variable-depth (up to 16-block) heterogeneous neural architecture combining `b1.58`, `NVFP4`, `sLTC` liquid ODE, `SSM`, $\mathbb{F}_2$ affine ribosome, `QGTL`, and `C2C` blocks under ReZero ($\alpha_0 = 0$) and spectral norm clamping ($\|W\|_2 \le 0.99$).
- **GGUF v3 (`Q4_0`/`F16`/`F32`):** Standard `llama.cpp`-compatible binary model container with 32-byte aligned tensor data arena; `Q4_0` packs 32 signed 4-bit weights + one IEEE-754 `f16` scale into 18 bytes ($64/9 \approx 7.11\times$ compression vs. `F32`).
- **Holm-Bonferroni FWER:** Step-down family-wise error rate multiple-comparison correction applied across all 4 pre-registered hypotheses (`H1`–`H4`) in `scripts/verify_stats.py`.
- **MCP (Model Context Protocol):** Standardized JSON-RPC 2.0 stdio tool/resource interface exposed by `beast mcp-serve`.
- **NVFP4 (`E2M1`):** 4-bit floating-point format (1 sign, 2 exponent, 1 mantissa bit; 16 representable levels) packed 2 nibbles/byte.
- **Palomar Six-Kernel Contract:** Lean 4.35.0 verification protocol isolating pure definitions (`Spec.lean`) and sorry-stubbed statements (`Challenge.lean`) from sorry-free proofs (`Solution.lean`), cross-checked via `lake check --paranoid` and `lake comparator --paranoid` inside `bubblewrap`.
- **Philox4x32-10:** Counter-based stateless parallel PRNG (Salmon et al., SC '11) implemented bit-identically across Rust and Zig with explicit domain separation.
- **Planaria Capsule:** 8-shard systematic Cauchy MDS erasure code over $\mathrm{GF}(2^8)$ ($K=4$ data + $4$ parity) authenticated by a domain-separated depth-3 SHA-256 Merkle tree; survives 50% shard destruction and quarantines Byzantine Trojan shards before decoding.
- **Planted Mutant:** Deliberately corrupted input, boundary violation, or spoofed artifact embedded in every verification gate to prove the verifier fails loudly on invalid states.
- **Ring-0 Guardian (`beast-root`):** Immutable supervisory core enforcing normalized path ACLs, micro-USD spend caps, secret scrubbing, Win32 Job Objects, and 0-bit cryptographic rollback.
- **Rosette-Wire Receipt:** HMAC-SHA256-authenticated, hash-chained execution record verifying step-by-step agent provenance.
- **SwaT (Swarm Telepathy):** Cross-family linear-affine KV projection mapper with energy-based top-$k$ token compaction (`crates/beast-c2c`).
