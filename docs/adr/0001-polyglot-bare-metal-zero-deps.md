<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# ADR-0001: Zero-Dependency Polyglot Bare-Metal Architecture

- **Status:** Accepted (`v1.0.0`, 2026-11-26)
- **Context:** Conventional AI agent frameworks pull hundreds of transitive npm/PyPI/Cargo packages, obscuring audit boundaries, introducing supply-chain vulnerabilities, and causing non-deterministic behavior across operating systems and runtimes.
- **Decision:** Build the entire `beast-civilization` stack with **zero external runtime dependencies**:
  - 8 Rust crates (`Cargo.toml` workspace) using `std` only (zero crates.io packages).
  - Zig 0.17.1 (`beast-metal`, freestanding ELF32/ELF64 kernels) using Zig `std` only.
  - CRuby 3.4.9 using built-in `prism`, `fiddle`, `digest`, and `rbconfig` only (zero gems).
  - Direct CUDA Driver API (`nvcuda.dll` / `libcuda.so.1`) launching hand-authored PTX without `nvcc` or MSVC.
  - Pure Lean 4.35.0 with zero Mathlib dependencies (`lean/lakefile.toml`).
  - Python 3 verification/audit/blueprint scripts using stdlib only.
- **Consequences:**
  - *Positive:* Instant offline builds, bit-exact cross-language Philox4x32-10 and SHA-256 parity, small auditable TCB, and immune to upstream package registry churn.
  - *Negative:* Binary/protocol parsers (`gguf.rs`, `elf_audit.rs`, `graft_io.rs`, `llm_server.rs`, `mcp_server.rs`) must be hand-hardened against malformed input, overflow, and DoS — mitigated directly by our 1,000-case fuzz & regression suite (`crates/beast-cli/tests/hardening_fuzz_props.rs`).
