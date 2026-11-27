<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Errata & Statement-Audit Resolution Ledger

This file preserves the historical **2026-11-26 statement audit** of frozen `v0.7.0` artifacts (`c421135`) and records how each finding was resolved in **`v1.0.0`**.

---

## Resolution Summary in `v1.0.0` (2026-11-26 / 2026-11-27)

Every finding from the `v0.7.0` statement audit below was addressed cleanly in `v1.0.0`:
1. **Exact Axiom Split & Mechanical Counts:** `v1.0.0` expands the suite to `81` Palomar declarations (`55` axiom-free + `26` strictly within `{propext, Classical.choice, Quot.sound}`), verified mechanically line-for-line against `results/lean4-audit.txt`, `README.md`, and `paper/main.tex` Table 4, explicitly partitioning `41` quantified/witness/refutation declarations vs. `40` closed ground/constant-check statements.
2. **Standalone Model Boundary:** `README.md`, `paper/main.tex` §4.1, and `docs/statement-fidelity.md` explicitly state that all Lean 4.35.0 definitions are standalone specification models not extracted from or linked to the Zig, Rust, C11, or PTX source code; `B09`–`B10` model a userland root-policy record (`RootState`), not CPU ring 0; `B54`–`B55` prove `DualInt` commutativity only (tri-mode AD agreement `0.00e0` rests on empirical gate `G27` + Python oracle); `B02` proves the left-inverse round-trip across all 243 `Fin 3` tuples (surjectivity checked in Python/C11).
3. **Honest Receipt Exporter (`crates/beast-cli/src/main.rs`):** Removed all passing fallback defaults (`else { 5 }` / `!available || ...`); `beast verify-all --export-lean` now writes `0` whenever GPU, AUTOPHAGE, QEMU, or Win32 Job Object backends are absent, and `B60`/`B65` are explicitly documented as consistency checks of committed constants in `LiveReceiptData.lean`.
4. **Terminology, Identity & Workstation Paths:** Replaced "Telepathy" with "Cross-Architecture KV-Cache Transfer" in manuscript/citation titles (`paper/main.tex`, `CITATION.cff`, `CITATION.bib`, `.zenodo.json`); removed all unformed corporate-entity attributions (dual `Apache-2.0` / `CC-BY-4.0` under `Alexander Stephen Donahue, Independent Researcher`); removed workstation drive paths from docs and `vendor/VENDOR_MANIFEST.json`; updated CI Tectonic step to install via official `drop-sh.fullyjustified.net`.

---

## Historical 2026-11-26 Statement Audit (Frozen `v0.7.0`, commit `c421135`)

Audit log: <https://github.com/0thernes/0thernes-research-archive/blob/main/docs/reviews/2026-11-26-statement-audit.md>. Audited commit: `c4211353bb2bf3e8af3aab0bdef088db2ea0fb34`.

### `v0.7.0` Paper & Docs Findings (All Resolved in `v1.0.0`)
1. `v0.7.0` Table `tab:lean` reported `43 zero-ax / 22 std` instead of `45 zero-ax / 20 std` (resolved in `v1.0.0`: `55 zero-ax / 26 std` across `81` declarations verified directly from `results/lean4-audit.txt`).
2. `v0.7.0` overstated Lean certification of native code without separating closed literal statements (`39/65` in `v0.7.0`) from standalone formal models (resolved in `v1.0.0` §4.1 & `docs/statement-fidelity.md`).
3. `v0.7.0` described `B60`/`B65` as binding runtime execution to Lean while `verify-all --export-lean` substituted passing defaults when hardware was absent (resolved in `v1.0.0`: exporter writes `0` when absent; `B60`/`B65` labeled committed-constant consistency checks).
4. `v0.7.0` cited `B54`/`B55` alongside empirical tri-mode AD agreement `G27` (resolved in `v1.0.0`: `B54`/`B55` documented strictly as `DualInt` commutativity; `G27` + Python oracle credited for float AD).
5. `v0.7.0` used "Cache-to-Cache Telepathy" metaphor in paper/citation titles (resolved in `v1.0.0`: "Cross-Architecture KV-Cache Transfer").
6. `v0.7.0` `LICENSE`, `THIRD_PARTY_NOTICES.md`, `vendor/README.md` referenced an unformed LLC, and `.github/workflows/ci.yml` mislabeled Tectonic `0.15.0` as `0.17.0` (all resolved in `v1.0.0`).
