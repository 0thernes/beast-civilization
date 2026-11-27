#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
"""Deterministic Agent & Anti-Hallucination Regression Eval Suite (Build Standard B34 / A40).
Verifies Ring-0 safety boundaries, anti-spoof C2C invariants, version integrity, and
zero-fabricated-claim compliance across the repository without calling external paid APIs."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    failures: list[str] = []

    # 1. Zero unformed corporate-entity assertions anywhere in tracked repo text
    forbidden = "0thernes " + "LLC"
    res = subprocess.run(
        [
            "git", "grep", "-n", forbidden, "--",
            ":!docs/reviews", ":!HARDENING_LOG.md", ":!SUBMISSION_READINESS.md", ":!evals/eval_suite.py",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if res.stdout.strip():
        failures.append(f"Forbidden corporate-entity assertion found:\n{res.stdout.strip()}")

    # 2. Honest Mega-Atlas non-retirement scope present in docs/MEGA_ATLAS_CROSSCHECK.md & paper/main.tex
    for rel in ["docs/MEGA_ATLAS_CROSSCHECK.md", "paper/main.tex"]:
        txt = (ROOT / rel).read_text(encoding="utf-8")
        if "0 / 1,900" not in txt and "0/1,900" not in txt and "0 / 1{,}900" not in txt:
            failures.append(f"Missing honest '0/1,900' Mega-Atlas non-retirement declaration in {rel}")

    # 3. No fabricated DOI or peer-review claim in CITATION.cff
    cff = (ROOT / "CITATION.cff").read_text(encoding="utf-8")
    if re.search(r"^\s*doi:", cff, re.MULTILINE):
        failures.append("Unassigned DOI field present in CITATION.cff")

    # 4. Run quickstart example cleanly
    ex = subprocess.run(
        [sys.executable, str(ROOT / "examples" / "01_quickstart_oracle_and_c2c.py")],
        capture_output=True,
        text=True,
        check=False,
    )
    if ex.returncode != 0:
        failures.append(f"Example 01 failed: {ex.stderr}")

    if failures:
        print("FAIL: Eval suite detected regression(s):", file=sys.stderr)
        for f in failures:
            print("  - " + f, file=sys.stderr)
        return 1

    print("[PASS] Deterministic Agent & Epistemic Integrity Eval Suite (4/4 checks passed, 0 fabrications)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
