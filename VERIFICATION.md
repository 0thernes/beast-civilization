# Verification & Falsification Dossier — `beast-civilization` `v0.7.0`

**Author:** Alexander Stephen Donahue (Independent Researcher, Spokane, WA, USA · ORCID [`0009-0006-8168-9648`](https://orcid.org/0009-0006-8168-9648))  
**Release:** `v0.7.0` · **Date:** 2026-11-26 · **Zero External Dependencies:** `0` Cargo crates, `0` Ruby gems, `0` Lean Mathlib imports

## Summary of Live Verification Evidence

| Verification Layer | Result | Evidence File |
| :--- | :--- | :--- |
| **Full Win64 Silicon Build (`.\build.ps1`)** | **ALL GREEN (exit `0`)** | [`results/verify_win64.txt`](results/verify_win64.txt) |
| **Live Hardware Gates & Planted Mutants** | **`33 / 33` gates PASS** · **`38 / 38` planted mutants caught** (`3` Zig + `33` Rust/CLI + `2` CRuby/Prism) | [`results/verify_win64.txt`](results/verify_win64.txt) |
| **Rust 8-Crate Unit & Integration Tests** | **`21 / 21` PASS** (`0` warnings) | [`results/verify_win64.txt`](results/verify_win64.txt) |
| **Pure Lean 4.35.0 (`lean/VerifyAxioms.lean`)** | **`65 / 65` theorems PASS** (`43` zero-axiom + `22` strictly within `[propext, Classical.choice, Quot.sound]`, `0 sorry`, `0` warnings) | [`results/lean4-audit.txt`](results/lean4-audit.txt) |
| **Clean-Room Python 3 Exact Oracle** | **`243/243` b1.58 exhaustive + `65,536/65,536` GF($2^8$) + Eshkol AD + `300`-matrix + `65/65` Lean audit + `6/6` oracle mutants** | [`scripts/verify_independent_oracle.py`](scripts/verify_independent_oracle.py) |
| **12-DLL Bare-Metal Ecosystem** | **`12 / 12` live DLLs** (Zig + 3× 0thernes A-Life + 7× compiled upstream Tsotchke C11 + `nvcuda.dll`) | [`results/verify_win64.txt`](results/verify_win64.txt) |
| **Blackwell RTX 5070 Ti (`sm_12.0`) PTX** | **`5 / 5` kernels verified** + **`20.21 GB/s`** pinned async VRAM paging (`16 MiB` 0-bit exact) | [`cuda/beast_blackwell.ptx`](cuda/beast_blackwell.ptx) |
| **Ring-0 QEMU Multiboot1 ELF32 Boot** | **`12,692 B` ELF32 booted live** (`B158=7/7`, `NVFP4=59/59`, `BIT_EXACT=true`) | [`zig/freestanding_kernel/src/boot32.zig`](zig/freestanding_kernel/src/boot32.zig) |
| **OpenAI/GGUF HTTP/1.1 Server & Agent Wire** | **`beast serve` ↔ `beast agent-run --endpoint`** + scripted **`beast repl --script`** + **`Q4_0`/`F16`/`F32`** GGUF v3 codec | [`crates/beast-model/src/llm_server.rs`](crates/beast-model/src/llm_server.rs) |
| **AMS-LaTeX White Paper & arXiv Bundle** | Zero-warning Tectonic PDF + standalone clean-room verified `arxiv_submission.tar.gz` | [`dist/Donahue_2026_beast_civilization.pdf`](dist/Donahue_2026_beast_civilization.pdf) |

## Independent Reproduction & Falsification Commands

```powershell
# Windows (full 12-DLL + Blackwell RTX 5070 Ti + QEMU + CRuby + 65 Lean 4 proofs)
.\build.ps1
python scripts/verify_independent_oracle.py
```

```bash
# Linux / CI (Zig 0.17.1 + Rust 8-crate + Python exact oracle + 65 Pure Lean 4.35.0 proofs)
zig test zig/beast-metal/src/root.zig
cargo test --workspace
python3 scripts/verify_independent_oracle.py
(cd lean && lean -R . -o LiveReceipt.olean LiveReceipt.lean && lean -R . -o BeastProofs.olean BeastProofs.lean && lean -R . VerifyAxioms.lean)
```
