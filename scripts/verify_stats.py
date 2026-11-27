#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
"""Pre-Registered N=24 Multi-Seed Paired Statistical Verifier (Build Standard B28-B32).

Invokes the compiled native binary `beast stats-n24` across all 24 locked paired seeds
(5000..5023) and computes:
  - Paired mean differences, win fractions, fold-change ratios
  - Cohen's paired d_z and deterministic 2,000-resample 95% bootstrap CIs
  - Deterministic 10,000-iteration two-sided paired sign-flip permutation p-values
  - True step-down Holm-Bonferroni FWER adjusted p-values (p_Holm <= 0.05)

Writes verbatim audit report to `results/paired_stats_n24.txt`."""

from __future__ import annotations

import os
import random
import statistics
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def find_beast_bin() -> Path:
    env_p = os.environ.get("BEAST_BIN")
    if env_p and Path(env_p).is_file():
        return Path(env_p)
    ext = ".exe" if sys.platform == "win32" else ""
    for cand in [
        ROOT / "bin" / f"beast{ext}",
        ROOT / "target" / "release" / f"beast{ext}",
        ROOT / "target" / "debug" / f"beast{ext}",
    ]:
        if cand.is_file():
            return cand
    raise FileNotFoundError("Compiled beast CLI binary not found; build with cargo build --release --bin beast")


def paired_sign_flip_p(diffs: list[float], n_perm: int = 10_000, seed: int = 20261126) -> float:
    rng = random.Random(seed)
    obs = abs(sum(diffs) / len(diffs))
    ge = 0
    for _ in range(n_perm):
        s = sum(d if rng.getrandbits(1) else -d for d in diffs) / len(diffs)
        if abs(s) >= obs - 1e-15:
            ge += 1
    return (ge + 1) / (n_perm + 1)


def bootstrap_ci_95(diffs: list[float], n_boot: int = 2000, seed: int = 20261127) -> tuple[float, float]:
    rng = random.Random(seed)
    n = len(diffs)
    means = sorted(sum(diffs[rng.randrange(n)] for _ in range(n)) / n for _ in range(n_boot))
    lo = means[int(0.025 * n_boot)]
    hi = means[int(0.975 * n_boot)]
    return lo, hi


def holm_bonferroni(p_vals: list[float]) -> list[float]:
    m = len(p_vals)
    order = sorted(range(m), key=lambda i: p_vals[i])
    adj = [0.0] * m
    running_max = 0.0
    for rank, idx in enumerate(order):
        val = min(1.0, p_vals[idx] * (m - rank))
        running_max = max(running_max, val)
        adj[idx] = running_max
    return adj


def main() -> int:
    beast = find_beast_bin()
    res = subprocess.run([str(beast), "stats-n24"], cwd=ROOT, capture_output=True, text=True, check=False)
    if res.returncode != 0:
        print(f"FAIL: beast stats-n24 exited {res.returncode}:\n{res.stderr}", file=sys.stderr)
        return res.returncode

    lines = [ln.strip() for ln in res.stdout.splitlines() if ln.strip()]
    header = lines[0].split("\t")
    rows = [dict(zip(header, ln.split("\t"))) for ln in lines[1:]]
    if len(rows) != 24:
        print(f"FAIL: Expected 24 paired seeds, got {len(rows)}", file=sys.stderr)
        return 1

    h_specs = [
        (
            "H1",
            "MIT SwaT Augment vs. Destructive Overwrite (self-recall cos)",
            [float(r["swat_aug_cos"]) for r in rows],
            [float(r["swat_ovr_cos"]) for r in rows],
            lambda a, b: a - b,
            "higher_better",
        ),
        (
            "H2",
            "Zig Chimera 60-Gen Breeder vs. Matched Random Control (held-out MSE reduction)",
            [float(r["chimera_ctrl_mse"]) for r in rows],
            [float(r["chimera_evo_mse"]) for r in rows],
            lambda ctrl, evo: ctrl - evo,
            "lower_evo_better",
        ),
        (
            "H3",
            "Zig b1.58 SIMD vs. Scalar Reference (2M trits/seed speedup, 0-bit exact)",
            [float(r["b158_ref_us"]) for r in rows],
            [float(r["b158_simd_us"]) for r in rows],
            lambda ref, simd: ref - simd,
            "lower_simd_better",
        ),
        (
            "H4",
            "Moonlab Cosine-Schedule QUBO Annealer vs. Unannealed Initial State (energy drop)",
            [float(r["qubo_init_e"]) for r in rows],
            [float(r["qubo_best_e"]) for r in rows],
            lambda init_e, best_e: init_e - best_e,
            "lower_best_better",
        ),
    ]

    results = []
    raw_ps = []
    for hid, desc, a_vals, b_vals, diff_fn, mode in h_specs:
        diffs = [diff_fn(a, b) for a, b in zip(a_vals, b_vals)]
        mean_a = statistics.mean(a_vals)
        mean_b = statistics.mean(b_vals)
        mean_d = statistics.mean(diffs)
        sd_d = statistics.stdev(diffs)
        dz = mean_d / sd_d if sd_d > 1e-15 else float("inf")
        wins = sum(1 for d in diffs if d > 0.0)
        p_raw = paired_sign_flip_p(diffs)
        ci_lo, ci_hi = bootstrap_ci_95(diffs)
        if mode == "higher_better":
            ratio_str = f"delta={mean_d:+.4f}"
        elif mode == "lower_evo_better":
            ratio_str = f"{mean_a / max(mean_b, 1e-12):.2f}x lower MSE"
        elif mode == "lower_simd_better":
            ratio_str = f"{mean_a / max(mean_b, 1e-12):.2f}x faster"
        else:
            ratio_str = f"delta_E={mean_d:+.4f}"
        raw_ps.append(p_raw)
        results.append((hid, desc, mean_a, mean_b, mean_d, ci_lo, ci_hi, dz, wins, p_raw, ratio_str))

    holm_ps = holm_bonferroni(raw_ps)

    out_lines = [
        "=========================================================================================",
        "  BEAST CIVILIZATION v1.0.0 — PRE-REGISTERED N=24 PAIRED STATISTICAL EVALUATION (B28-B32)",
        "  Seeds: 5000..5023 (N=24) | 10,000-iter two-sided sign-flip | Step-down Holm-Bonferroni",
        "=========================================================================================",
    ]
    all_pass = True
    for (hid, desc, ma, mb, md, lo, hi, dz, wins, pr, rstr), ph in zip(results, holm_ps):
        ok = (ph <= 0.05) and (wins >= 20)
        if not ok:
            all_pass = False
        out_lines.append(
            f"[{'PASS' if ok else 'FAIL'}] {hid}: {desc}\n"
            f"       mean_A={ma:.6f}  mean_B={mb:.6f}  mean_diff={md:+.6f}  "
            f"95%_CI=[{lo:+.6f}, {hi:+.6f}]\n"
            f"       Cohen_dz={dz:.3f}  wins={wins}/24  effect=({rstr})  "
            f"p_raw={pr:.5f}  p_Holm={ph:.5f}"
        )

    out_lines.append("-----------------------------------------------------------------------------------------")
    out_lines.append(f"GLOBAL FWER VERDICT (alpha=0.05): {'4/4 HYPOTHESES SIGNIFICANT (PASS)' if all_pass else 'FAILED'}")
    out_lines.append("")
    out_lines.append("RAW PER-SEED TABLE (seeds 5000..5023):")
    out_lines.extend(lines)

    report = "\n".join(out_lines) + "\n"
    out_path = ROOT / "results" / "paired_stats_n24.txt"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(report, encoding="utf-8", newline="\n")
    print(report)
    return 0 if all_pass else 1


if __name__ == "__main__":
    raise SystemExit(main())
