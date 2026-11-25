# Clean-Clone Replication Protocol (`B01`, `B43`)

## Pinned Toolchains (`B02`)
- **Zig:** `0.17.1` (`C:\tools\zig\zig.exe`, SHA-256 `b03f477a1943673e02aca0f570e8030dffd536ce1f45cbc4d3326ec31159562e`)
- **Rust:** `1.94.1` (`rustc` + `cargo`, zero external crates.io dependencies — 100% offline reproducible!)
- **CRuby:** `3.4.9 +PRISM` (`C:\tools\ruby34\bin\ruby.exe`, stdlib + `prism` + `fiddle` only — zero external gems!)
- **Lean 4:** `v4.35.0` (`~/.elan/toolchains/leanprover--lean4---v4.35.0/bin/lean.exe`, zero Mathlib!)
- **Optional GPU:** NVIDIA Driver exposing `nvcuda.dll` / `libcuda.so.1` (verified on `RTX 5070 Ti Laptop GPU`, Compute Capability `12.0`).
- **Optional Local A-Life Engine:** `Z:\[Vibe Coded (AI)]\ANTIGRAVITY\autophage\bin\autophage.dll`.

## One-Command Reproduction (Windows x64)
```powershell
Set-Location -LiteralPath 'Z:\[Vibe Coded (AI)]\ANTIGRAVITY\beast-civilization'
.\build.ps1
```

## Individual Verification Commands
```powershell
# 1. Zig unit tests + 3 planted mutants + DLL + freestanding x86-64 ELF
C:\tools\zig\zig.exe test zig\beast-metal\src\root.zig

# 2. Rust 6-crate unit & mutant test suite
cargo test --workspace

# 3. Live hardware probe, 8-gate/8-mutant verification, Chimera evolution & silicon benchmarks
.\bin\beast.exe probe
.\bin\beast.exe verify-all --export-lean lean\LiveReceipt.lean
.\bin\beast.exe evolve-chimera
.\bin\beast.exe bench

# 4. CRuby 3.4 + YARV + Prism AST guard + YJIT + Zig Fiddle C-ABI
ruby --yjit ruby\beast_yarv.rb

# 5. Pure Lean 4 kernel proofs & strict axiom audit
$env:LEAN_PATH = "$PWD\lean;$env:USERPROFILE\.elan\toolchains\leanprover--lean4---v4.35.0\lib\lean"
& "$env:USERPROFILE\.elan\toolchains\leanprover--lean4---v4.35.0\bin\lean.exe" -o lean\LiveReceipt.olean lean\LiveReceipt.lean
& "$env:USERPROFILE\.elan\toolchains\leanprover--lean4---v4.35.0\bin\lean.exe" lean\BeastProofs.lean
```
