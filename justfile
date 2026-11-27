# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
# Task runner for BEAST CIVILIZATION (Build Standard A31 / B01 / B16 / B17 / B18)

set windows-shell := ["pwsh.exe", "-NoProfile", "-Command"]

# Full clean-clone reproducible verification pipeline (B01)
reproduce: test prop oracle lean paper site audit

# Build native Zig + Rust release artifacts
build:
    zig build-lib zig/beast-metal/src/root.zig -dynamic -OReleaseFast -femit-bin=bin/beast_metal.dll
    cargo build --release --bin beast

# Run Zig, Rust unit + integration/fuzz/property, and CRuby/Prism tests
test:
    zig test zig/beast-metal/src/root.zig
    cargo test --workspace
    ruby -w ruby/beast_yarv.rb

# Strict static analysis & linting (B17 / B18 — zero warnings)
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# 1,000-case deterministic parser fuzzing suite (B15)
fuzz:
    cargo test --workspace --test hardening_fuzz_props fuzz_1000_cases

# C11 bridge ASan/UBSan sanitizer check on Linux/POSIX (B16)
sanitize:
    zig cc -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined -O1 -g -c c_bridges/tsotchke_pinn_bridge.c -Ivendor/tsotchke__PINN/include -o /tmp/pinn_san.o

# Clean-room Python exact-arithmetic oracle (B12)
oracle:
    python scripts/verify_independent_oracle.py

# Property-based randomized verifier (B10)
prop:
    python scripts/prop_check.py

# Pre-registered N=24 paired statistical evaluation with Holm-Bonferroni FWER (B28-B32)
stats:
    python scripts/verify_stats.py

# Palomar Lean 4.35.0 build, axiom audit, negative tamper tests & README parity (B22)
lean:
    cd lean && lake build && lake build Challenge && lake env lean VerifyAxioms.lean
    python scripts/test_lean_negative.py
    python scripts/check_readme_axioms.py

# Compile AMS-LaTeX white paper + clean-room arXiv tarball + live Crossref citation audit
paper:
    python scripts/check_citations.py --write-json results/citation-audit.json
    python scripts/build_paper.py

# Build static GitHub Pages interactive Proof Blueprint portal (A27)
site:
    python scripts/build_blueprint.py

# Audit repository against Build Standard v1.2.0 + version consistency
audit:
    python scripts/check_versions.py
    pwsh -NoProfile -File scripts/Audit-Repo.ps1
