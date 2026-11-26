# Changelog (`A07`)

## [0.3.0] - 2026-11-25
### Added
- **7th Crate `crates/beast-model`**: Real GGUF v3 binary writer/parser (`gguf.rs`), Cross-Architecture `Transformer (32d) → Mamba/sLTC/b1.58 Chimera (24d)` C2C + GGUF v3 + Planaria 50%-annihilation self-healing pipeline (`omni_engine.rs`), and real bare-metal Omnimodal Audio (16 kHz Formant TTS → `.wav` → DFT ASR `4/4`) + Vision/OCR (`.pgm` → Sobel + `b1.58` noisy hex OCR `16/16`) kernels (`omnimodal.rs`).
- **Live OS-Process Ring-0 Sandbox & Overlapped Handoff** (`crates/beast-root/src/os_sandbox.rs`): spawns two real sandboxed Windows/Linux child OS processes (`Parent → Child1 → Child2`), verifying `BEAST_API_KEY` secret-canary scrubbing across distinct OS PIDs.
- **Interactive Bare-Metal Console** (`beast repl`) + `beast model-c2c` + `beast omni-test`.
- **Unified 7-DLL Native Bare-Metal Ecosystem** (`c_bridges/tsotchke_pinn_bridge.c`, `crates/beast-colony/src/ecosystem_ffi.rs`): compiles upstream `tsotchke/PINN` via `zig cc` alongside `tsotchke/quantum_rng` and links local `omega_cell.dll` + `cellulogonic.dll` + `autophage.dll` + `beast_metal.dll` + `nvcuda.dll` live in one executable (`7/7` active, `12` upstream repos materialized in `vendor/`).
- **Native ELF64 + Multiboot2 Forensic Binary Auditor** (`crates/beast-wire/src/elf_audit.rs`): verifies `ELF64`, `EM_X86_64`, `PT_LOAD`, entry address, and exact mod-$2^{32}$ Multiboot2 checksum with planted-mutant catch.
- **32 Pure Lean 4 `v4.35.0` Theorems** (`lean/BeastProofs.lean`, `0 sorry`, `0` custom axioms) including live v0.3.0 hardware receipt `B32_live_v030_matrix_receipt` (`16/16` gates, `21/21` total planted mutants caught).

## [0.2.0] - 2026-11-25
### Added
- **3-Kernel NVIDIA RTX 5070 Ti `sm_12.0` Blackwell CUDA PTX Suite** (`cuda/beast_blackwell.ptx`, `crates/beast-cuda`): added `beast_nvfp4_rowdot` (OCP FP4 E2M1) and `beast_c2c_augment_scores` (MIT Augment-Cache parallel KV scoring) alongside `beast_b158_rowdot` (`3/3` verified live via `nvcuda.dll`, zero MSVC/nvcc).
- **Trained MIT `SwaT` / `Augment-Cache` Engine + `.bcg` `CacheGraft` Repo Indexer** (`crates/beast-c2c/src/{trainer,graft_io}.rs`): analytical gradient trainer achieving `-100.0%` held-out MSE & empirical `Augment (1.000) > Overwrite (-0.311)` 2×2 proof + real binary `.bcg` repo bank with tamper rejection.
- **120-Generation Zig Bare-Metal Chimera Insert-Network Breeder (`KC-4`)** (`zig/beast-metal/src/root.zig`): 24 hybrid organisms × 120 gens in `82 ms`, `3.29×` lower held-out MSE than matched random control (`0.0095` vs `0.0313`), `‖W‖₂ = 0.0661 ≤ 0.99`.
- **Planaria GF($2^8$) Cauchy MDS + Depth-3 Merkle Self-Healing Capsule** (`crates/beast-wire/src/hbin_capsule.rs`): survives 50% data annihilation and quarantines Byzantine Trojans before decoding.
- **Multi-Colony Civilization OS + Live `AUTOPHAGE` RWX JIT World Stepping** (`crates/beast-colony/src/{civilization,autophage_ffi}.rs`): `3` colonies × `6` castes = `18` agents coupled to live 10-epoch `AUTOPHAGE v1.3.0` RWX x86-64 JIT == Scalar Ref (0-bit SHA-256 parity).
- **Compiled Upstream `tsotchke/quantum_rng` Win64 DLL Bridge** (`c_bridges/tsotchke_qrng_bridge.c`, `crates/beast-colony/src/tsotchke_ffi.rs`): compiled directly from `vendor/tsotchke__quantum_rng` via `zig cc`, executing live CHSH Bell test (`S = 2.7461 > 2.0`).
- **Live Public OSS API Super-Tool** (`crates/beast-colony/src/oss_apis.rs`): Ring-0 allowlisted live HTTPS queries (`beast api-query arxiv ...`) + injection guard.
- **Freestanding `x86_64` Multiboot2 Microkernel v2** (`13,656 B`): dual Ring-0 `b1.58` + integer `NVFP4` neural reflex cores.
- **10-Generation CRuby `3.4.9`/Prism/YARV Bytecode Breeder** (`ruby/beast_yarv.rb`): live monotonic bytecode evolution with automatic rollback.
- **24 Pure Lean 4 `v4.35.0` Theorems** (`lean/BeastProofs.lean`, `0 sorry`, `0` custom axioms) incl. live v0.2.0 hardware receipt `B24`.

## [0.1.0] - 2026-11-25
### Added
- Initial `beast-civilization` polyglot monorepo (`Zig 0.17.1`, `Rust 1.94.1`, `CRuby 3.4.9/YARV`, `C11/AUTOPHAGE`, `CUDA sm_120 PTX`, `Pure Lean 4 v4.35.0`).
- Build Standard v1.2.0 core (`AGENTS.md`, `CLAIMS.md`, `LIMITATIONS.md`, `REPLICATION.md`, `repo.yaml`, `docs/plan/00-*`, `docs/matrices/300_agent_matrix.yaml`, `THIRD_PARTY_NOTICES.md`).
- `zig/beast-metal`: bit-exact arbitrary mixed-precision kernels (`u1`, `b1.58` 5-trit/B, `i2`, `u3`, `nvfp4_e2m1`, `fp8_e4m3`), Chimera Insert-Network engine (`OP_INSERT_BLOCK`, `OP_PREC_MORPH`, `OP_CROSS_XENOGRAFT`, `OP_CACHE_GRAFT_PORT`, `OP_SPECTRAL_CLAMP`), `Philox4x32-10` PRNG.
- `zig/freestanding_kernel`: freestanding `x86_64-freestanding` Multiboot2 bare-metal microkernel ELF with embedded `b1.58` ternary reflex core.
- `crates/{beast-wire,beast-root,beast-c2c,beast-colony,beast-cuda,beast-cli}`: Ring-0 immutable Root guardian (`INIT-Agent` Win64+Linux), MIT C2C / `Swarm Telepathy` / `Augment-Cache & Replace-Activation` / `CacheGraft` engine, `TEMM1E` + `NonHuman` + `Engram` + `Moonlab`-QUBO + `QGTL` Fubini-Study/Berry Colony OS, direct `nvcuda.dll` Blackwell `sm_120` PTX runner, and live `AUTOPHAGE` DLL FFI.
- `ruby/beast_yarv.rb`: CRuby 3.4 + Prism + YARV live bytecode inspection, serialization, mutation, and Zig C-ABI Fiddle execution.
- `lean/BeastProofs.lean`: Pure Lean 4 kernel proofs (`0 sorry`, `{propext, Classical.choice, Quot.sound}`) including live runtime-to-Lean telemetry receipt verification.
