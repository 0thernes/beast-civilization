<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Verification & Falsification Dossier — `beast-civilization` `v1.0.0`

**Author:** Alexander Stephen Donahue (Independent Researcher, Spokane, WA, USA · ORCID [`0009-0006-8168-9648`](https://orcid.org/0009-0006-8168-9648))  
**Release:** `v1.0.0` · **Date:** 2026-11-26 · **Zero External Dependencies:** `0` Cargo crates, `0` Ruby gems, `0` Lean Mathlib imports

## Summary of Live Verification Evidence

| Verification Layer | Result | Evidence File |
| :--- | :--- | :--- |
| **Full Win64 Silicon Build (`.\build.ps1`)** | **ALL GREEN (exit `0`)** | [`results/verify_win64.txt`](results/verify_win64.txt) |
| **Live Hardware Gates & Runtime Planted Mutants** | **`33 / 33` gates PASS** · **`40 / 40` runtime mutants caught** (`3` Zig + `33` Rust/CLI + `4` CRuby/Prism) | [`results/verify_win64.txt`](results/verify_win64.txt) |
| **Pre-Registered `N=24` Paired Holm-Bonferroni Stats** | **`4 / 4` primary hypotheses SIGNIFICANT** (`p_Holm = 0.00040 <= 0.05`, `24/24` paired wins each) | [`results/paired_stats_n24.txt`](results/paired_stats_n24.txt) |
| **Rust 8-Crate Unit, Regression, Property & 1,000-Case Fuzz Suite** | **`29 / 29` PASS** (`21` unit + `8` hardening/regression/property/1,000-case fuzz) · **`0` Clippy warnings (`-D warnings`)** | [`crates/beast-cli/tests/hardening_fuzz_props.rs`](crates/beast-cli/tests/hardening_fuzz_props.rs) |
| **Six-Kernel Palomar Pure Lean 4.35.0 Suite** | **`81 / 81` declarations PASS** (`65` domain + `8` `NV01`–`NV08` + `8` `AT01`–`AT08`; `55` zero-ax + `26` standard; `0 sorry`, `0` custom axioms, `0` warnings; `3/3` negative tamper rejected; live sorry-tamper comparator rejection in CI; verbatim `README.md` parity) | [`results/lean4-audit.txt`](results/lean4-audit.txt), [`lean/comparator.json`](lean/comparator.json) |
| **Clean-Room Python 3 Oracle & Randomized Property Suite** | **`243/243` `b1.58` + `65,536/65,536` $\mathrm{GF}(2^8)$ + `Q4_0 = 64/9×` + Eshkol AD (`0.00e+00`) + `300`-matrix + `81/81` Lean audit + `6/6` oracle mutants + `3,000/3,000` randomized property trials** | [`scripts/verify_independent_oracle.py`](scripts/verify_independent_oracle.py), [`scripts/prop_check.py`](scripts/prop_check.py) |
| **12-DLL Bare-Metal Ecosystem & Blackwell RTX 5070 Ti PTX** | **`12 / 12` live DLLs** + **`5 / 5` `sm_12.0` PTX kernels** (`16–122 µs` warm) + **`16.69–17.03 GB/s`** pinned async VRAM paging (`16 MiB` 0-bit exact) | [`results/verify_win64.txt`](results/verify_win64.txt), [`cuda/beast_blackwell.ptx`](cuda/beast_blackwell.ptx) |
| **Ring-0 QEMU ELF32 Boot + Hardened OpenAI/GGUF Server** | **`12,692 B` ELF32 booted live** (`B158=7/7`, `NVFP4=59/59`), `beast serve` (`/healthz`, `405`/`413`, SSRF/CRLF blocked), `Q4_0`/`F16`/`F32` codec | [`zig/freestanding_kernel/src/boot32.zig`](zig/freestanding_kernel/src/boot32.zig), [`crates/beast-model/src/llm_server.rs`](crates/beast-model/src/llm_server.rs) |
| **11-Page AMS-LaTeX Monograph, Citations & Proof Blueprint** | **Zero-warning Tectonic PDF (`0` overfull hboxes, `0` hyperref warnings) + clean-room `arxiv_submission.tar.gz` + `45/45` live Crossref DOIs (`0` retractions) + static `_site/` Blueprint** | [`dist/Donahue_2026_beast_civilization.pdf`](dist/Donahue_2026_beast_civilization.pdf), [`results/citation-audit.json`](results/citation-audit.json) |
| **Build Standard v1.2.0 & Playbook v1.3 Compliance** | **`100.0%` Build Standard self-audit** · **43/43 Playbook readiness** · **DigiCert RFC 3161 `.tsr` timestamp** · **SLSA v4 attestations** · **`0/1,900` honest Mega-Atlas `v2.8.0` cross-check** | [`SUBMISSION_READINESS.md`](SUBMISSION_READINESS.md), [`docs/MEGA_ATLAS_CROSSCHECK.md`](docs/MEGA_ATLAS_CROSSCHECK.md) |

## Independent Reproduction Commands

```powershell
# Windows x86-64 (full 12-DLL + RTX 5070 Ti PTX + QEMU + CRuby + 81 Lean 4.35.0 proofs)
.\build.ps1
python scripts/verify_stats.py
python scripts/verify_independent_oracle.py
python scripts/prop_check.py
python scripts/test_lean_negative.py
python scripts/check_readme_axioms.py
python scripts/check_citations.py
python scripts/build_paper.py
python scripts/build_blueprint.py
python evals/eval_suite.py
pwsh -NoProfile -File scripts/Audit-Repo.ps1
```

```bash
# Linux / macOS / CI (Zig 0.17.1 + Rust 8-crate + C11 ASan/UBSan + Oracle + 6-Kernel Palomar Lean 4.35.0)
zig test zig/beast-metal/src/root.zig
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash scripts/check_sanitizers.sh
python3 scripts/verify_independent_oracle.py
python3 scripts/prop_check.py
python3 scripts/check_versions.py
(cd lean && lake build && lake build Challenge && lake env lean VerifyAxioms.lean)
python3 scripts/check_readme_axioms.py
python3 scripts/test_lean_negative.py
```
