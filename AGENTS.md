<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# AGENTS.md — Canonical Agent Constitution (`beast-civilization v1.0.0`)

> **Canonical entry point (Build Standard v1.2.0 `A32`).** `CLAUDE.md`, `GEMINI.md`, and `.cursor/rules/build-standard.mdc` delegate here.

## Mission
`beast-civilization` (`v1.0.0`) is a **zero-dependency bare-metal polyglot multi-colony AI engine, cross-architecture Cache-to-Cache substrate, and 6-kernel formally verified research stack** written in **Zig 0.17.1 + Rust 1.94.1 (8 crates) + CRuby 3.4.9/Prism/YARV + C11 + NVIDIA Blackwell `sm_12.0` CUDA PTX + Pure Lean 4.35.0 (81 declarations)**.
Target profiles: `T13` (OS kernel/bare-metal), `T07` (AI/LLM systems), `T05` (Library/SDK), `T04` (CLI). **Zero external Cargo crates, zero Ruby gems, zero Lean Mathlib imports.**

## Hard Rules (`R1–R9`)
- **R1–R4:** Never add branch protection, blocking hooks, commit/tag signing requirements, PR templates, Dependabot/Renovate, CodeRabbit, `FUNDING.yml`, or `.github/copilot-instructions.md`. Commit and push straight to `main`. CI is strictly informational.
- **R5:** Independent-researcher lean, not corporate. Prefer `.\build.ps1` / `just reproduce` and plain auditable files.
- **R6 (Zero Fabrication):** Every number, latency, speedup, `p`-value, DOI, proof count, and pass rate MUST come from a live run of the current code. Unverified claims are forbidden. Claim `0 / 1,900` overarching Mega-Atlas `v2.8.0` open problems retired.
- **R7:** Classify every audit finding as exactly one of: `bug`, `error`, `issue`, `problem`, `concern`, `dilemma`, `conflict`, `gap`, `hole`, `blindspot`.
- **R8:** Precedence: Owner's explicit instruction > Hard Rules > Build Standard v1.2.0 & Playbook v1.3 > ecosystem conventions.
- **R9:** Safety comes from Ring-0 ACL normalization, Win32 Kernel Job Objects, 40 planted mutants, 1,000-case fuzzing, 6-kernel Lean 4 proofs, and tested 0-bit rollback — never from blocking commits.

## Non-Negotiable AI & Verification Guardrails
- **PowerShell Bracket Safety:** The working copy may sit in a folder whose path contains brackets and spaces. ALWAYS pass `-LiteralPath` to PowerShell file cmdlets (`Test-Path`, `Get-Content`, `Set-Location`, etc.).
- **Author & License Discipline:** Author is `Alexander Stephen Donahue` (`Independent Researcher, Spokane, WA, USA`, ORCID `0009-0006-8168-9648`, `0_0@0thernes.art`). Code & Lean proofs: `Apache-2.0`; Manuscript & Docs: `CC-BY-4.0`. Never fabricate DOIs, peer review, or institutional claims.
- **Lean 4.35.0 Palomar Purity:** `lean/Challenge.lean` imports ONLY `BeastProofs.Spec`. `lean/Solution.lean` and all 9 proof modules (`81` declarations: `B01`–`B65`, `NV01`–`NV08`, `AT01`–`AT08`) MUST compile with `0 sorry`, `0 admit`, `0` custom axioms, `0 bv_decide`/`native_decide`, `0` warnings, strictly within `{propext, Classical.choice, Quot.sound}`, passing `lake comparator --config comparator.json --paranoid` and `scripts/check_readme_axioms.py`.
- **Rust / Zig / CRuby Purity:** `cargo clippy --workspace --all-targets -- -D warnings` MUST exit `0` (`0` warnings); `cargo test --workspace` (`29/29`), `zig test zig/beast-metal/src/root.zig`, and `ruby -w ruby/beast_yarv.rb` (`4/4` Prism mutants blocked) MUST all pass cleanly.

## Architecture Map
1. `zig/beast-metal/` — `u1`/`b1.58`/`i2`/`NVFP4`/`FP8` SIMD kernels, Chimera breeder, Philox4x32-10, bounded C-ABI (`beast_metal.dll` / `.so` / `.dylib`).
2. `zig/freestanding_kernel/` — Zero-`libc` `x86_64` Multiboot2 (`kernel.zig`) + live QEMU-booted `i386` Multiboot1 (`boot32.zig`) neural microkernels.
3. `crates/beast-wire/` — Rosette-Wire SHA-256 receipts, Planaria $\mathrm{GF}(2^8)$+Merkle capsules, overflow-hardened ELF64/ELF32 Multiboot auditor.
4. `crates/beast-root/` — Ring-0 canonicalized ACL guardian, micro-USD spend cap, OS-process sandbox handoff, Win32 Kernel Job Objects (`128 MiB`), 0-bit rollback.
5. `crates/beast-c2c/` — MIT `SwaT` gradient trainer, `Augment-Cache` vs `Overwrite`, bounded `.bcg` `CacheGraft` codec, anti-spoof transport enum.
6. `crates/beast-colony/` — 3-colony × 6-caste OS, NonHuman economics, Wilson score, cosine-schedule QUBO, 12-DLL live FFI (`AUTOPHAGE`, `omega_cell`, `cellulogonic`, 7 compiled upstream Tsotchke C11 bridges).
7. `crates/beast-model/` — Hardened GGUF v3 (`Q4_0`/`F16`/`F32`), zero-crate OpenAI HTTP/1.1 server (`beast serve` with `/healthz`, `405`/`413`), 4-layer GQA→Chimera C2C LLM, Eshkol tri-mode exact AD (`0.00e0`), deterministic DSP TTS/ASR/OCR.
8. `crates/beast-agent/` — Merkle AST Repo-Map (`69` files / `416` syms), 4-repo ABI Graph (`94` syms / `31` edges), Confluent Scheduler & Worktrees, hardened MCP JSON-RPC 2.0 server, Zig/LLVM disassembler, SSRF-guarded LLM client, Autonomous Trinity Forge.
9. `crates/beast-cli/` — Unified `beast` CLI (`verify-all`, `stats-n24`, `serve`, `inspect-gguf`, `agent-run`, `repl`, `mcp-serve`, `bench`) + `tests/hardening_fuzz_props.rs` (`1,000`-case fuzz + `70/70` Planaria subsets).
10. `ruby/beast_yarv.rb` — CRuby 3.4.9 + Prism AST Ring-0 gate (`4/4` mutants blocked) + 10-gen YARV breeder + Zig Fiddle FFI.
11. `cuda/beast_blackwell.ptx` & `crates/beast-cuda/` — 5-kernel `sm_12.0` Blackwell PTX + pinned async VRAM paging via `nvcuda.dll`.
12. `lean/` — Six-kernel Palomar Lean 4.35.0 suite (`81/81` declarations, `Spec`/`Challenge`/`Solution`/`comparator.json`/`VerifyAxioms.lean`).

## One-Command Build & Verify (`A31`)
```powershell
.\build.ps1
```
or cross-platform:
```bash
just reproduce
```
