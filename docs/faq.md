<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Frequently Asked Questions (`A44`)

### 1. Does `beast-civilization` require an NVIDIA RTX 5070 Ti GPU or Windows to run?
No. All 8 Rust crates, the Zig 0.17.1 SIMD metal library, both freestanding ELF32/ELF64 microkernels, CRuby 3.4.9 / Prism / YARV, the clean-room Python oracle, the 1,000-case fuzz suite, and all 81 Pure Lean 4.35.0 proofs run natively across **Linux x86-64, Windows x86-64, and macOS ARM64** (verified in 3-OS CI). NVIDIA Blackwell `sm_12.0` PTX (`nvcuda.dll`), Win32 Kernel Job Objects, and local companion DLLs activate automatically when present on the host and degrade gracefully without mocking when absent.

### 2. Why are there zero external Cargo crates, zero Ruby gems, and zero Mathlib imports?
See [ADR-0001](adr/0001-polyglot-bare-metal-zero-deps.md): zero runtime dependencies keep the trusted computing base compact, 100% offline-reproducible, and deterministic across compilers and operating systems.

### 3. Does the Lean 4.35.0 suite verify the Rust, Zig, C11, or CUDA PTX binaries?
**No.** Lean 4.35.0 verifies 81 formal mathematical, encoding, isolation, non-vacuity, adversarial-refutation, and live-receipt-consistency theorems over the discrete specification (`lean/BeastProofs/Spec.lean`). Native runtime behavior and memory safety are checked separately via unit/property/fuzz tests, planted mutants, sanitizers, and live hardware execution. See [`docs/statement-fidelity.md`](statement-fidelity.md) for the exact row-by-row correspondence.

### 4. Does this project claim to solve any open problem in the Unsolved Problems Mega-Atlas?
**No.** As documented in [`docs/MEGA_ATLAS_CROSSCHECK.md`](MEGA_ATLAS_CROSSCHECK.md), **`0 / 1,900`** overarching Mega-Atlas `v2.8.0` open problems are claimed solved or retired.

### 5. Why are 4 of the 300 capability matrix items marked `SPEC` instead of `VERIFIED`?
Strict epistemic honesty (`Build Standard R6`): `F036` (`tsotchke/tensorcore`) targets Apple Silicon Metal/Accelerate and does not run natively on Win64 x86-64, and `F038`/`F068`/`F069` correspond to external multi-gigabyte pretrained checkpoints (HuggingFace/Whisper/Kokoro/TFLite) intentionally not bundled in this zero-dependency repo.

### 6. Can I use `beast serve` with OpenAI SDK clients or `curl`?
Yes. Run `beast serve --port 11435 --gguf bin/beast_4layer_chimera.gguf` and send standard HTTP/1.1 requests to `GET /health`, `GET /healthz`, `GET /v1/models`, or `POST /v1/chat/completions`.
