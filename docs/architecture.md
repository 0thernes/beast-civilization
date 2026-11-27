<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# System Architecture (`A20`)

`BEAST CIVILIZATION v1.0.0` is a five-plane, zero-third-party-dependency polyglot bare-metal multi-colony AI architecture unifying **Zig 0.17.1**, **Rust 1.94.1** (8 workspace crates), **CRuby 3.4.9 / Prism AST / YARV**, **C11** (7 compiled upstream Tsotchke physics/cryptography bridges + 3 optional local A-Life engines), **NVIDIA Blackwell `sm_12.0` CUDA PTX**, and **Pure Lean 4.35.0** (81 six-kernel Palomar theorems, zero Mathlib).

## Five Architectural Planes

1. **Plane 1 — Formal Verification & Falsification (`lean/`, `scripts/`):**
   - `lean/BeastProofs/Spec.lean` holds pure definitions (`0` proofs).
   - 7 domain modules (`Quantization`, `CacheTelepathy`, `Ring0Security`, `ColonyGovernance`, `CryptoPhysics`, `SystemsRuntime`, `LiveReceipt`) + `NonVacuity.lean` (`NV01`–`NV08`) + `AdversarialTests.lean` (`AT01`–`AT08`) export **81 sorry-free theorems** via `lean/Solution.lean`.
   - `lean/Challenge.lean` isolates statement signatures with `by sorry` (importing only `Spec.lean`) and is strictly matched against `Solution.lean` via `lake comparator --config comparator.json --paranoid` across 6 independent kernels in sandboxed Ubuntu CI.
   - `scripts/verify_independent_oracle.py`, `scripts/prop_check.py`, and `scripts/verify_stats.py` provide clean-room exact-arithmetic, randomized property, and pre-registered $N=24$ Holm-Bonferroni statistical verification.

2. **Plane 2 — Bare-Metal Silicon & Freestanding OS (`zig/`, `cuda/`, `c_bridges/`):**
   - `zig/beast-metal/src/root.zig` exports the `0x00010000` C-ABI (`beast_metal.{dll,so,dylib}`) with bit-exact scalar vs. `@Vector` SIMD `b1.58` (5 trits/byte), `u1`, `i2`, `NVFP4 E2M1`, MIT `Augment-Cache`, and the 16-block Chimera evolutionary breeder.
   - `zig/freestanding_kernel/` cross-compiles zero-`libc` `ELF64` Multiboot2 (`beast_microkernel.elf`) and QEMU-bootable `ELF32` Multiboot1 (`beast_boot32.elf`) neural microkernels.
   - `cuda/beast_blackwell.ptx` defines 5 hand-authored `sm_12.0` PTX kernels (`beast_b158_rowdot`, `beast_nvfp4_rowdot`, `beast_c2c_augment_scores`, `beast_mla_latent_compress`, `beast_moe_swiglu_sparse`) launched directly through `nvcuda.dll` / `libcuda.so.1` with page-locked pinned async host-to-device paging.
   - `c_bridges/` links 7 upstream Tsotchke C11 libraries (QRNG CHSH Bell, PINN, `libirrep` AVX2 SO(3), `moonlab` QGT Chern/SSH/Kitaev, `moonlab` FIPS-203 ML-KEM-768 + FIPS-197 AES-256, `SbNN` Fibonacci anyons + spintronic $p$-bits, `QGTL` Bloch fidelity).

3. **Plane 3 — Wire Trust, Ring-0 Isolation & Colony OS (`crates/beast-wire`, `beast-root`, `beast-colony`, `ruby/`):**
   - `beast-wire`: NIST FIPS 180-4 SHA-256, HMAC-SHA256 Rosette-Wire receipt chains, Philox4x32-10 PRNG (bit-exact with Zig), ELF32/ELF64 Multiboot1/2 auditors, Planaria $\mathrm{GF}(2^8)$ Cauchy MDS + depth-3 Merkle capsules (50% loss survival + Trojan quarantine).
   - `beast-root`: Immutable Ring-0 ACL (case-normalized, prefix-normalized, control-byte-safe), micro-USD spend metering, 0-bit snapshot rollback, multi-PID OS secret scrubbing, and Win32 Kernel Job Objects (`128 MiB`, `KILL_ON_JOB_CLOSE`).
   - `beast-colony`: 3-colony × 6-caste = 18-agent Civilization OS with Moonlab QUBO annealing, TEMM1E Wilson graduation, $\lambda$-memory, NonHuman cost minimization, and allowlisted public OSS API queries.
   - `ruby/beast_yarv.rb`: CRuby 3.4.9 + Prism AST Ring-0 gate (`4/4` injection/dispatch/constant mutants blocked) + 10-generation YARV bytecode breeder + Fiddle C-ABI bridge to Zig.

4. **Plane 4 — Cross-Architecture C2C Telepathy, GGUF v3 & Omnimodal Models (`crates/beast-c2c`, `beast-model`):**
   - `beast-c2c`: Trained cross-family `SwaT` mappers, non-destructive `Augment-Cache` cross-attention, activation replacement, Merkle-signed `.bcg` repository cache banks, and anti-spoof `Bc2cPacketHeader` enforcement.
   - `beast-model`: 4-layer GQA/RoPE Transformer → `sLTC`/`b1.58` Chimera C2C bridge, standard `llama.cpp` GGUF v3 (`Q4_0`/`F16`/`F32`/`BEAST_B158`/`BEAST_NVFP4`) bounded codec, Eshkol tri-mode exact AD (`Forward == Reverse == Symbolic`), deterministic 16 kHz Formant TTS/DFT ASR + noisy 2D Sobel/`b1.58` OCR, and zero-crate OpenAI-compatible HTTP/1.1 inference server (`beast serve`).

5. **Plane 5 — Autonomous Agent Bus, Trinity Forge & CLI (`crates/beast-agent`, `beast-cli`):**
   - Merkle multi-language AST repo-map, cross-repo C-ABI dependency graph, Church-Rosser confluent task reduction, isolated parallel worktrees, MCP JSON-RPC 2.0 stdio server (`beast mcp-serve`), SSRF/CRLF-guarded HTTP/1.1 LLM client, Trinity Forge (`forge-kernel`, `forge-model`, `forge-alife`), interactive/scripted REPL, `stats-n24`, and `verify-all --export-lean`.
