<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# BEAST CIVILIZATION — Six-Kernel Palomar Lean 4.35.0 Proof Suite (`81` Theorems)

- **Toolchain:** `leanprover/lean4:v4.35.0` (`lean-toolchain`), zero Mathlib (`lake-manifest.json`), `autoImplicit = false`, `relaxedAutoImplicit = false`, `linter.unusedVariables = true`, `linter.deprecated = true`.
- **Module Architecture (`module` headers on every file):**
  - `BeastProofs/LiveReceiptData.lean` — Pure hardware-exported `Nat` receipt constants (`0` proofs).
  - `BeastProofs/Spec.lean` — Pure shared definitions (`0` proofs, `0` `sorry`).
  - `BeastProofs/{Quantization,CacheTelepathy,Ring0Security,ColonyGovernance,CryptoPhysics,SystemsRuntime,LiveReceipt}.lean` — 65 core domain theorems (`B01`–`B65`).
  - `BeastProofs/NonVacuity.lean` — 8 concrete inhabited witnesses (`NV01`–`NV08`).
  - `BeastProofs/AdversarialTests.lean` — 8 formal `¬ P` false-variant refutations (`AT01`–`AT08`).
  - `Challenge.lean` — Isolated statement-only `by sorry` specification importing ONLY `BeastProofs.Spec`.
  - `Solution.lean` — Sorry-free export matched strictly against `Challenge.lean` via `comparator.json`.
  - `VerifyAxioms.lean` — Standalone `#print axioms` audit across all 81 declarations.

## Commands
```bash
cd lean
lake build
lake build Challenge
lake env lean VerifyAxioms.lean
# On Linux with Lean 4.35.0 + bubblewrap (run automatically in GitHub Actions CI):
lake check --paranoid
lake comparator --config comparator.json --paranoid
```
