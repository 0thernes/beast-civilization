# Changelog (`A07`)

## [0.7.0] - 2026-11-26 — OPENAI/GGUF LIVE SERVER, AMS-LATEX WHITE PAPER & ARCHIVE RELEASE
### Added
- **Standard `llama.cpp` GGUF v3 `Q4_0` / `F16` / Full-Metadata Codec** (`crates/beast-model/src/gguf.rs`, `beast inspect-gguf`, `G31`): exact IEEE-754 `binary16` conversion, 18-byte `Q4_0` block quantization/dequantization (`64/9×` compression vs `F32`, `max_err=0.1261`), full GGUF v3 metadata type decoding.
- **Zero-Crate OpenAI-Compatible Local HTTP/1.1 Inference Server (`beast serve`) & SSRF-Guarded LLM Client (`beast agent-run --endpoint`)** (`crates/beast-model/src/llm_server.rs`, `crates/beast-agent/src/llm_client.rs`, `G32`): live TCP `GET /health`, `GET /v1/models`, and `POST /v1/chat/completions` backed by 4-layer GQA→Chimera C2C inference, wired directly into Ring-0 metered agent missions with strict loopback SSRF + CRLF injection blocking.
- **Scripted & Interactive Colony Mission Console (`beast repl --script`, `mission <goal>`)** (`G33`): end-to-end live GGUF/C2C loopback mission dispatch signed into Rosette-Wire chains with Ring-0 protected-root refusal.
- **65 Pure Lean 4.35.0 Theorems + Dedicated `lean/VerifyAxioms.lean` + `lean/lean-toolchain` + `lean/lakefile.toml`**: added `B61`–`B65`, strictly within `{propext, Classical.choice, Quot.sound}` (`0 sorry`, `0` custom axioms, `0` warnings).
- **Clean-Room Python 3 Exact-Arithmetic Oracle** (`scripts/verify_independent_oracle.py`), **Publication-Grade AMS-LaTeX White Paper** (`paper/main.tex` -> `dist/Donahue_2026_beast_civilization.pdf` + clean-room `dist/arxiv_submission.tar.gz`), `CITATION.cff`, `VERIFICATION.md`, and full 3-job Linux GitHub Actions CI (`.github/workflows/ci.yml`).
- **Forward-Compatible `cellulogonic.dll` ABI Check**: cleanly links upgraded `cellulogonic-mecha v1.6.0` (`ABI=0x00010600`) and `AUTOPHAGE` (`ABI=0x00010400`).

## [0.6.0] - 2026-11-25 — THE OMNISCIENT HYPER-COLONY
### Added
- **12th Live Native DLL (`tsotchke/quantum_geometric_tensor` QGTL) + FIPS-197 AES-256 KAT & PQC-Sealed C2C**:
  - `c_bridges/tsotchke_qgtl_bridge.c` → `bin/tsotchke_qgtl_bridge.dll`: compiles upstream `QGTL` `complex_arithmetic.c` via `zig cc`, verifying Bloch-sphere normalization, antipodal orthogonality (`ortho_err=2.28e-9`), and Fubini-Study fidelity (`0.750000`, `G26`).
  - Upgraded `c_bridges/tsotchke_moonlab_mlkem_bridge.c` to also compile upstream `aes.c`, verifying exact NIST FIPS-197 Appendix C.3 AES-256 KAT + end-to-end Post-Quantum ML-KEM-768 + AES-256-CTR sealing/unsealing of C2C KV headers (`G21`).
- **5-Kernel NVIDIA RTX 5070 Ti `sm_12.0` Blackwell CUDA PTX + Pinned Host-RAM ↔ VRAM Async Paging (`W059`, `F058`)**:
  - Added PTX Kernel 4 (`beast_mla_latent_compress`, 4× MLA KV compression, `22 µs`) and PTX Kernel 5 (`beast_moe_swiglu_sparse`, Top-2 Gated SwiGLU FFN, `18 µs`) to `cuda/beast_blackwell.ptx` + `crates/beast-cuda/src/lib.rs`.
  - Added genuine CUDA Pinned Host Memory (`cuMemAllocHost_v2`, `cuStreamCreate`, `cuMemcpyHtoDAsync_v2`, `cuMemcpyDtoHAsync_v2`) transferring `16 MiB` round-trip at **`10.17 GB/s`** with 0-bit exact verification (`G11`).
- **Eshkol-Style Zero-Finite-Diff Exact Tri-Mode Autodiff** (`crates/beast-model/src/eshkol_ad.rs`, `beast ad-test`, `G27`): Forward Dual Numbers, Reverse Wengert Tape, and Symbolic AST differentiation agree with **`max_err = 0.00e0`** (`P067`, `F063`, `F099`).
- **Cross-Repository Multi-Workspace ABI Graph + Bio-Frostbite Confluent Scheduler + Isolated Parallel Worktrees** (`crates/beast-agent/src/hyper_graph.rs`, `beast cross-repo`, `G28`): resolves `71` ABI symbols & `10` edges across `3` Dev-Drive repos (`P008`, `W008`), proves Church-Rosser confluence across `12` parallel tasks (`W028`), and executes isolated parallel worktree subagents (`F028`).
- **Standardized MCP JSON-RPC 2.0 Stdio Server + Native Zig/LLVM x86-64 Disassembly Inspector** (`crates/beast-agent/src/mcp_server.rs`, `beast mcp-serve`, `beast inspect-asm`, `G29`/`G30`): closes `W088`, `F083`, and `F087` (`691,622 B`, `9,936` real x86-64 instructions, `74` exported symbols).
- **60 Pure Lean 4 `v4.35.0` Theorems** (`lean/BeastProofs.lean`, `0 sorry`, `0` custom axioms, `0` warnings) adding `B50`–`B60` + `30/30` gates + `35/35` total planted mutants + `19/19` Rust unit tests + **`296 / 300` (`98.7%`)** agent matrix items live-verified!

## [0.5.0] - 2026-11-25 — THE MATRIX SINGULARITY
### Added
- Compiled upstream `tsotchke/moonlab` FIPS-203 ML-KEM-768 PQC & `tsotchke/spin_based_neural_network` Fibonacci Anyons + Spintronic `p-bit` (`11/11` live DLLs), genuine Win32 OS Kernel Job Object hard sandbox (`128 MiB` + `KILL_ON_JOB_CLOSE`), 4-layer autoregressive GQA → Chimera live C2C LLM (`142,496 B` GGUF v3), Autonomous Bare-Metal A-Life Forge (`862,208 B` Zig `.exe`, `residual=0`, P9, Planaria healed), `25/25` gates, `30/30` mutants, and `50/50` Pure Lean 4 theorems.

## [0.4.0] - 2026-11-25 — THE AUTONOMOUS MATRIX FORGE
### Added
- Live QEMU `11.1.0` Ring-0 bootable `i386` Multiboot1 AI microkernel (`F-08` closed), compiled upstream `tsotchke/libirrep` & `tsotchke/moonlab` QGT DLLs (`9/9` DLLs), 8th crate `crates/beast-agent`, `20/20` gates, `25/25` mutants, `40/40` Pure Lean 4 theorems.

## [0.3.0] - 2026-11-25
### Added
- 7th crate `crates/beast-model` (GGUF v3, cross-arch C2C + Planaria heal, TTS/ASR `.wav` + Sobel/`b1.58` OCR `.pgm`), live OS-process Ring-0 handoff, `beast repl`, `7/7` DLLs (`+tsotchke/PINN`), ELF64/Multiboot2 auditor, `32` Pure Lean 4 theorems.

## [0.2.0] - 2026-11-25
### Added
- 3-kernel RTX 5070 Ti PTX, analytical-gradient MIT `SwaT` trainer + `.bcg` `CacheGraft`, 120-gen Zig Chimera breeder (`KC-4`), Planaria GF($2^8$)+Merkle capsule, Multi-Colony OS + live `AUTOPHAGE` RWX JIT, upstream `tsotchke/quantum_rng` Bell DLL, Public OSS API tool, freestanding Microkernel v2, 10-gen CRuby/YARV breeder, `24` Pure Lean 4 theorems.

## [0.1.0] - 2026-11-25
### Added
- Initial `beast-civilization` polyglot monorepo (`Zig 0.17.1`, `Rust 1.94.1`, `CRuby 3.4.9/YARV`, `C11/AUTOPHAGE`, `CUDA sm_120 PTX`, `Pure Lean 4 v4.35.0`), Build Standard v1.2.0 core, and `300`-item agent matrix.
