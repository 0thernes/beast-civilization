<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Honest Benchmarks & Performance Budgets (`A37`, `B26`, `B27`)

All numbers below are measured live on real hardware with **zero mocks** and **zero synthetic timers**. Timing measurements vary with CPU/GPU boost state, cold PTX JIT compilation, and OS scheduler load; exact bit-level outputs (`SHA-256`, `int64` dot products, `residual == 0`, Lean kernel reductions) are 100% deterministic across runs.

## Primary Silicon Profile
- **CPU:** Intel Core Ultra 9 275HX (`24C/24T`, x86-64 AVX2/VNNI/GFNI)
- **GPU:** NVIDIA GeForce RTX 5070 Ti Laptop GPU (`12,227 MiB`, `sm_12.0` Blackwell, `46 SMs`, direct `nvcuda.dll`)
- **OS & Toolchains:** Windows 11 x86-64 · Zig `0.17.1` · Rust `1.94.1` · CRuby `3.4.9 +PRISM` · Lean `4.35.0` · QEMU `11.1.0`

## Performance Budgets & Release History (`B27`)

| Metric | Budget / Kill Criterion | `v0.7.0` Baseline | `v1.0.0` Verbatim (`results/verify_win64.txt`) | Status |
|---|---|---|---|---|
| Rust ↔ Zig Philox4x32-10 FFI | `< 50 ns/call` (`KC-2`) | `6.68 ns/call` | **`7.50 ns/call`** | **PASS** |
| CRuby → Zig Fiddle FFI | `< 10,000 ns/call` (`KC-2`) | `1,713.5 ns/call` | **`1,787.2 ns/call`** | **PASS** |
| Zig `b1.58` SIMD vs Scalar (10M trits) | `>= 2.0×`, `0`-bit exact | `3.42×` (`5,551 → 1,624 µs`) | **`3.31×`** (`24,107 → 7,294 µs`, `0`-bit exact) | **PASS** |
| Zig `b1.58` SIMD vs Scalar (`N=24` `H3`, 2M trits) | `>= 1.5×`, `p_Holm <= 0.05` | *(added in v1.0.0)* | **`5.58×`** (`2,663.2 → 477.5 µs`, `24/24`, `p_Holm=0.00040`) | **PASS** |
| MIT `SwaT` C2C vs Text Prefill | `>= 2.0×`, `self_kv` preserved | `4.34×` (`2,159 → 498 µs`) | **`29.62×`** (`19,253 → 650 µs`, `64/80` self KV) | **PASS** |
| Chimera 120-Gen Breeder (`KC-4`) | `< 2,000 ms`, beats random ctrl | `100 ms` (`0.0277 → 0.0095` vs `0.0313`) | **`170 ms`** (`0.0277 → 0.0095` vs `0.0313`, `‖W‖₂=0.0661`) | **PASS** |
| Chimera 60-Gen Breeder (`N=24` `H2`) | `p_Holm <= 0.05`, `>= 20/24` wins | *(added in v1.0.0)* | **`3.29×` lower MSE** (`0.0323 → 0.0098`, `24/24`, `p_Holm=0.00040`) | **PASS** |
| Blackwell `sm_12.0` 5-Kernel PTX (warm `probe`) | `5/5` verified vs CPU oracle | `411/72/25/19/25 µs` | **`122/33/22/19/16 µs`** (cold JIT `bench` `b1.58` outlier `5,903 µs` disclosed) | **PASS** |
| CUDA Pinned Host↔VRAM Async Paging | `>= 10 GB/s`, `0`-bit exact | `20.21 GB/s` (`16 MiB`) | **`16.69–17.03 GB/s`** (`16 MiB`, `0`-bit exact) | **PASS** |
| 1,000-Case Binary/Protocol Fuzz Suite | `< 1,000 ms`, `0` panics | *(added in v1.0.0)* | **`40 ms`** (`8/8` hardening/fuzz tests, `0` panics) | **PASS** |

## Pre-Registered Multi-Seed Statistical Evaluation (`B28`–`B32`)
See [`docs/preregistration.md`](preregistration.md) and [`results/paired_stats_n24.txt`](../results/paired_stats_n24.txt) for the locked 24-seed (`5000..5023`) paired permutation, Cohen's $d_z$, 95% bootstrap CI, and step-down Holm-Bonferroni FWER evaluation across `H1`–`H4` (**`4 / 4` significant at $\alpha_{\mathrm{FWER}} = 0.05$**).
