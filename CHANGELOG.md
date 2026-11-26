# Changelog (`A07`)

## [0.5.0] - 2026-11-25 — THE MATRIX SINGULARITY
### Added
- **2 More Compiled Upstream Tsotchke Win64 DLLs (`11 / 11` Live Native DLL Ecosystem!)**:
  - `c_bridges/tsotchke_moonlab_mlkem_bridge.c` → `bin/tsotchke_moonlab_mlkem_bridge.dll`: compiles upstream `tsotchke/moonlab` **FIPS-203 ML-KEM-512/768/1024** (`mlkem.c` + `poly.c` + `sha3.c`) via `zig cc`, verifying live ML-KEM-768 KeyGen/Sec7.2+7.3/Encaps/Decaps + IND-CCA2 Fujisaki-Okamoto 1-bit ciphertext tamper rejection (`G21`).
  - `c_bridges/tsotchke_sbnn_bridge.c` → `bin/tsotchke_sbnn_bridge.dll`: compiles upstream `tsotchke/spin_based_neural_network` (`fibonacci_anyons.c` + `pbit.c`), verifying golden-ratio Fibonacci non-Abelian anyons (`φ=1.618034`, Yang-Baxter `B₁B₂B₁ == B₂B₁B₂` `err=2.43e-16`, `dim₈=13`) + Spintronic `p-bit` Ising MaxCut annealing (`E=-10.500`, `G22`).
- **Genuine Win32 OS Kernel Job Object Hard Sandbox** (`crates/beast-root/src/win32_job.rs`): direct zero-crate FFI to `kernel32.dll` (`CreateJobObjectW`, `SetInformationJobObject(128 MiB + KILL_ON_JOB_CLOSE)`, `AssignProcessToJobObject`, `IsProcessInJob`), enforcing OS-kernel memory/lifetime limits on Agent-Zone child PIDs alongside `BEAST_API_KEY` scrubbing (`G23`).
- **Real 4-Layer Autoregressive GQA Transformer → Heterogeneous `sLTC/b1.58` Chimera Live C2C LLM** (`crates/beast-model/src/llm_engine.rs`, `beast llm-c2c`): bridges all 4 live RoPE/RMSNorm/SwiGLU KV layers across architectures (`64d/32kv → 48d/24kv`, zero text prefill, `swat_cos=0.9998`, `142,496 B` GGUF v3 export, `G24`).
- **Autonomous Bare-Metal A-Life Organism Forge** (`crates/beast-agent/src/alife_forge.rs`, `beast forge-alife`): completes the Bare-Metal Trinity Forge (`OS Kernel` + `Chimera Model` + `Standalone Zig A-Life .exe`), verifying `0`-residual exact integer energy conservation, Observer-Independent P9 determinism (`telemetry=1 == telemetry=0`), and Planaria 50%-shred `.exe` binary self-healing (`G25`, integrated as Step 4 of `beast agent-run` in `G20`).
- **50 Pure Lean 4 `v4.35.0` Theorems** (`lean/BeastProofs.lean`, `0 sorry`, `0` custom axioms, `0` warnings) adding `B40`–`B50` (FIPS-203 ML-KEM-768 sizes & NTT prime `3329`, Fibonacci growth, Win32 Job Object limits, 4-layer RoPE pairs, exact A-Life energy conservation, P9 observer independence, 11-DLL unity, 30-mutant total, Trinity Forge receipts, and live `v0.5.0` receipt `B50`).
- **25/25 Gates · 30/30 Total Planted Mutants · 18/18 Rust Unit Tests · `284 / 300` (`94.7%`) Matrix Items Live-Verified**.

## [0.4.0] - 2026-11-25 — THE AUTONOMOUS MATRIX FORGE
### Added
- Live QEMU `11.1.0` Ring-0 bootable `i386` Multiboot1 AI microkernel (`F-08` closed: `B158=7/7`, `NVFP4=59/59`, `BIT_EXACT=true`), compiled upstream `tsotchke/libirrep` (AVX2+FMA SO(3) Unsöld) & `tsotchke/moonlab` (QGT Chern/SSH/Kitaev) Win64 DLLs (`9/9` DLLs), 8th Rust crate `crates/beast-agent` (Merkle AST Repo-Map + Autonomous Kernel/Model Forge + Agent Loop), `20/20` gates, `25/25` mutants, and `40/40` Pure Lean 4 theorems.

## [0.3.0] - 2026-11-25
### Added
- 7th crate `crates/beast-model` (GGUF v3, cross-arch C2C + Planaria heal, 16 kHz Formant TTS/ASR `.wav` + noisy 2D Sobel/`b1.58` OCR `.pgm`), live OS-process Ring-0 sandbox handoff, interactive `beast repl`, `7/7` native DLLs (`+tsotchke/PINN`), ELF64/Multiboot2 auditor, `32` Pure Lean 4 theorems.

## [0.2.0] - 2026-11-25
### Added
- 3-kernel RTX 5070 Ti Blackwell PTX suite, analytical-gradient MIT `SwaT` trainer + `.bcg` `CacheGraft`, 120-gen Zig Chimera breeder (`KC-4`), Planaria GF($2^8$)+Merkle capsule, Multi-Colony Civilization OS + live `AUTOPHAGE` RWX JIT, upstream `tsotchke/quantum_rng` Bell DLL, Public OSS API tool, freestanding Microkernel v2, 10-gen CRuby/YARV breeder, `24` Pure Lean 4 theorems.

## [0.1.0] - 2026-11-25
### Added
- Initial `beast-civilization` polyglot monorepo (`Zig 0.17.1`, `Rust 1.94.1`, `CRuby 3.4.9/YARV`, `C11/AUTOPHAGE`, `CUDA sm_120 PTX`, `Pure Lean 4 v4.35.0`), Build Standard v1.2.0 core, and `300`-item agent matrix.
