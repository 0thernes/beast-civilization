# BEAST CIVILIZATION (`v0.2.0`)

**Bare-metal, self-rewriting, multi-colony AI civilization and evolutionary compute substrate in Zig `0.17.1` + Rust `1.94.1` + CRuby `3.4.9`/YARV + C11 (`AUTOPHAGE` + `tsotchke/quantum_rng`) + NVIDIA RTX 5070 Ti Blackwell `sm_12.0` 3-Kernel PTX + Pure Lean 4 `v4.35.0`.**

> **Author:** Alexander Stephen Donahue ([0thernes LLC](https://0thernes.art) · `0_0@0thernes.art` · ORCID [`0009-0006-8168-9648`](https://orcid.org/0009-0006-8168-9648))  
> **Target Profiles:** `T13` (OS kernel / bare-metal) · `T07` (AI/LLM systems) · `T05` (Library/SDK) · `T04` (CLI) — **Zero SaaS / Zero PWA / Zero External Cargo Crates or Ruby Gems.**  
> **Fused Lineage:** `nagisanzenin/{INIT-Agent,nonhuman,engram,effortmining,nagi,strictlybetter,crucible,math-first}` · `temm1e-labs/{temm1e,temrust}` · `tsotchke/{eshkol,moonlab,quantum_geometric_tensor,quantum_rng,rosette-wire,libirrep,PINN,spin_based_neural_network,tensorcore}` · `0thernes/{autophage,cellular-engine-omega,cellulogonic-mecha,noetic-organism-omega,planaria,alife-singularity-omega-package}` · **MIT & ICLR 2026 Cache-to-Cache / Swarm Telepathy (`arXiv:2510.03215`, `arXiv:2610.15095`, `Augment-Cache / Replace-Activation`, `CacheGraft`)**.

---

## Live Verified Results (`v0.2.0` — Intel Core Ultra 9 275HX + NVIDIA RTX 5070 Ti `sm_12.0` · Win64)

Every metric below was recorded directly from `.\build.ps1` in [`results/verify_win64.txt`](results/verify_win64.txt) and [`results/lean4-audit.txt`](results/lean4-audit.txt):

| Verification Pillar | Live Measured Result (Zero Mocks) | Evidence Artifact |
|---|---|---|
| **Zig `0.17.1` `beast-metal` (`u1`/`b1.58`/`i2`/`NVFP4`)** | **`4.65×` SIMD speedup** (`10M` trits: `5232 µs → 1124 µs`, `~8.9B trits/s`), **0-bit diff**, **`5.46 ns/call`** Rust↔Zig FFI, `3/3` Zig mutants caught | `zig/beast-metal/src/root.zig` |
| **120-Gen Zig Chimera Insert-Network Breeder (`KC-4`)** | `24` hybrid organisms × `120` gens in **`82 ms`** (`10` blocks, all 6 ops `0x3f`, `‖W‖₂=0.0661 ≤ 0.99`): held-out MSE **`0.0277 → 0.0095`** vs random control **`0.0313`** (**`3.29×` better**) | `zig/beast-metal/src/root.zig`, `results/verify_win64.txt` |
| **Freestanding `x86_64` Multiboot2 Microkernel v2** | Zero-libc `ELF64` (**`13,656 bytes`**) with Multiboot2 `0xE85250D6` header + embedded Ring-0 **`b1.58` + integer `NVFP4`** dual neural reflex cores | `zig/freestanding_kernel/src/kernel.zig` |
| **RTX 5070 Ti Blackwell `sm_12.0` 3-Kernel CUDA PTX** | Direct `nvcuda.dll` JIT (**zero MSVC/nvcc**) on `46 SMs`, **`3/3` kernels verified**: `b1.58(4096×640) = 178 µs` (0-bit exact) · `nvfp4(4096×256) = 61 µs` · `c2c_aug(768×64) = 28 µs` | `cuda/beast_blackwell.ptx`, `crates/beast-cuda/src/lib.rs` |
| **Trained MIT `SwaT` C2C + 2×2 Law + `.bcg` `CacheGraft`** | **`5.64×`** vs text prefill; trained held-out MSE **`-100.0%`**, cos **`1.0000`**; **`Augment self-recall = 1.000` vs `Overwrite = -0.311`**; real `.bcg` repo bank (`3,120 B`) + tamper & cloud-spoof mutants caught | `crates/beast-c2c/src/{lib,trainer,graft_io}.rs` |
| **Planaria GF($2^8$) Cauchy MDS + Merkle Capsule** | **50% data annihilated** (all 4 original data shards destroyed) + Byzantine Trojan injected → **`1/1` Trojan quarantined** & **`100.000%` bit-exact heal** | `crates/beast-wire/src/hbin_capsule.rs` |
| **Multi-Colony Civilization OS + Live `AUTOPHAGE` RWX JIT** | `3` colonies × `6` castes = `18` agents, `QUBO E=-9.70`, `20` C2C handoffs, `18/18` Wilson graduated, `NonHuman -28.8%`; **live 10-epoch `AUTOPHAGE v1.3` RWX x86-64 JIT == Scalar (`bit_exact=true`, `sha=1f7d6d6f5ee09fbe..`)** | `crates/beast-colony/src/{civilization,autophage_ffi}.rs` |
| **Live Compiled Upstream `tsotchke/quantum_rng` C DLL** | Compiled `vendor/tsotchke__quantum_rng` via `zig cc` (`tsotchke_qrng_bridge.dll`) & executed live from Rust: **CHSH Bell violation `S = 2.7461 > 2.0`** (plus QGTL `g_tt=0.2500 / Berry=-0.4330`, QUBO `E=-7.20`) | `c_bridges/tsotchke_qrng_bridge.c`, `crates/beast-colony/src/tsotchke_ffi.rs` |
| **CRuby `3.4.9` + Prism AST + 10-Gen YARV Breeder (`T1a`)** | Live 10-generation YARV binary bytecode evolution (`fit 3.8078 → 4.4078`, `5` promotions, `5` rollbacks), **`2/2` injection mutants blocked by Prism**, **`1,640.2 ns/call`** CRuby→Zig Fiddle FFI | `ruby/beast_yarv.rb` |
| **Pure Lean 4 `v4.35.0` Formal Proofs** | **`24 / 24` theorems PROVEN** strictly within `{propext, Classical.choice, Quot.sound}` (**`0 sorry`, `0 admit`, `0` custom axioms, `0 bv_decide`, `0 warnings`**) incl. live receipt **`B24_live_v020_hardware_receipt`** | `lean/BeastProofs.lean`, `results/lean4-audit.txt` |
| **Total Verification & Anti-LARP Mutants** | **`12 / 12` Gates PASS · `17 / 17` Planted Mutants Caught (`3` Zig + `12` Rust/CLI + `2` CRuby/Prism) · `14 / 14` Rust Unit Tests PASS · `0` Compiler Warnings** | `results/verify_win64.txt` |

---

## Quickstart (`A03`)

```powershell
# One-command full 6-layer build, test, 3-kernel GPU PTX, Tsotchke C DLL, YARV breeder, and 24 Lean 4 proofs:
.\build.ps1

# Bare-metal CLI commands:
.\bin\beast.exe probe
.\bin\beast.exe verify-all --export-lean lean\LiveReceipt.lean
.\bin\beast.exe evolve-chimera
.\bin\beast.exe colony-sim
.\bin\beast.exe graft-repo
.\bin\beast.exe bench
.\bin\beast.exe api-query arxiv "cache-to-cache"
ruby ruby\beast_yarv.rb
```
