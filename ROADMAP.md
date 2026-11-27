<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Roadmap (`A29`)

> **Maintenance Status (`A42`):** `status: active` — maintained on `main` with semantic versioning (`v1.0.0`).

- **S0 (`v0.1.0`, Done):** Build Standard core, `300_agent_matrix.yaml`, Zig 0.17.1 setup, upstream `vendor/VENDOR_MANIFEST.json` SHA pinning, local A-Life ABI links.
- **S1 (`v0.1.0`–`v0.2.0`, Done):** 4-language bare-metal core (`Zig` `u1`/`b1.58`/`i2`/`NVFP4` SIMD + Chimera breeder + freestanding ELF64; `Rust` Ring-0 + C2C/SwaT + Colony OS; `CRuby/YARV`; `CUDA sm_12.0` PTX; initial Pure Lean 4 suite).
- **S2 (`v0.3.0`–`v0.5.0`, Done):** 4-layer autoregressive `GQA → Chimera` C2C LLM, Planaria $\mathrm{GF}(2^8)$+Merkle 50% healing, QEMU Ring-0 booted ELF32 microkernel, Win32 Kernel Job Objects, Autonomous Trinity Forge, 11 live native DLLs.
- **S3 (`v0.6.0`–`v0.7.0`, Done):** 12th live DLL (`QGTL`), FIPS-197 AES-256 KAT, 5-kernel Blackwell PTX (`+MLA 4×`, `+MoE Top-2`) + pinned async VRAM paging, Eshkol tri-mode exact AD, standard `llama.cpp` GGUF v3 `Q4_0`/`F16`/`F32` codec, zero-crate OpenAI HTTP/1.1 server (`beast serve`), MCP JSON-RPC 2.0 server, and initial AMS-LaTeX preprint.
- **S4 (`v1.0.0`, Done — Current Release):** 3-cycle adversarial security & parser hardening (27 remediated findings across all 10 finding classes, 1,000-case fuzz suite, `29/29` Rust tests, `0` Clippy warnings under `-D warnings`, `4/4` CRuby Prism AST mutants), Palomar 6-kernel modular Lean 4.35.0 suite (`81/81` declarations incl. `NV01`–`NV08` & `AT01`–`AT08`, live sorry-tamper rejection), pre-registered `N=24` Holm-Bonferroni statistics (`4/4` PASS), 11-page AMS-LaTeX monograph (`45/45` live Crossref DOIs, `0` retractions), interactive GitHub Pages Proof Blueprint, 3-OS CI, DigiCert RFC 3161 `.tsr` timestamp, SLSA v4 attestations, and `100.0%` Build Standard v1.2.0 + Playbook v1.3 readiness.
- **S5 (`v1.1.x`, Planned):** Linux `cgroups v2` / `seccomp-bpf` sandbox parity with Win32 Job Objects; Blackwell `tcgen05` tensor-core MMA PTX kernels; multi-host distributed C2C benchmarking on frontier open-weight GGUF checkpoints via `--endpoint`; external third-party human replications logged in `REPLICATION.md`.
