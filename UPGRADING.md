<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Upgrading & Deprecation Guide (`A41`)

`beast-civilization` follows Semantic Versioning (`MAJOR.MINOR.PATCH`) anchored in the single version source [`VERSION`](VERSION) and verified automatically by [`scripts/check_versions.py`](scripts/check_versions.py).

## Upgrading from `v0.7.0` to `v1.0.0`

### Breaking / Structural Changes
1. **License Transition (`MIT` → Dual `Apache-2.0` + `CC-BY-4.0`):** Code, Rust crates, Zig kernels, C11 bridges, CRuby scripts, and Lean 4 proofs are now licensed under **Apache-2.0** (with explicit patent grant); manuscripts, figures, and docs are licensed under **CC-BY-4.0** with REUSE 3.x metadata (`REUSE.toml`, `NOTICE`, `LICENSES/`).
2. **Modular Lean 4.35.0 Palomar 6-Kernel Layout (`lean/`):**
   - `lean/BeastProofs.lean` remains the top-level library re-export, while proofs are modularized under `lean/BeastProofs/{Spec,Quantization,CacheTelepathy,Ring0Security,ColonyGovernance,CryptoPhysics,SystemsRuntime,LiveReceipt,NonVacuity,AdversarialTests}.lean` with Lean 4.35.0 `module` headers and strict `autoImplicit = false`.
   - `beast verify-all --export-lean lean/BeastProofs/LiveReceipt.lean` now emits a `module` file directly compatible with `lake build`, `lake check --paranoid`, and `lake comparator --config comparator.json --paranoid`.
   - Theorem count increased from `65` (`B01`–`B65`) to **`81`** (`B01`–`B65` + `NV01`–`NV08` non-vacuity witnesses + `AT01`–`AT08` adversarial refutations).
3. **Hardened Parsers & Ring-0 Guards (Security Fixes):**
   - `Ring0Root::check_write_acl` now normalizes `./`, `//`, `\`, control bytes, and ASCII case (rejecting Windows case-bypass attempts).
   - `GgufFile::decode`, `audit_elf32_multiboot1`, `audit_elf64_multiboot2`, and `load_bcg` enforce checked arithmetic, array recursion depth (`<= 8`), and strict allocation ceilings.
   - `beast serve` adds `GET /healthz`, `405 Method Not Allowed`, strict `Content-Length <= 256 KiB` rejection (`413`), and UTF-8-safe JSON parsing.
   - `ruby/beast_yarv.rb` Prism AST gate now blocks `send`/`public_send`/`__send__`/`require`/`ConstantReadNode`/`ConstantPathNode` (`4/4` mutants blocked) and supports macOS `.dylib`.
   - `beast-cli` adds `beast stats-n24` for pre-registered 24-seed statistical replication (`scripts/verify_stats.py`).
4. **C-ABI Stability (`beast_metal.h`):** `BEAST_METAL_ABI_VERSION` remains `0x00010000` (backward-compatible binary layout with added bounds checks on `num_blocks <= 16` and `len_self/len_peer <= 512`).

## Deprecation Policy
- Public C-ABI symbols (`zig/beast-metal/include/beast_metal.h`) and CLI subcommands will remain backward-compatible across `1.x` releases; deprecated interfaces will emit clear notices for at least one minor release before removal in `2.0.0`.
