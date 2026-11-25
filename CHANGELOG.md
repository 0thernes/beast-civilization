# Changelog (`A07`)

## [0.1.0] - 2026-11-25
### Added
- Initial `beast-civilization` polyglot monorepo (`Zig 0.17.1`, `Rust 1.94.1`, `CRuby 3.4.9/YARV`, `C11/AUTOPHAGE`, `CUDA sm_120 PTX`, `Pure Lean 4 v4.35.0`).
- Build Standard v1.2.0 core (`AGENTS.md`, `CLAIMS.md`, `LIMITATIONS.md`, `REPLICATION.md`, `repo.yaml`, `docs/plan/00-*`, `docs/matrices/300_agent_matrix.yaml`, `THIRD_PARTY_NOTICES.md`).
- `zig/beast-metal`: bit-exact arbitrary mixed-precision kernels (`u1`, `b1.58` 5-trit/B, `i2`, `u3`, `nvfp4_e2m1`, `fp8_e4m3`), Chimera Insert-Network engine (`OP_INSERT_BLOCK`, `OP_PREC_MORPH`, `OP_CROSS_XENOGRAFT`, `OP_CACHE_GRAFT_PORT`, `OP_SPECTRAL_CLAMP`), `Philox4x32-10` PRNG.
- `zig/freestanding_kernel`: freestanding `x86_64-freestanding` Multiboot2 bare-metal microkernel ELF with embedded `b1.58` ternary reflex core.
- `crates/{beast-wire,beast-root,beast-c2c,beast-colony,beast-cuda,beast-cli}`: Ring-0 immutable Root guardian (`INIT-Agent` Win64+Linux), MIT C2C / `Swarm Telepathy` / `Augment-Cache & Replace-Activation` / `CacheGraft` engine, `TEMM1E` + `NonHuman` + `Engram` + `Moonlab`-QUBO + `QGTL` Fubini-Study/Berry Colony OS, direct `nvcuda.dll` Blackwell `sm_120` PTX runner, and live `AUTOPHAGE` DLL FFI.
- `ruby/beast_yarv.rb`: CRuby 3.4 + Prism + YARV live bytecode inspection, serialization, mutation, and Zig C-ABI Fiddle execution.
- `lean/BeastProofs.lean`: Pure Lean 4 kernel proofs (`0 sorry`, `{propext, Classical.choice, Quot.sound}`) including live runtime-to-Lean telemetry receipt verification.
