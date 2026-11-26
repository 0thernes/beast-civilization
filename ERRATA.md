# Errata

This file records known errors in frozen artifacts (the white paper, its source and the release PDF) and in the Lean sources. They are not corrected in place; the corrections below take precedence over the original text.

## 2026-11-26 statement audit

Audit log: <https://github.com/0thernes/0thernes-research-archive/blob/main/docs/reviews/2026-11-26-statement-audit.md>. Audited commit: `c4211353bb2bf3e8af3aab0bdef088db2ea0fb34`.

### Paper (`paper/main.tex`, `dist/Donahue_2026_beast_civilization.pdf`, `dist/arxiv_submission.tar.gz`)

1. **Where:** Section "Formal Verification in Pure Lean 4.35.0", Table `tab:lean`, row "Total Audited".
   **What is wrong:** the axiom split is given as `43 zero-ax / 22 std`. `#print axioms` (via `lean/VerifyAxioms.lean`) and the repository's own `results/lean4-audit.txt` give `45` theorems with no axioms and `20` using only standard axioms (`10` use `[propext]`, `7` use `[propext, Quot.sound]`, `3` use `[propext, Classical.choice, Quot.sound]`).
   **Correction:** `45 zero-ax / 20 std`. In the same table, row `B06`–`B08`, `B18`, `B21`, `B30` should read `4 zero-ax, 2 [p]` (not `3`/`3`), and row `B17`, `B22`, `B37`, `B40`–`B41`, `B50` should read `4 zero-ax, 2 std` (not `3`/`3`).
   **Affects a theorem statement:** no.

2. **Where:** Introduction (paragraph before "Contributions"); Section "Falsifiability, Epistemic Boundaries, and Disclosure", item "Lean Kernel Scope & Open Problems"; and the AI-assistance disclosure paragraph.
   **What is wrong:** the text says the Lean kernel checks or "certifies" the system's algebraic, encoding, isolation and runtime-receipt invariants. `39` of the `65` theorems are closed literal statements (for example `B53`: `2 * (8 * 1024 * 1024) = 16777216`), and no Lean definition is linked to the Zig, Rust, C or PTX code.
   **Correction:** the Lean suite machine-checks spec arithmetic, bookkeeping sums, small reference models of the C2C, root-policy rewrite, spend-cap and loopback rules, and equalities of recorded receipt constants. The Lean models are standalone and are not linked to the shipped code.
   **Affects a theorem statement:** no.

3. **Where:** Section "Formal Verification in Pure Lean 4.35.0", Remark "Live Hardware-to-Lean Receipt Bridge".
   **What is wrong:** the remark says the exported receipt "binds runtime receipts to Lean". `B60` and `B65` check constants committed in `lean/LiveReceipt.lean`. CI (`.github/workflows/ci.yml`, job `lean4-proofs`) compiles the committed file and never runs `beast verify-all --export-lean` or diffs its output, and the exporter writes passing default values when hardware is absent (see the Lean issues below).
   **Correction:** `B60` and `B65` are consistency checks of committed constants. They do not show that any run produced those constants.
   **Affects a theorem statement:** no.

4. **Where:** Section "Planaria Holographic Healing, Eshkol Tri-Mode AD, and Colony OS" (forward-mode dual numbers cited with `B54`, `B55`) and the live-results table row "Eshkol Tri-Mode Exact AD" (`G27`, `B54`–`B55`).
   **What is wrong:** `B54` proves `dualMul x y = dualMul y x` and `B55` proves `dualAdd x y = dualAdd y x`. Neither states a derivative, the product rule, reverse mode or symbolic differentiation.
   **Correction:** the tri-mode AD agreement (`max_err = 0.00e0`) is an empirical result of gate `G27` and the Python oracle. `B54`/`B55` are commutativity facts about dual-number arithmetic only.
   **Affects a theorem statement:** no.

5. **Where:** title, and Section "Cache-to-Cache Telepathy and the Augment-over-Overwrite Law".
   **What is wrong:** "Cache-to-Cache Telepathy" is a metaphor.
   **Correction:** read it as cross-architecture KV-cache (cache-to-cache) transfer between models.
   **Affects a theorem statement:** no.

### Known issues in the formal code (not yet fixed)

1. **Where:** `lean/BeastProofs.lean` lines 1-2.
   **What is wrong:** the header says "beast-civilization v0.6.0" and "60 Machine-Checked Pure-Kernel Theorems". The file has `65` theorems and the release is `v0.7.0`.
   **Correction:** the header should read `v0.7.0`, `65` theorems.
   **Affects a theorem statement:** no.

2. **Where:** `lean/BeastProofs.lean` line 220 (docstring) and line 233 (`B54_dual_leibniz_comm`).
   **What is wrong:** the docstring names the "Leibniz product rule & associativity of multiplication"; the theorem states only commutativity of `dualMul`.
   **Correction:** rename to `B54_dual_mul_comm` and describe it as commutativity of dual-number multiplication, or prove the product rule for a defined derivative.
   **Affects a theorem statement:** the statement is correct as written; its name and docstring overstate it.

3. **Where:** `lean/BeastProofs.lean` line 87 (`B13_wse_spectral_contraction`), line 152 (`B35_quantum_geometric_bound_psd`), line 198 (`B46_p9_observer_independence`), line 247 (`B56_church_rosser_xor_confluence`).
   **What is wrong:** the names suggest spectral, quantum, observer and rewriting-system results. `B13` proves `wMilli * x < 1000 * x` from `wMilli <= 990` and `x > 0`; `B35` is an integer inequality from a hypothesis; `B46` is proved by `rfl`; `B56` is XOR reassociation on three `BitVec 64` values.
   **Correction:** name theorems after what they state (for example `B13_scaled_lt_of_le_990`, `B56_xor_reassoc`).
   **Affects a theorem statement:** no.

4. **Where:** `lean/BeastProofs.lean` line 55 (section heading "Ring-0 Immutable Guardian & Budget Safety"), theorems `B09`, `B10`.
   **What is wrong:** the theorems model a userland root-policy rewrite on a record (`applyRewrite`), not CPU ring 0, and can be confused with the separate QEMU ring-0 kernel boot.
   **Correction:** describe them as a root-policy rewrite guard and spend cap (software model).
   **Affects a theorem statement:** no.

5. **Where:** `lean/LiveReceipt.lean`; `lean/BeastProofs.lean` `B60` (line 261) and `B65` (line 289); `crates/beast-cli/src/main.rs` lines 969-973; `.github/workflows/ci.yml` lines 58-64.
   **What is wrong:** the receipt constants are committed into the repository and CI only compiles them. The exporter substitutes passing values when hardware is missing, for example `let gpu_kernels: u64 = if gpu.available { gpu.kernels_verified as u64 } else { 5 };`, and likewise for pinned paging, AUTOPHAGE, QEMU and the Win32 Job Object. So `B60`/`B65` would hold for a run with no GPU.
   **Correction:** have the exporter write `0` or a distinct not-run value when a backend is absent, and regenerate and diff `LiveReceipt.lean` in CI if it is to count as evidence. Until then, treat `B60`/`B65` as consistency checks of committed constants.
   **Affects a theorem statement:** no (the statements are true of the committed constants).

6. **Where:** `lean/BeastProofs.lean` `B29`, `B36`, `B47`, `B57` (7/9/11/12 DLLs); `B39`, `B48`, `B58`, `B63` (25/30/35/38 mutants); `B60` (v0.6.0 receipt).
   **What is wrong:** superseded counts are kept and counted among the `65`; each is a literal sum of numbers typed into the statement.
   **Correction:** move historical counts to the changelog and report the number of non-literal theorems separately (`26` of `65` have free variables or hypotheses).
   **Affects a theorem statement:** no.

### Other files not edited in this pass

1. **Where:** `VERIFICATION.md` line 13.
   **What is wrong:** gives the axiom split as `43` zero-axiom + `22` standard.
   **Correction:** `45` zero-axiom + `20` within `[propext, Classical.choice, Quot.sound]`.

2. **Where:** `LICENSE` line 3, `THIRD_PARTY_NOTICES.md` lines 19-20, `vendor/README.md` line 16.
   **What is wrong:** these lines name a limited-liability company that has not been formed.
   **Correction:** the copyright holder is the individual author, Alexander Stephen Donahue; the local engines are the author's own "0thernes" projects.

3. **Where:** `.github/workflows/ci.yml` line 103 (job `ams-latex-pdf`).
   **What is wrong:** the step is named "Install Tectonic 0.17.0" but downloads Tectonic 0.15.0.
   **Correction:** the step label should match the version downloaded (0.15.0).
