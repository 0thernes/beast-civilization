#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
"""Single-source-of-truth version consistency checker across ALL tracked manifests,
headers, proofs, scripts, and paper sources (Build Standard A08)."""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def read_text(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def main() -> int:
    ver_file = ROOT / "VERSION"
    if not ver_file.exists():
        print("FAIL: VERSION file missing at repository root.", file=sys.stderr)
        return 1
    version = ver_file.read_text(encoding="utf-8").strip()
    if not re.match(r"^\d+\.\d+\.\d+$", version):
        print(f"FAIL: VERSION '{version}' is not SemVer X.Y.Z", file=sys.stderr)
        return 1

    checks: list[tuple[str, str]] = [
        ("Cargo.toml", f'version = "{version}"'),
        ("ruby/beast_yarv.rb", f"v{version}"),
        ("lean/lakefile.toml", f'version = "{version}"'),
        ("lean/formalization.yaml", f"v{version}"),
        ("CITATION.cff", f'version: "{version}"'),
        ("CITATION.bib", f"v{version}"),
        (".zenodo.json", f'"version": "{version}"'),
        ("repo.yaml", f"v{version}"),
        ("CHANGELOG.md", f"## [{version}]"),
        ("README.md", f"v{version}"),
        ("paper/main.tex", f"v{version}"),
    ]

    failures: list[str] = []
    for rel, needle in checks:
        p = ROOT / rel
        if not p.exists():
            failures.append(f"MISSING {rel}")
            continue
        txt = read_text(rel)
        if needle.lower() not in txt.lower():
            failures.append(f"MISMATCH in {rel}: expected '{needle}'")

    if failures:
        print(f"FAIL: Version consistency check failed for VERSION={version}:", file=sys.stderr)
        for f in failures:
            print(f"  - {f}", file=sys.stderr)
        return 1

    print(f"PASS: All {len(checks)} versioned manifests, proofs, scripts & paper files match VERSION={version}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
