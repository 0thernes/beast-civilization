#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
"""Verify that README.md contains the verbatim `#print axioms` audit block matching
`results/lean4-audit.txt` line-for-line (Playbook v1.3 Gold Standard Check)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    audit_path = ROOT / "results" / "lean4-audit.txt"
    readme_path = ROOT / "README.md"
    if not audit_path.exists():
        print("FAIL: results/lean4-audit.txt missing", file=sys.stderr)
        return 1
    expected = [ln.strip() for ln in audit_path.read_text(encoding="utf-8").splitlines() if ln.strip()]
    readme = readme_path.read_text(encoding="utf-8")
    m = re.search(r"<!-- LEAN4_AXIOMS_BEGIN -->\s*```text\s*(.*?)\s*```\s*<!-- LEAN4_AXIOMS_END -->", readme, re.DOTALL)
    if not m:
        print("FAIL: Verbatim <!-- LEAN4_AXIOMS_BEGIN --> block missing in README.md", file=sys.stderr)
        return 1
    actual = [ln.strip() for ln in m.group(1).splitlines() if ln.strip()]
    if actual != expected:
        print(f"FAIL: README.md verbatim axioms ({len(actual)} lines) != results/lean4-audit.txt ({len(expected)} lines)", file=sys.stderr)
        return 1
    print(f"[PASS] README.md verbatim `#print axioms` block matches results/lean4-audit.txt ({len(actual)}/81 lines exact)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
