# AGENTS.md — Canonical Agent Constitution (`beast-civilization`)

> **Canonical entry point (Build Standard v1.2.0 `A32`).** `CLAUDE.md`, `GEMINI.md`, and `.cursor/rules/build-standard.mdc` all delegate here.

## Mission
`beast-civilization` is a **bare-metal, self-rewriting, multi-colony AI civilization and evolutionary compute substrate** written in **Zig + Rust + CRuby/YARV + C11/CUDA (`sm_120`) + Lean 4 (`v4.35.0`)**.
Target profiles: `T13` (OS kernel/bare-metal), `T07` (AI/LLM systems), `T05` (Library/SDK), `T04` (CLI). **Explicitly NOT `T02` (zero SaaS / PWA / web-app bloat).**

## Hard Rules (`R1–R9`)
- **R1–R4:** Never add branch protection, blocking hooks, required checks, PR templates, Dependabot/Renovate, CodeRabbit, or `.github/copilot-instructions.md`. Commit and push straight to `main`. CI is strictly informational.
- **R5:** One-person lean, not corporate. Prefer one command (`.\build.ps1`) and plain files over ceremony.
- **R6 (Zero Fabrication):** Every number, latency, speedup, proof count, and pass rate MUST come from a live run of the current code. Unverified means labeled `UNVERIFIED`.
- **R7:** Classify every finding as exactly one of: `bug`, `error`, `issue`, `problem`, `concern`, `dilemma`, `conflict`, `gap`, `hole`, `blindspot`.
- **R8:** Precedence: Owner's explicit instruction > Hard Rules > Build Standard > ecosystem conventions.
- **R9:** Safety comes from Ring-0 isolation, planted-mutant falsification, Lean 4 proofs, and tested 0-bit rollback — never from blocking commits.

## Non-Negotiable AI Guardrails (`AI01–AI15`)
- **PowerShell Bracket Safety:** This repo lives on ReFS Dev Drive `Z:\[Vibe Coded (AI)]\ANTIGRAVITY\beast-civilization`. ALWAYS pass `-LiteralPath` to PowerShell file cmdlets.
- **Zero Theater (`AI04`–`AI07`):** No stubbed bodies passed off as done. No `assert true` tests. Every test suite MUST include planted negative mutants that fail when injected. Never disguise HTTP cloud text calls as Cache-to-Cache KV transfer.
- **Lean 4 Purity:** All Lean 4 proofs in `lean/` MUST compile with `0 sorry`, `0 admit`, `0` custom axioms, `0 bv_decide`/`native_decide`, strictly within `{propext, Classical.choice, Quot.sound}`.
- **Determinism (`B03`):** Track-A verified benchmarks MUST use domain-isolated `Philox4x32-10` with 0-bit cross-run parity. Track-B feral exploration explicitly switches to hardware `RDSEED` / `QRNG`.

## Architecture Layers
1. `zig/beast-metal/` — Arbitrary bit-width (`u1`, `b1.58` 5-trit/B, `i2`, `u3`, `nvfp4_e2m1`, `fp8_e4m3`) SIMD tensor kernels, Chimera Insert-Network engine, Philox4x32-10, C-ABI (`beast_metal.dll` / `libbeast_metal.a`).
2. `zig/freestanding_kernel/` — Freestanding `x86_64` Multiboot2 (`kernel.zig`) + live QEMU-booted `i386` Multiboot1 (`boot32.zig`) bare-metal microkernels embedding `b1.58` + integer `NVFP4` neural reflex cores (zero libc).
3. `crates/beast-wire/` — Verifiable typed component/composition/receipt protocol (`rosette-wire`, SHA-256, Planaria GF($2^8$)+Merkle self-healing capsules, ELF64/ELF32 Multiboot auditor).
4. `crates/beast-root/` — Ring-0 immutable guardian (`INIT-Agent` Win64+Linux OS-process sandbox handoff + Win32 OS Kernel Job Objects `128 MiB` + `Noetic` ACL + `G0–G4` self-rewrite gates + 12 harm signals + 0-bit rollback).
5. `crates/beast-c2c/` — Bare-metal Cache-to-Cache & MIT `Swarm Telepathy` (`SwaT`) gradient trainer + `Augment-Cache / Replace-Activation` + persistent `.bcg` `CacheGraft` repo bank + honest `TEXT_CLOUD_FALLBACK` tagging.
6. `crates/beast-colony/` — Multi-colony civilization OS (`TEMM1E` + `NonHuman` + `effortmining` + `Nagi` + `Engram` + `Moonlab` QUBO/QGT/FIPS-203 ML-KEM-768+AES-256 + `SbNN` Fibonacci Anyons & `p-bit` + `QGTL` Bloch Fidelity + `libirrep` SO(3) + `PINN` + `QRNG` + `AUTOPHAGE` + `OMEGA` + `CELLULOGONIC` 12-DLL live FFI).
7. `crates/beast-model/` — Real GGUF v3 binary writer/parser (`F32`/`F16`/`Q4_0`/`b1.58`/`NVFP4`), zero-crate OpenAI HTTP/1.1 inference server (`llm_server.rs`), 4-Layer Autoregressive GQA Transformer → `sLTC/b1.58` Chimera live C2C LLM, Eshkol-style Forward/Reverse/Symbolic Exact AD (`0.00e0` err), Planaria self-healing, and real 16 kHz Formant TTS/ASR `.wav` + noisy 2D Sobel/`b1.58` OCR `.pgm` DSP kernels.
8. `crates/beast-agent/` — Merkle Multi-Language AST Repo-Map, Cross-Repo ABI Graph, Bio-Frostbite Confluent Scheduler & Isolated Worktrees, Standardized MCP JSON-RPC 2.0 Server, Zig/LLVM x86-64 Disassembly Inspector, SSRF-guarded OpenAI/Ollama HTTP/1.1 LLM client (`llm_client.rs`), Autonomous Bare-Metal Trinity Forge (`forge-kernel` + `forge-model` + `forge-alife`), and unified `INIT`×`NonHuman`×`Engram`×`TEMM1E` agent loop.
9. `crates/beast-cli/` — Unified `beast` CLI binary (`serve`, `inspect-gguf`, `agent-run --endpoint`, `repl --script`), interactive REPL, & 33-gate / 33-mutant verification harness.
10. `ruby/beast_yarv.rb` — CRuby 3.4 + YARV + Prism live-mutable 10-gen bytecode breeder (`T1a` tier) calling Zig `beast_metal.dll` via Fiddle C-ABI.
11. `cuda/beast_blackwell.ptx` & `crates/beast-cuda/` — Direct `nvcuda.dll` 5-kernel CUDA Driver API execution (`b1.58`, `nvfp4`, `c2c`, `mla 4x`, `moe swiglu`) + Pinned Host-RAM ↔ VRAM async paging on NVIDIA RTX 5070 Ti (`sm_120` Blackwell) with zero MSVC dependency.
12. `lean/BeastProofs.lean` & `lean/VerifyAxioms.lean` — 65 Pure Lean 4 (`v4.35.0`) formal proofs + live runtime-to-Lean receipt verification (`B65`).

## One-Command Build & Verify (`A31`)
```powershell
.\build.ps1
```
