#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
"""Live Negative Lean 4.35.0 Tamper Rejection Test (Playbook v1.3 / Build Standard B06).
Verifies our strict axiom parser mechanically REJECTS three live exploit artifacts:
  1. A theorem proved via `sorry` (`sorryAx`)
  2. A theorem proved via a custom `axiom`
  3. A theorem proved via `native_decide` (`Lean.ofReduceBool` / `Lean.trustCompiler`)"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ALLOWED = {"propext", "Classical.choice", "Quot.sound"}
FORBIDDEN = ("sorryAx", "ofReduceBool", "trustCompiler", "native_decide", "bv_decide")


def find_lean() -> str:
    env_p = os.environ.get("LEAN_BIN")
    if env_p and Path(env_p).exists():
        return env_p
    for cand in [
        shutil.which("lean"),
        str(Path.home() / ".elan" / "toolchains" / "leanprover--lean4---v4.35.0" / "bin" / ("lean.exe" if sys.platform == "win32" else "lean")),
    ]:
        if cand and Path(cand).exists():
            return cand
    raise FileNotFoundError("lean v4.35.0 binary not found")


def audit_output_is_clean(out: str) -> bool:
    lines = [ln.strip() for ln in out.splitlines() if ln.strip()]
    if not lines:
        return False
    for ln in lines:
        if any(bad in ln for bad in FORBIDDEN):
            return False
        m0 = re.match(r"^'([^']+)' does not depend on any axioms$", ln)
        m1 = re.match(r"^'([^']+)' depends on axioms: \[([^\]]*)\]$", ln)
        if not (m0 or m1):
            return False
        if m1:
            axs = {a.strip() for a in m1.group(2).split(",") if a.strip()}
            if not axs.issubset(ALLOWED):
                return False
    return True


def main() -> int:
    lean = find_lean()
    exploits = [
        ("sorry_exploit", "theorem bad_sorry : 1 = 2 := by sorry\n#print axioms bad_sorry\n"),
        ("custom_axiom_exploit", "axiom evilAxiom : False\ntheorem bad_ax : 1 = 2 := False.elim evilAxiom\n#print axioms bad_ax\n"),
        ("native_decide_exploit", "theorem bad_native : 2 + 2 = 4 := by native_decide\n#print axioms bad_native\n"),
    ]
    tmp = Path(tempfile.mkdtemp(prefix="beast_lean_neg_"))
    try:
        rejected = 0
        for name, src in exploits:
            fp = tmp / f"{name}.lean"
            fp.write_text(src, encoding="utf-8")
            res = subprocess.run([lean, str(fp)], capture_output=True, text=True, check=False)
            combined = res.stdout + "\n" + res.stderr
            if audit_output_is_clean(combined):
                print(f"FAIL: Negative Lean tamper test '{name}' was NOT rejected!\n{combined}", file=sys.stderr)
                return 1
            rejected += 1
        print(f"[PASS] Live Negative Lean 4.35.0 Tamper Tests: {rejected}/{len(exploits)} exploits mechanically rejected (sorry, custom axiom, native_decide)")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
