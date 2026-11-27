<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Threat Model (`B36`)

This document enumerates the trust boundaries, adversary capabilities, enforced controls, and residual risks of `BEAST CIVILIZATION v1.0.0`.

## Assets & Trust Boundaries
1. **Ring-0 Guardian (`crates/beast-root`):** Holds signing keys, charter hash, spend cap, snapshot history, and `BEAST_API_KEY` secrets. Must remain strictly unreachable from agent-zone self-edits or spawned subprocesses.
2. **Formal Proof Corpus (`lean/`):** Must never be weakened by agent edits, custom axioms, `sorry`, `native_decide`, or specification mismatch between `Challenge.lean` and `Solution.lean`.
3. **Binary & Protocol Parsers (`gguf.rs`, `elf_audit.rs`, `graft_io.rs`, `hbin_capsule.rs`, `llm_server.rs`, `mcp_server.rs`, `llm_client.rs`):** Process untrusted files, HTTP bodies, URLs, and JSON-RPC frames.
4. **Embedded Strategy VM (`ruby/beast_yarv.rb`):** Compiles evolved Ruby strategy lambdas to YARV bytecode; must block arbitrary code execution before compilation.
5. **Cross-Language C-ABI (`zig/beast-metal/src/root.zig`, `c_bridges/`):** Raw pointer/struct boundaries crossing Rust ↔ Zig / C11.

## Threat Matrix & Mitigations

| Threat | Attack Vector | Enforced Mitigation | Verification Gate / Test |
|---|---|---|---|
| **T1: Ring-0 Self-Edit Bypass** | Path traversal (`../`), `./` prefix, `//`, Windows case bypass (`CRATES/BEAST-ROOT`), NUL bytes | Normalized case-insensitive ACL in `Ring0Root::check_write_acl`; `task_requests_ring0_violation` | `G02`, `G20`, `G33`, `regression_ring0_acl_*`, Lean `B09` |
| **T2: Secret Exfiltration via Child Process** | Reading parent `BEAST_API_KEY` / `OPENAI_API_KEY` from spawned worker | `env_clear()` + secret substring scrub (`API_KEY`, `SECRET`, `TOKEN`, `PASSWORD`, `PRIVATE_KEY`, `CREDENTIAL`) + Win32 Job Object (`128 MiB`, `KILL_ON_JOB_CLOSE`) | `G15`, `G23`, Lean `B28`, `B43` |
| **T3: SSRF / CRLF Injection** | Redirecting `beast agent-run --endpoint` to `169.254.169.254`, `0.0.0.0`, `file://`, or injecting `\r\n` headers | Strict `http://` + case-insensitive loopback allowlist (`127.0.0.1`, `localhost`, `[::1]`) unless `--allow-remote`; rejects all control bytes & whitespace | `G32`, `regression_mcp_id_injection_and_ssrf_guards`, Lean `B62` |
| **T4: Malformed Binary DoS / OOM / Overflow** | Crafted GGUF (`n_tensors=0xFFFFFFFF`, nested `GGUF_TYPE_ARRAY` bomb), crafted ELF (`phoff=MAX`, `phentsize=0`), crafted `.bcg` (`seq*dim` wrap), HTTP `Content-Length=u64::MAX` | Checked arithmetic (`checked_mul`/`checked_add`), recursion depth cap (`<= 8`), allocation caps, `MAX_HTTP_BODY_BYTES = 256 KiB` (`413`) | `hardening_fuzz_props.rs` (`1,000`-case fuzz + 6 regression tests) |
| **T5: Byzantine Data / Model Poisoning** | Corrupting `.gguf`, `.bcg`, microkernel ELF, or Planaria capsule shards | Domain-separated SHA-256 Merkle verification quarantines forged shards BEFORE $\mathrm{GF}(2^8)$ Cauchy inversion; ML-KEM-768 Fujisaki-Okamoto implicit rejection | `G05`, `G06`, `G13`, `G19`, `G21`, `property_planaria_all_70_subsets` |
| **T6: Ruby Strategy Sandbox Escape** | `system()`, backticks, `public_send(:system, ...)`, `File.read(...)`, `Kernel`/`Fiddle` access | Prism AST walker rejects forbidden calls, dynamic dispatch, backticks, AND all `ConstantReadNode`/`ConstantPathNode` BEFORE YARV compilation | `ruby/beast_yarv.rb` (`4/4` planted mutants) |
| **T7: Cloud-Spoofed C2C Claim ("LARP")** | Tagging remote HTTP text call as `is_local_c2c = 1` | `Bc2cPacketHeader::validate` rejects `TextCloudFallback` or `CloudFrontier` with `is_local_c2c != 0` | `G03`, Lean `B08`, `AT02` |
| **T8: Formal Proof / Axiom Tampering** | Injecting `sorry`, custom `axiom`, `native_decide`, or weakening `Challenge.lean` statements | 6-kernel Palomar CI (`lake check --paranoid`, `lake comparator --paranoid`) + live sorry-tamper comparator test + `test_lean_negative.py` + `check_readme_axioms.py` | CI `lean-6-kernel` job |
| **T9: Zig C-ABI Out-of-Bounds** | Calling `beast_chimera_forward` with `num_blocks > 16` or `beast_c2c_augment_attn_f32` with overflowing `len_self + len_peer` | Explicit bounds checks (`num_blocks <= 16`, `len_self/len_peer <= 512`, `isFinite(max_norm)`) at the Zig C-ABI boundary | `zig test`, `G09` |

## Residual Risks (Honest Scope)
- Native Rust/Zig/C11 memory safety is tested empirically (unit tests, fuzzing, ASan/UBSan, Clippy `-D warnings`) and is NOT proved inside the Lean 4 kernel.
- Win32 Job Objects enforce memory & job-close kill limits on Windows; POSIX runners use process environment scrubbing (`G15`) without `seccomp-bpf` containers.
- `beast serve` is designed for local loopback inference (`127.0.0.1`), not direct exposure to hostile public WAN traffic.
