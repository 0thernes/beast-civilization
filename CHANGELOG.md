<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Changelog (`A07`)

All notable changes to `beast-civilization` follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and Semantic Versioning (`VERSION`).

## [1.0.0] - 2026-11-26 — THREE-CYCLE HARDENING, 6-KERNEL PALOMAR LEAN 4.35.0, N=24 HOLM-BONFERRONI & ACADEMIC MONOGRAPH RELEASE

### Added
- **Six-Kernel Palomar Modular Lean 4.35.0 Suite (`81 / 81` Declarations):** Refactored `lean/` into `module`-based `BeastProofs/{LiveReceiptData,Spec,Quantization,CacheTelepathy,Ring0Security,ColonyGovernance,CryptoPhysics,SystemsRuntime,LiveReceipt,NonVacuity,AdversarialTests}.lean`, isolated `Challenge.lean` (importing only `Spec.lean`), sorry-free `Solution.lean`, `comparator.json` (`81` theorems, `10` definitions), `formalization.yaml` (v0.4), `lake-manifest.json`, `8` Non-Vacuity witnesses (`NV01`–`NV08`), `8` Adversarial refutations (`AT01`–`AT08`), `scripts/test_lean_negative.py` (`3/3` exploits rejected), `scripts/check_readme_axioms.py`, and 6-kernel Ubuntu `bubblewrap` CI with live sorry-tamper comparator rejection.
- **Pre-Registered `N=24` Paired Statistical Pipeline (`B28`–`B32`):** Added `beast stats-n24` CLI subcommand (`24` locked seeds `5000..5023`), `docs/preregistration.md`, and `scripts/verify_stats.py` (10,000-iter two-sided sign-flip permutation, Cohen's $d_z$, 2,000-resample 95% bootstrap CIs, step-down Holm-Bonferroni FWER $\alpha=0.05$) — **`4 / 4` primary hypotheses PASS** (`p_Holm = 0.00040`, `24/24` wins each) archived in `results/paired_stats_n24.txt`.
- **1,000-Case Adversarial Fuzzing, Regression & Property Suite:** Added `crates/beast-cli/tests/hardening_fuzz_props.rs` (`8` test suites covering Ring-0 ACL normalization, GGUF/ELF/`.bcg` overflow & recursion bombs, HTTP `413`/`405`/`/healthz`/UTF-8, MCP ID injection & SSRF, all `70/70` `C(8,4)` Planaria subsets, and `1,000`-case fuzzing) + `scripts/prop_check.py` (`3,000` randomized trials) + `scripts/check_sanitizers.sh` (C11 ASan/UBSan).
- **11-Page Elite AMS-LaTeX Monograph & Live Crossref Citation Audit:** Expanded `paper/refs.bib` to `46` scholarly references (`45/45` live-verified DOIs via Crossref/DOI Handle API, `0` retractions, archived in `results/citation-audit.json`) and upgraded `paper/main.tex` (`\bibliographystyle{plainurl}`, `0` warnings, `0` overfull hboxes, clean-room verified `dist/arxiv_submission.tar.gz`).
- **Interactive GitHub Pages Proof Blueprint (`A27`):** Added `scripts/build_blueprint.py` and `.github/workflows/pages.yml` publishing a zero-dependency WCAG 2.2 AA searchable 81-theorem SVG DAG portal.
- **Dual Open Licensing & Build Standard v1.2.0 / Playbook v1.3 Governance:** Migrated to dual `Apache-2.0` (code/Lean) + `CC-BY-4.0` (manuscript/docs) with `REUSE.toml`, `LICENSES/`, `NOTICE`, `repo.yaml`, `justfile`, `docs/{architecture,ai,benchmarks,configuration,faq,glossary,operations,threat-model,MEGA_ATLAS_CROSSCHECK,statement-fidelity,run-log,AI_DISCLOSURE}.md`, `docs/adr/0001..0003`, `examples/`, `evals/`, `HARDENING_LOG.md`, `SUBMISSION_READINESS.md`, `docs/reviews/2026-11-26-3-cycle-hardening-audit.md`, 3-OS CI (`ubuntu`/`windows`/`macos`), SLSA v4 attestations, and DigiCert RFC 3161 `.tsr` timestamp.

### Security & Bug Fixes (27 Remediated Findings Across 3 Hardening Cycles)
- **Ring-0 ACL Bypass (`HOLE-01`):** Hardened `check_write_acl` against `./`, `//`, `\`, NUL/control bytes, and case-insensitive Windows filesystem paths.
- **Parser OOM / Overflow / Recursion Bombs (`HOLE-02`–`HOLE-04`):** Bounded GGUF `n_tensors`/`n_kv`/`ndims` and `GGUF_TYPE_ARRAY` nesting (`depth <= 8`), fixed ELF `phoff + i * phentsize` overflow & `phentsize=0` spin, and bounded `.bcg` allocation multipliers.
- **HTTP/1.1, MCP & SSRF Hardening (`HOLE-05`, `HOLE-08`, `BUG-03`–`BUG-04`):** Guarded `Content-Length = u64::MAX` overflow (`413`), added `GET /healthz` and `405 Method Not Allowed`, fixed UTF-8 char-boundary truncation panics, escaped MCP JSON-RPC `id`, and blocked uppercase `LOCALHOST`, `0.0.0.0`, and CRLF/whitespace bypasses.
- **Zig C-ABI & CRuby Prism AST Hardening (`HOLE-06`–`HOLE-07`):** Added `BEAST_CHIMERA_MAX_BLOCKS` and `len_self/len_peer <= 512` guards in `root.zig`; blocked `send`/`public_send`/`__send__`/`require`/`binding` and `ConstantReadNode`/`ConstantPathNode` in `ruby/beast_yarv.rb` (`4/4` planted mutants blocked, `40/40` total runtime mutants).
- **Cross-Repo ABI, macOS `.dylib`, Lean Parser & Zero Clippy Warnings (`BUG-01`–`BUG-02`, `BUG-05`, `ERROR-01`):** Fixed `hyper_graph.rs` Zig header path (`4` repos / `94` syms / `31` edges), added macOS `.dylib` loader support, added Lean 4.35.0 `public theorem` extraction to `repo_map.rs` (`69` files / `416` syms), and eliminated all `cargo clippy --workspace --all-targets -- -D warnings` errors/warnings.

## [0.7.0] - 2026-11-26 — OPENAI/GGUF LIVE SERVER, AMS-LATEX WHITE PAPER & ARCHIVE RELEASE
### Added
- Standard `llama.cpp` GGUF v3 `Q4_0` / `F16` codec (`G31`), zero-crate OpenAI HTTP/1.1 server (`beast serve`) & SSRF-guarded endpoint client (`G32`), scripted REPL colony missions (`G33`), `65` Pure Lean 4.35.0 theorems (`VerifyAxioms.lean`), clean-room Python oracle, initial AMS-LaTeX preprint, and Linux CI.

## [0.6.0] - 2026-11-25 — THE OMNISCIENT HYPER-COLONY
### Added
- 12th live native DLL (`QGTL`), FIPS-197 AES-256 KAT & PQC-sealed C2C (`G21`, `G26`), 5-kernel Blackwell PTX (`+MLA 4×`, `+MoE Top-2`) + pinned async VRAM paging (`G11`), Eshkol tri-mode exact AD (`G27`), Cross-Repo ABI Graph & Confluent Worktrees (`G28`), MCP JSON-RPC 2.0 server (`G29`), Zig/LLVM x86-64 disassembly (`G30`), `60` Lean 4 theorems, and `296/300` verified agent matrix.

## [0.5.0] - 2026-11-25 — THE MATRIX SINGULARITY
### Added
- Compiled upstream `tsotchke/moonlab` FIPS-203 ML-KEM-768 PQC & `SbNN` Fibonacci Anyons/`p-bit` (`11/11` DLLs), Win32 Kernel Job Object sandbox (`128 MiB`), 4-layer GQA→Chimera C2C LLM, Autonomous Bare-Metal A-Life Forge (`862,208 B`), `50` Lean 4 theorems.

## [0.4.0] - 2026-11-25 — THE AUTONOMOUS MATRIX FORGE
### Added
- Live QEMU `11.1.0` Ring-0 bootable `i386` Multiboot1 AI microkernel, compiled upstream `libirrep` & `moonlab` QGT DLLs (`9/9` DLLs), 8th crate `crates/beast-agent`, `40` Lean 4 theorems.

## [0.3.0] - 2026-11-25
### Added
- 7th crate `crates/beast-model` (GGUF v3, cross-arch C2C + Planaria heal, TTS/ASR `.wav` + Sobel/`b1.58` OCR `.pgm`), live OS-process Ring-0 handoff, `beast repl`, `7/7` DLLs, ELF64/Multiboot2 auditor, `32` Lean 4 theorems.

## [0.2.0] - 2026-11-25
### Added
- 3-kernel RTX 5070 Ti PTX, analytical-gradient MIT `SwaT` trainer + `.bcg` `CacheGraft`, 120-gen Zig Chimera breeder (`KC-4`), Planaria $\mathrm{GF}(2^8)$+Merkle capsule, Multi-Colony OS + live `AUTOPHAGE` RWX JIT, upstream `quantum_rng` Bell DLL, CRuby/YARV 10-gen breeder, `24` Lean 4 theorems.

## [0.1.0] - 2026-11-25
### Added
- Initial `beast-civilization` polyglot monorepo (`Zig 0.17.1`, `Rust 1.94.1`, `CRuby 3.4.9/YARV`, `C11`, `CUDA sm_12.0 PTX`, `Pure Lean 4 v4.35.0`) and `300`-item agent matrix.
