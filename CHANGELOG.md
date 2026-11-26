# Changelog (`A07`)

## [0.4.0] - 2026-11-25 — THE AUTONOMOUS MATRIX FORGE
### Added
- **Live QEMU `11.1.0` Ring-0 Bootable Multiboot1 AI Microkernel (`F-08` CLOSED!)**: Added `zig/freestanding_kernel/src/boot32.zig` + `linker32.ld` (`x86-freestanding-none`, `12,692 B` ELF32) + `audit_elf32_multiboot1` in `crates/beast-wire/src/elf_audit.rs`. Boots directly inside QEMU Ring-0 via `-kernel` + `isa-debug-exit`, executes embedded `b1.58` + integer `NVFP4` neural inference over COM1 (`0x3F8`), and matches Host CPU Oracle bit-for-bit (`B158=7/7`, `NVFP4=59/59`, `BIT_EXACT=true`).
- **2 More Compiled Upstream Tsotchke Win64 DLLs (9 / 9 Live Native DLL Ecosystem!)**:
  - `c_bridges/tsotchke_libirrep_bridge.c` → `bin/tsotchke_libirrep_bridge.dll`: compiles upstream `tsotchke/libirrep` with `-mavx2 -mfma`, verifying $S^2$ orthonormal spherical harmonics Unsöld Addition Theorem (`sum_Y2 = 0.716197 == 9/(4π)`) and AVX2==Scalar batch parity.
  - `c_bridges/tsotchke_moonlab_qgt_bridge.c` → `bin/tsotchke_moonlab_qgt_bridge.dll`: compiles upstream `tsotchke/moonlab` (`qgt.c` + `matrix_math.c`), verifying Qi-Wu-Zhang Chern Insulator (`C_topo = 1`, `C_triv = 0`), SSH 1D Winding (`W = 1`), Kitaev Majorana chain (`Z₂ = 1`), and Provost-Vallée Fubini-Study metric + Quantum Geometric Bound (`Tr(g)=0.4685`, `Ω=0.4465`).
- **8th Zero-Dependency Rust Crate `crates/beast-agent`**:
  - `repo_map.rs`: Deterministic SHA-256 Merkle Multi-Language AST/Symbol Repo-Map (`44` files / `246` symbols, `G18`).
  - `forge.rs`: `beast forge-kernel` & `beast forge-model` — autonomously synthesizes, compiles, ELF32-audits, live-boots in QEMU Ring-0, and Planaria-heals custom neural OS microkernels + Chimera GGUF v3 / `.bcg` models (`G19`).
  - `loop_engine.rs`: Unified `INIT`×`NonHuman`×`Engram`×`TEMM1E` autonomous bare-metal agent loop (`beast agent-run`) with signed Rosette-Wire receipt chains and Ring-0 prompt guard (`G20`).
- **40 Pure Lean 4 `v4.35.0` Theorems** (`lean/BeastProofs.lean`, `0 sorry`, `0` custom axioms, `0` warnings) adding `B32`–`B40` (Multiboot1 checksum mod $2^{32}$, spherical harmonic dimension, Moonlab topological invariants, Quantum Geometric Bound PSD proof, 9-DLL unity, Merkle doubling, forge name bounds, 25-mutant total, and live `v0.4.0` receipt `B40`).
- **20/20 Gates & 25/25 Total Planted Mutants** (`3` Zig + `20` Rust/CLI + `2` CRuby/Prism), `17/17` Rust unit tests, `280/300` (`93.3%`) matrix items live-verified.

## [0.3.0] - 2026-11-25
### Added
- **7th Crate `crates/beast-model`**: Real GGUF v3 binary writer/parser (`gguf.rs`), Cross-Architecture `Transformer (32d) → Mamba/sLTC/b1.58 Chimera (24d)` C2C + GGUF v3 + Planaria 50%-annihilation self-healing pipeline (`omni_engine.rs`), and real bare-metal Omnimodal Audio (16 kHz Formant TTS → `.wav` → DFT ASR `4/4`) + Vision/OCR (`.pgm` → Sobel + `b1.58` noisy hex OCR `16/16`) kernels (`omnimodal.rs`).
- **Live OS-Process Ring-0 Sandbox & Overlapped Handoff** (`crates/beast-root/src/os_sandbox.rs`): spawns two real sandboxed Windows/Linux child OS processes (`Parent → Child1 → Child2`), verifying `BEAST_API_KEY` secret-canary scrubbing across distinct OS PIDs.
- **Interactive Bare-Metal Console** (`beast repl`) + `beast model-c2c` + `beast omni-test`.
- **Unified 7-DLL Native Bare-Metal Ecosystem** (`c_bridges/tsotchke_pinn_bridge.c`, `crates/beast-colony/src/ecosystem_ffi.rs`).
- **Native ELF64 + Multiboot2 Forensic Binary Auditor** (`crates/beast-wire/src/elf_audit.rs`).
- **32 Pure Lean 4 `v4.35.0` Theorems** (`lean/BeastProofs.lean`).

## [0.2.0] - 2026-11-25
### Added
- **3-Kernel NVIDIA RTX 5070 Ti `sm_12.0` Blackwell CUDA PTX Suite** (`cuda/beast_blackwell.ptx`, `crates/beast-cuda`).
- **Trained MIT `SwaT` / `Augment-Cache` Engine + `.bcg` `CacheGraft` Repo Indexer** (`crates/beast-c2c/src/{trainer,graft_io}.rs`).
- **120-Generation Zig Bare-Metal Chimera Insert-Network Breeder (`KC-4`)** (`zig/beast-metal/src/root.zig`).
- **Planaria GF($2^8$) Cauchy MDS + Depth-3 Merkle Self-Healing Capsule** (`crates/beast-wire/src/hbin_capsule.rs`).
- **Multi-Colony Civilization OS + Live `AUTOPHAGE` RWX JIT World Stepping** (`crates/beast-colony/src/{civilization,autophage_ffi}.rs`).
- **Compiled Upstream `tsotchke/quantum_rng` Win64 DLL Bridge** (`c_bridges/tsotchke_qrng_bridge.c`).
- **Live Public OSS API Super-Tool**, **Freestanding `x86_64` Microkernel v2**, **10-Gen CRuby/YARV Breeder**, and **24 Pure Lean 4 Theorems**.

## [0.1.0] - 2026-11-25
### Added
- Initial `beast-civilization` polyglot monorepo (`Zig 0.17.1`, `Rust 1.94.1`, `CRuby 3.4.9/YARV`, `C11/AUTOPHAGE`, `CUDA sm_120 PTX`, `Pure Lean 4 v4.35.0`), Build Standard v1.2.0 core, and `300`-item agent matrix.
