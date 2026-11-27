<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Pre-Registered Statistical Hypotheses & Experimental Protocol (`B28`–`B32`)

- **Version Locked:** `v1.0.0` (`2026-11-26`, committed prior to final `v1.0.0` execution)
- **Seed Set (Locked):** `N = 24` paired deterministic seeds `S = {5000, 5001, ..., 5023}`
- **Execution Driver:** `bin/beast stats-n24` (compiled Zig `beast_metal` + Rust `beast-c2c` + `beast-colony`, zero mocks)
- **Statistical Engine:** `scripts/verify_stats.py`
  - **Primary Significance Test (`B29`):** Deterministic `10,000`-iteration two-sided paired sign-flip permutation test (seeded PRNG `20261126`) computing exact empirical $p_{\mathrm{raw}} = (1 + \#\{|T_{\mathrm{perm}}| \ge |T_{\mathrm{obs}}|\}) / (10000 + 1)$.
  - **Effect Sizes & Uncertainty (`B30`):** Paired mean difference $\bar{\Delta}$, fold-change ratio, win fraction ($k / 24$), Cohen's paired $d_z = \bar{\Delta} / s_{\Delta}$, and deterministic 2,000-resample 95% bootstrap confidence interval.
  - **Multiple-Comparison Correction (`B31`):** True step-down Holm-Bonferroni family-wise error rate (FWER) adjustment across all $m = 4$ primary hypotheses, with global significance threshold $\alpha_{\mathrm{FWER}} = 0.05$.

## Pre-Registered Primary Hypotheses (`H1`–`H4`)

| ID | Primary Comparison (Treatment vs. Matched Paired Control) | Primary Metric | Pre-Registered Direction | Falsification Criterion |
|---|---|---|---|---|
| **H1** | MIT `SwaT` `Augment-Cache` vs. Destructive `Overwrite-Cache` (100 grad steps, `32d → 24d`, disjoint test set) | Receiver self-context recall cosine similarity | $\cos_{\mathrm{Aug}} > \cos_{\mathrm{Ovr}}$ | Fail if $p_{\mathrm{Holm}} > 0.05$ or $\bar{\Delta} \le 0.50$ |
| **H2** | Zig Chimera 60-Gen Evolutionary Breeder (`KC-4`, pop 16, `||W||_2 <= 0.99`) vs. Matched 6-Block Random Control | Disjoint held-out sequence MSE (`domain 0x8200`) | $\mathrm{MSE}_{\mathrm{Evo}} < \mathrm{MSE}_{\mathrm{Ctrl}}$ | Fail if $p_{\mathrm{Holm}} > 0.05$ or $\bar{\Delta} \ge 0$ |
| **H3** | Zig `0.17.1` `@Vector` SIMD `b1.58` Dot-Product vs. Bit-Exact Scalar Reference (`2M` trits/seed, 0-bit diff enforced) | Wall-clock execution latency ($\mu\text{s}$) | $t_{\mathrm{SIMD}} < t_{\mathrm{Ref}}$ | Fail if $p_{\mathrm{Holm}} > 0.05$, any bit mismatch, or mean speedup $< 1.5\times$ |
| **H4** | Moonlab Cosine-Schedule QUBO Annealer (`200` sweeps, 6-caste conflict matrix) vs. Initial Unannealed Random State | QUBO Hamiltonian energy $E(x)$ | $E_{\mathrm{Best}} < E_{\mathrm{Init}}$ | Fail if $p_{\mathrm{Holm}} > 0.05$ or $E_{\mathrm{Best}} > E_{\mathrm{Init}}$ on any seed |
