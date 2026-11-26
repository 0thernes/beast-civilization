# Clean-Clone Replication Protocol (`B01`, `B43` — `v0.5.0`)

## Pinned Toolchains (`B02`)
- **Zig:** `0.17.1` (`C:\tools\zig\zig.exe`, SHA-256 `b03f477a1943673e02aca0f570e8030dffd536ce1f45cbc4d3326ec31159562e`) — builds `beast_metal.dll`, both freestanding ELFs, forged A-Life `.exe` binaries, and all 6 upstream Tsotchke C11 DLLs via `zig cc`.
- **Rust:** `1.94.1` (`rustc` + `cargo`, 8-crate workspace, **zero external crates.io dependencies** — 100% offline reproducible!).
- **CRuby:** `3.4.9 +PRISM` (`C:\tools\ruby34\bin\ruby.exe`, stdlib + `prism` + `fiddle` only — **zero external gems!**).
- **Lean 4:** `v4.35.0` (`~/.elan/toolchains/leanprover--lean4---v4.35.0/bin/lean.exe`, **zero Mathlib!**).
- **QEMU:** `11.1.0` (`~/scoop/apps/qemu/current/qemu-system-x86_64.exe`, installed via `scoop install qemu`) for live Ring-0 Multiboot1 AI microkernel boot verification.
- **Optional GPU:** NVIDIA Driver exposing `nvcuda.dll` / `libcuda.so.1` (verified on `RTX 5070 Ti Laptop GPU`, Compute Capability `12.0`).
- **Optional Local 0thernes A-Life DLLs:** `autophage/bin/autophage.dll`, `cellular-engine-omega/bin/omega_cell.dll`, `cellulogonic-mecha/bin/cellulogonic.dll`.

## One-Command Reproduction (Windows x64)
```powershell
Set-Location -LiteralPath 'Z:\[Vibe Coded (AI)]\ANTIGRAVITY\beast-civilization'
.\build.ps1
```

## Individual Verification Commands
```powershell
# 1. Zig unit tests + 3 planted mutants
C:\tools\zig\zig.exe test zig\beast-metal\src\root.zig

# 2. Rust 8-crate unit & mutant test suite (18/18 tests)
cargo test --workspace

# 3. Live 11-DLL probe, 25-gate/25-mutant audit, QEMU Ring-0 boot, 4-Layer LLM C2C, Trinity Forge & benchmarks
.\bin\beast.exe probe
.\bin\beast.exe verify-all --export-lean lean\LiveReceipt.lean
.\bin\beast.exe llm-c2c
.\bin\beast.exe repo-map
.\bin\beast.exe boot-kernel
.\bin\beast.exe forge-kernel matrix_alpha
.\bin\beast.exe forge-model chimera_alpha
.\bin\beast.exe forge-alife matrix_life
.\bin\beast.exe agent-run "full matrix forge kernel model and alife"
.\bin\beast.exe bench

# 4. CRuby 3.4 + YARV 10-Gen Breeder + Prism AST guard + Zig Fiddle C-ABI
C:\tools\ruby34\bin\ruby.exe ruby\beast_yarv.rb

# 5. Pure Lean 4 kernel proofs (50/50 theorems) & strict axiom audit
Push-Location -LiteralPath 'lean'
& "$env:USERPROFILE\.elan\toolchains\leanprover--lean4---v4.35.0\bin\lean.exe" -R . -o LiveReceipt.olean LiveReceipt.lean
& "$env:USERPROFILE\.elan\toolchains\leanprover--lean4---v4.35.0\bin\lean.exe" -R . BeastProofs.lean
Pop-Location
```
