<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Contributing to BEAST CIVILIZATION (`A14`)

Contributions, independent replications, bug reports, security audits, and Lean 4 formalization extensions are welcome.

## Standards & Non-Negotiable Rules
1. **Zero Hidden Dependencies:** Keep all 8 Rust crates free of external crates.io dependencies, CRuby free of external gems, Lean 4 free of Mathlib, and Python scripts standard-library-only.
2. **Zero Warnings & Strict Safety:** Every Rust change must pass `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Every untrusted-input parser must handle malformed inputs without panics, integer overflows, or OOM aborts.
3. **Lean 4.35.0 Palomar Integrity:** Zero `sorry`, zero `admit`, zero `native_decide`, zero `bv_decide`, and zero custom axioms in `lean/Solution.lean` or `lean/BeastProofs/`. All theorems must audit strictly inside `{propext, Classical.choice, Quot.sound}` via `lean/VerifyAxioms.lean` and pass `lake check --paranoid` + `lake comparator --config comparator.json --paranoid`.
4. **Falsifiability & Honesty:** Every new gate must include a planted negative mutant that fails when the invariant is violated. Never fabricate benchmarks, DOIs, or claims.
5. **Finding Taxonomy:** Classify audit findings using the 10 standard labels in `.github/labels.tsv`: `bug`, `error`, `issue`, `problem`, `concern`, `dilemma`, `conflict`, `gap`, `hole`, `blindspot`.
6. **Licensing:** Code and Lean contributions are licensed under Apache-2.0; documentation and manuscript contributions under CC-BY-4.0 (see `LICENSE`, `LICENSE-DOCS`, and `REUSE.toml`).

## Local Pre-Flight Commands
```powershell
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/verify_independent_oracle.py
python scripts/prop_check.py
python scripts/check_versions.py
pwsh -NoProfile -File scripts/Audit-Repo.ps1
```
