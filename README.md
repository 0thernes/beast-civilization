# BEAST CIVILIZATION (`v0.1.0`)

**Bare-metal, self-rewriting, multi-colony AI civilization and evolutionary compute substrate in Zig `0.17.1` + Rust `1.94.1` + CRuby `3.4.9`/YARV + C11 (`AUTOPHAGE`) + Blackwell `sm_12.0` CUDA PTX + Pure Lean 4 `v4.35.0`.**

> **Author:** Alexander Stephen Donahue ([0thernes LLC](https://0thernes.art) · `0_0@0thernes.art` · ORCID [`0009-0006-8168-9648`](https://orcid.org/0009-0006-8168-9648))  
> **Target Profiles:** `T13` (OS kernel / bare-metal) · `T07` (AI/LLM systems) · `T05` (Library/SDK) · `T04` (CLI) — **Zero SaaS / Zero PWA / Zero External Crate or Gem Dependencies.**  
> **Fused Lineage:** `nagisanzenin/{INIT-Agent,nonhuman,engram,effortmining,nagi,strictlybetter,crucible,math-first}` · `temm1e-labs/{temm1e,temrust}` · `tsotchke/{eshkol,moonlab,quantum_geometric_tensor,quantum_rng,rosette-wire,libirrep,PINN,spin_based_neural_network,tensorcore}` · `0thernes/{autophage,cellular-engine-omega,cellulogonic-mecha,noetic-organism-omega,planaria,alife-singularity-omega-package}` · **MIT & ICLR 2026 Cache-to-Cache / Swarm Telepathy (`arXiv:2510.03215`, `arXiv:2610.15095`, `Augment-Cache / Replace-Activation`, `CacheGraft`)**.

---

## Live Verified Results (Measured on Intel Core Ultra 9 275HX + NVIDIA RTX 5070 Ti `sm_12.0` · Win64)

Every metric below comes from an actual execution of `.\build.ps1` recorded in [`results/verify_win64.txt`](results/verify_win64.txt) and [`results/lean4-audit.txt`](results/lean4-audit.txt):

| Verification Pillar | Live Measured Result | Evidence Artifact |
|---|---|---|
| **Zig `0.17.1` `beast-metal` (`u1`/`b1.58`/`i2`/`NVFP4`)** | **`4.57×` SIMD speedup** over scalar (`10M` trits: `6554 µs → 1434 µs`), **0-bit diff**, **`3/3` planted mutants caught**, **`6.34 ns/call`** Rust↔Zig FFI | `zig/beast-metal/src/root.zig`, `results/verify_win64.txt` |
| **Freestanding `x86_64` Multiboot2 OS Microkernel** | Compiled zero-libc `ELF64` binary (**`13,448 bytes`**) with verified `0xE85250D6` Multiboot2 header + embedded `b1.58` ternary reflex core | `zig/freestanding_kernel/src/kernel.zig`, `bin/beast_microkernel.elf` |
| **NVIDIA RTX 5070 Ti Blackwell `sm_12.0` PTX (`46 SMs`)** | Direct `nvcuda.dll` PTX JIT (zero MSVC, zero nvcc): **`4096 × 640` (`2.62M` trits) in `176 µs` (~`14.9B trits/s`)**, **`bit_exact_cpu=true`** | `cuda/beast_blackwell.ptx`, `crates/beast-cuda/src/lib.rs` |
| **MIT Cache-to-Cache / Swarm Telepathy (`beast-c2c`)** | Cross-family `Qwen3 (96d) → TemRust (64d)` **`4.30×` faster** than text prefill (`2662 µs → 619 µs`), `self_kv_kept=64/80`, Rust==Zig `max_diff=1.49e-8`, Cloud-spoof mutant caught | `crates/beast-c2c/src/lib.rs` |
| **Ring-0 Immutable Root (`INIT-Agent` Win64+Linux + `Noetic`)** | ACL blocks Ring-0/Lean/path-traversal overwrites, hard spend cap, `G0–G4` hot-swap + **0-bit SHA-256 rollback**, HMAC-SHA256 Rosette-Wire lineage | `crates/beast-root/src/lib.rs`, `crates/beast-wire/src/lib.rs` |
| **Colony OS Math + Tsotchke Quantum/Geo + `AUTOPHAGE` FFI** | `TEMM1E` Wilson `30/30 = 0.818891`, `NonHuman` `-25.5%` batching savings, `Engram` FSRS-4.5, `QUBO E=-7.20`, `QGTL g_tt=0.2500 / Berry=-0.4330`, `QRNG CHSH S=2.8464 > 2.0`, live `AUTOPHAGE v1.3` DLL (`AVX2/GFNI/PCLMUL/RDSEED/RWX_JIT=true`) | `crates/beast-colony/src/` |
| **Chimera Frankenstein Insert-Network Evolution** | Live `OP_INSERT_BLOCK` (ReZero loss-neutral), `OP_PREC_MORPH`, `OP_CROSS_XENOGRAFT`, `OP_SPECTRAL_CLAMP` (`‖W‖₂ = 0.2422 ≤ 0.99`), `8` heterogeneous blocks (`b1.58`, `NVFP4`, `sLTC`, `SSM`, `GF(2)`, `QGTL`, `C2C`) | `zig/beast-metal/src/root.zig`, `crates/beast-cli/src/main.rs` |
| **CRuby `3.4.9` + Prism AST + YARV Live Mutation** | Live `Gen0 → Gen1` YARV binary bytecode mutation (`728 B / 30 ops → 948 B / 45 ops`, score `0.7714 → 0.864`), **`2/2` injection mutants blocked by Prism**, **`1,623.1 ns/call`** CRuby→Zig Fiddle FFI | `ruby/beast_yarv.rb` |
| **Pure Lean 4 `v4.35.0` Formal Proofs** | **`16 / 16` theorems PROVEN** (`12` zero-axiom + `4` `[propext]` only; **`0 sorry`, `0 admit`, `0` custom axioms, `0 bv_decide`, `0 warnings`**) including live hardware receipt **`B16_live_hardware_receipt`** | `lean/BeastProofs.lean`, `results/lean4-audit.txt` |
| **Total Planted Negative Mutants Caught** | **`13 / 13` CAUGHT** (`3` Zig + `8` Rust/CLI + `2` CRuby/Prism) + `11 / 11` Rust unit tests passing | `results/verify_win64.txt` |

---

## Quickstart (`A03`)

```powershell
# One-command full 6-layer build, test, GPU PTX verify, YARV bridge, and Lean 4 audit:
.\build.ps1

# Run individual bare-metal CLI commands:
.\bin\beast.exe probe
.\bin\beast.exe verify-all --export-lean lean\LiveReceipt.lean
.\bin\beast.exe evolve-chimera
.\bin\beast.exe bench
ruby ruby\beast_yarv.rb
```

---

## Repository Map

- [`AGENTS.md`](AGENTS.md) — Canonical agent constitution (`CLAUDE.md`, `GEMINI.md`, `.cursor/rules/build-standard.mdc` delegate here).
- [`CLAIMS.md`](CLAIMS.md) — Falsifiable claims ledger mapping every claim to its exact command & receipt.
- [`LIMITATIONS.md`](LIMITATIONS.md) — Explicit real-vs-metaphor boundary and what would prove us wrong.
- [`REPLICATION.md`](REPLICATION.md) — Clean-clone pinned toolchain replication steps.
- [`docs/matrices/300_agent_matrix.yaml`](docs/matrices/300_agent_matrix.yaml) — Machine-checkable `100 Problems × 100 Wishlists × 100 Features` registry.
- [`vendor/VENDOR_MANIFEST.json`](vendor/VENDOR_MANIFEST.json) — Exact Git commit SHA pins for all `19` upstream GitHub repos + `6` local 0thernes engines.
- [`zig/beast-metal/`](zig/beast-metal/) & [`zig/freestanding_kernel/`](zig/freestanding_kernel/) — Zig `0.17.1` silicon kernels, Chimera Insert-Network engine, and freestanding `x86_64` microkernel.
- [`crates/`](crates/) — Zero-dependency Rust 6-crate workspace (`beast-wire`, `beast-root`, `beast-c2c`, `beast-colony`, `beast-cuda`, `beast-cli`).
- [`cuda/beast_blackwell.ptx`](cuda/beast_blackwell.ptx) — Bare-metal PTX GPU kernel JIT-loaded directly onto RTX 5070 Ti via `nvcuda.dll`.
- [`ruby/beast_yarv.rb`](ruby/beast_yarv.rb) — CRuby `3.4.9` + Prism AST + YARV live bytecode mutation (`T1a`) + Zig Fiddle C-ABI.
- [`lean/BeastProofs.lean`](lean/BeastProofs.lean) — Pure Lean 4 (`v4.35.0`) formal proof suite + live runtime receipt.
