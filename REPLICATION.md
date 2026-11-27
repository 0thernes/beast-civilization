<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Clean-Clone Replication Protocol & Dated Log (`B01`, `B43`, `B44` — `v1.0.0`)

## Pinned Toolchains (`B02`)
- **Zig:** `0.17.1` (`zig` on `PATH`) — builds `beast_metal.dll` / `libbeast_metal.so` / `libbeast_metal.dylib`, freestanding ELF64/ELF32 microkernels, forged A-Life `.exe`, x86-64 `-femit-asm` disassembly, and all 7 upstream Tsotchke C11 bridges via `zig cc`.
- **Rust:** `1.94.1` (`rustc` + `cargo`, `Cargo.lock` committed, 8-crate workspace, **zero external crates.io dependencies** — 100% offline reproducible).
- **CRuby:** `3.4.9 +PRISM` (`ruby -w`, stdlib + `prism` + `fiddle` only — **zero external gems**).
- **Lean 4:** `v4.35.0` pinned in [`lean/lean-toolchain`](lean/lean-toolchain) + [`lean/lake-manifest.json`](lean/lake-manifest.json) (**zero Mathlib**).
- **QEMU:** `11.1.0` (`qemu-system-x86_64`) for live Ring-0 Multiboot1 ELF32 execution.
- **Tectonic:** `0.17.0` for deterministic AMS-LaTeX PDF & clean-room `arxiv_submission.tar.gz` builds.
- **Optional GPU:** NVIDIA Driver exposing `nvcuda.dll` / `libcuda.so.1` (verified on `RTX 5070 Ti Laptop GPU`, Compute Capability `12.0`).

## One-Command Reproduction (`B01`)

```powershell
# Windows x86-64 (full 12-DLL + Blackwell GPU + QEMU + CRuby + 81 Lean 4.35.0 proofs)
.\build.ps1
python scripts/verify_stats.py
python scripts/verify_independent_oracle.py
```

```bash
# Linux / macOS (cross-platform via justfile)
just reproduce
```

## Step-by-Step Verification Checklist

1. **Zig unit & 3 planted mutants:** `zig test zig/beast-metal/src/root.zig`
2. **Rust strict lint & 29 tests (incl. 1,000-case fuzz & 70/70 Planaria subsets):** `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
3. **C11 ASan + UBSan check (Linux/POSIX):** `bash scripts/check_sanitizers.sh`
4. **Live 33-gate / 33-mutant verification + $N=24$ paired statistics:** `.\bin\beast.exe verify-all --export-lean lean\BeastProofs\LiveReceiptData.lean && python scripts/verify_stats.py`
5. **CRuby strict warnings + 4/4 Prism AST mutants + YARV 10-gen breeder:** `ruby -w ruby/beast_yarv.rb`
6. **Clean-room Python oracle + 3,000 property checks + agent eval suite:** `python scripts/verify_independent_oracle.py && python scripts/prop_check.py && python evals/eval_suite.py`
7. **Palomar Lean 4.35.0 (`81/81`) + README parity + 3/3 negative tamper tests:** `(cd lean && lake build && lake build Challenge && lake env lean VerifyAxioms.lean) && python scripts/check_readme_axioms.py && python scripts/test_lean_negative.py`
8. **Live Crossref citation audit (`45/45`), zero-warning PDF/arXiv compile, and Blueprint portal:** `python scripts/check_citations.py && python scripts/build_paper.py && python scripts/build_blueprint.py`

## Dated Replication Log (`B44`)

External third-party human replication is openly invited (open a GitHub issue with your platform, toolchain versions, and hashes to be added below).

| Date (UTC) | Replicator | Platform / Hardware | Scope Verified | Result |
|---|---|---|---|---|
| 2026-11-26 | Alexander Stephen Donahue (Author, local clean run) | Windows 11 x86-64 · Intel Core Ultra 9 275HX · RTX 5070 Ti `sm_12.0` · QEMU 11.1.0 | Full `.\build.ps1` (`33/33` gates, `12/12` DLLs, `5/5` PTX, `40/40` mutants), `N=24` Holm-Bonferroni (`4/4`), `29/29` Rust + 1,000-case fuzz, `81/81` Lean 4.35.0, `45/45` Crossref DOIs, 11-page PDF | **PASS** (`results/verify_win64.txt`, `results/paired_stats_n24.txt`) |
| 2026-11-26 | GitHub Actions CI (`ubuntu-latest`, `windows-latest`, `macos-latest`) | Cloud 3-OS x86-64 + ARM64 runners (CPU, zero GPU) | Zig `0.17.1`, Rust `1.94.1` (`29/29` + Clippy `-D warnings`), C11 ASan/UBSan, CRuby `3.4`, 6-kernel Palomar Lean `4.35.0` (`bubblewrap` + live sorry-tamper rejection), Tectonic PDF/arXiv, Blueprint, `gitleaks` & `osv-scanner` | **PASS** (verified on `v1.0.0` push) |
