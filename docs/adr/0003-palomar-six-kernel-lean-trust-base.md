<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# ADR-0003: Six-Kernel Palomar Lean 4.35.0 Separation (Spec / Challenge / Solution)

- **Status:** Accepted (`v1.0.0`, 2026-11-26)
- **Context:** Single-file Lean proof scripts compiled only with default `lean` risk subtle specification drift, vacuous hypotheses, unintended `autoImplicit` bindings, or undetected kernel edge cases.
- **Decision:** Adopt the Palomar verification contract on Lean `v4.35.0`:
  1. Every `.lean` file uses the Lean 4.35.0 `module` system with `autoImplicit = false`, `relaxedAutoImplicit = false`, `linter.unusedVariables = true`, `linter.deprecated = true`, and a committed `lean/lake-manifest.json`.
  2. Shared definitions live exclusively in `lean/BeastProofs/Spec.lean` (zero proofs).
  3. `lean/Challenge.lean` imports ONLY `BeastProofs.Spec` and states all target theorems with `by sorry` (`< 1,000` lines, `< 100 KiB`).
  4. `lean/Solution.lean` exports sorry-free proofs across 7 domain modules + `NonVacuity.lean` (`NV01`–`NV08`) + `AdversarialTests.lean` (`AT01`–`AT08`).
  5. Ubuntu CI verifies `Solution` under sandboxed `bubblewrap` across 6 independent kernel checks (`lean4`, `leanchecker`, `leanchecker-paranoid`, `lake check --paranoid` invoking `lean4lean`/`nanoda`/`con-leche`/`con-ron`, plus `lake comparator --config comparator.json --paranoid`) AND runs a live negative sorry-tamper comparator rejection test.
- **Boundary:** Formal proofs certify discrete mathematical, encoding, isolation, and exported receipt-consistency invariants; they do NOT certify Rust/Zig/C11/PTX memory safety or hardware timings (documented row-by-row in `docs/statement-fidelity.md`).
