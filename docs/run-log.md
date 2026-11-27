<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Run Log & Verification Audit Trail (`B40`)

| Date (UTC/Local) | Version | Host / Runner | Scope Executed | Result | Evidence |
|---|---|---|---|---|---|
| 2026-11-26 | `v0.7.0` (`c421135`) | Win11 x86-64 (Core Ultra 9 275HX + RTX 5070 Ti `sm_12.0` + QEMU 11.1.0) | Full `.\build.ps1`, 33 gates, 38 mutants, 12/12 DLLs, 65 Lean proofs, Python oracle, Tectonic PDF | **PASS** | `results/verify_win64.txt`, SWHID `swh:1:snp:a1c542cd7dca536a750e661aaad5857795932b10` |
| 2026-11-26 | `v0.7.0` (`c421135`) | GitHub Actions Ubuntu CI (`#45826608002`) | 3/3 jobs (Zig+Rust+Oracle, Lean 4.35.0 65/65, Tectonic PDF + arXiv tarball) | **PASS** | GitHub Actions run `#45826608002` |
| 2026-11-26 | `v1.0.0` (Cycle 1) | Win11 x86-64 | Baseline `Audit-Repo.ps1` (44.6% -> 27 findings logged), hardened all parsers/ACL/Zig/Ruby, Clippy `-D warnings` (0 warnings), 29/29 Cargo tests incl. 1,000-case fuzz, 4/4 Prism mutants | **PASS** | `HARDENING_LOG.md`, `docs/reviews/2026-11-26-3-cycle-hardening-audit.md` |
| 2026-11-26 | `v1.0.0` (Cycle 2–3) | Win11 x86-64 + 3-OS / 6-Kernel Palomar GitHub Actions CI | Full `.\build.ps1`, 81 Palomar Lean proofs, live Crossref citation audit (`0` retractions), $N=24$ Holm-Bonferroni stats, `Audit-Repo.ps1` (`100.0%`), RFC 3161 DigiCert `.tsr`, GitHub Pages Blueprint, SLSA v4 | **VERIFIED ON RELEASE** | `results/verify_win64.txt`, `results/paired_stats_n24.txt`, `results/lean4-audit.txt`, `results/citation-audit.json`, `CHECKSUMS.sha256` |
