param(
    [switch]$SkipBench,
    [switch]$SkipLean
)

$ErrorActionPreference = "Stop"
$proj  = Split-Path -Parent $MyInvocation.MyCommand.Path
$zig   = "C:\tools\zig\zig.exe"
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$ruby  = "C:\tools\ruby34\bin\ruby.exe"
$leanRoot = "$env:USERPROFILE\.elan\toolchains\leanprover--lean4---v4.35.0"
$lean  = "$leanRoot\bin\lean.exe"

foreach ($tool in @($zig, $cargo, $ruby)) {
    if (-not (Test-Path -LiteralPath $tool)) {
        throw "Required toolchain missing: $tool"
    }
}

$binDir = Join-Path $proj "bin"
$resDir = Join-Path $proj "results"
[System.IO.Directory]::CreateDirectory($binDir) | Out-Null
[System.IO.Directory]::CreateDirectory($resDir) | Out-Null

# ============================================================================
# [1/6] Zig 0.17.1 Bare-Metal Engine (`beast_metal.dll`) + Native Zig Tests
# ============================================================================
Write-Host "`n[1/6] Building & testing Zig 0.17.1 beast-metal (b1.58 / u1 / i2 / NVFP4 / Chimera)..."
& $zig test "$proj\zig\beast-metal\src\root.zig"
if ($LASTEXITCODE -ne 0) { throw "Zig unit & planted-mutant tests failed!" }

& $zig build-lib "$proj\zig\beast-metal\src\root.zig" `
    -dynamic -OReleaseFast `
    -femit-bin="$binDir\beast_metal.dll"
if ($LASTEXITCODE -ne 0) { throw "Zig beast_metal.dll build failed!" }

# Optional native Win64 compilation of vendored upstream `tsotchke/quantum_rng` via `zig cc`
$tsoDir = "$proj\vendor\tsotchke__quantum_rng"
if (Test-Path -LiteralPath "$tsoDir\src\quantum_rng\bell_test.c") {
    Write-Host "      Compiling upstream tsotchke/quantum_rng Bell-test bridge DLL via zig cc..."
    & $zig cc -O2 -msse3 -Wno-macro-redefined -shared `
        -I"$tsoDir" -I"$tsoDir\src" -I"$tsoDir\src\quantum_rng" `
        "$tsoDir\src\quantum_rng\quantum_state.c" `
        "$tsoDir\src\quantum_rng\quantum_gates.c" `
        "$tsoDir\src\quantum_rng\matrix_math.c" `
        "$tsoDir\src\quantum_rng\simd_ops.c" `
        "$tsoDir\src\quantum_rng\bell_test.c" `
        "$proj\c_bridges\tsotchke_qrng_bridge.c" `
        -o "$binDir\tsotchke_qrng_bridge.dll" -lm
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_qrng_bridge.dll build failed!" }
}

# ============================================================================
# [2/6] Zig Freestanding x86-64 Multiboot2 Bare-Metal OS Microkernel ELF
# ============================================================================
Write-Host "`n[2/6] Cross-compiling freestanding x86-64 Multiboot2 Microkernel ELF (zero libc)..."
$elfPath = "$binDir\beast_microkernel.elf"
& $zig build-exe "$proj\zig\freestanding_kernel\src\kernel.zig" `
    -target x86_64-freestanding-none `
    -OReleaseSmall `
    -T "$proj\zig\freestanding_kernel\linker.ld" `
    -femit-bin="$elfPath"
if ($LASTEXITCODE -ne 0) { throw "Freestanding x86-64 microkernel compile failed!" }

# Binary forensic verification: ELF64 magic (7F 45 4C 46) + Multiboot2 header magic (D6 50 52 E8)
$elfBytes = [System.IO.File]::ReadAllBytes($elfPath)
if ($elfBytes.Length -lt 64 -or $elfBytes[0] -ne 0x7F -or $elfBytes[1] -ne 0x45 -or $elfBytes[2] -ne 0x4C -or $elfBytes[3] -ne 0x46) {
    throw "Invalid ELF header in $elfPath"
}
$mbFound = $false
for ($i = 0; $i -le ($elfBytes.Length - 4); $i++) {
    if ($elfBytes[$i] -eq 0xD6 -and $elfBytes[$i+1] -eq 0x50 -and $elfBytes[$i+2] -eq 0x52 -and $elfBytes[$i+3] -eq 0xE8) {
        $mbFound = $true; break
    }
}
if (-not $mbFound) { throw "Multiboot2 magic 0xE85250D6 missing from $elfPath!" }
Write-Host "[PASS] Verified ELF64 + Multiboot2 header inside $elfPath ($($elfBytes.Length) bytes)"

# ============================================================================
# [3/6] Rust 1.94.1 6-Crate Workspace (Root, C2C, Colony, CUDA PTX, Wire, CLI)
# ============================================================================
Write-Host "`n[3/6] Testing & building Rust 6-crate workspace..."
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
Push-Location -LiteralPath $proj
try {
    & $cargo test --workspace
    if ($LASTEXITCODE -ne 0) { throw "Cargo workspace tests failed!" }

    & $cargo build --release --bin beast
    if ($LASTEXITCODE -ne 0) { throw "Cargo release build failed!" }
} finally {
    Pop-Location
}
Copy-Item -LiteralPath "$proj\target\release\beast.exe" -Destination "$binDir\beast.exe" -Force

# ============================================================================
# [4/6] Live End-to-End Verification Suite + RTX 5070 Ti CUDA PTX + Lean Export
# ============================================================================
Write-Host "`n[4/6] Running live end-to-end verification, 8/8 planted mutants, and Lean receipt export..."
Push-Location -LiteralPath $proj
try {
    & "$binDir\beast.exe" probe
    if ($LASTEXITCODE -ne 0) { throw "beast probe failed!" }

    & "$binDir\beast.exe" verify-all --export-lean "$proj\lean\LiveReceipt.lean"
    if ($LASTEXITCODE -ne 0) { throw "beast verify-all failed!" }

    & "$binDir\beast.exe" evolve-chimera
    if ($LASTEXITCODE -ne 0) { throw "beast evolve-chimera failed!" }

    & "$binDir\beast.exe" colony-sim
    if ($LASTEXITCODE -ne 0) { throw "beast colony-sim failed!" }

    & "$binDir\beast.exe" graft-repo
    if ($LASTEXITCODE -ne 0) { throw "beast graft-repo failed!" }

    if (-not $SkipBench) {
        & "$binDir\beast.exe" bench
        if ($LASTEXITCODE -ne 0) { throw "beast bench failed!" }
    }
} finally {
    Pop-Location
}

# ============================================================================
# [5/6] CRuby 3.4.9 + Prism + YARV Live Bytecode Mutation + Zig Fiddle Bridge
# ============================================================================
Write-Host "`n[5/6] Running CRuby 3.4 + YARV + Prism AST Guard + Zig Fiddle C-ABI..."
& $ruby "$proj\ruby\beast_yarv.rb"
if ($LASTEXITCODE -ne 0) { throw "CRuby/YARV bridge failed!" }

# ============================================================================
# [6/6] Pure Lean 4 v4.35.0 Kernel Proofs + Strict Axiom Audit
# ============================================================================
if (-not $SkipLean -and (Test-Path -LiteralPath $lean)) {
    Write-Host "`n[6/6] Machine-checking 24 Pure Lean 4 theorems (including live hardware receipt B24)..."
    $leanDir = "$proj\lean"
    $env:LEAN_PATH = "$leanDir;$leanRoot\lib\lean"
    Push-Location -LiteralPath $leanDir
    try {
        & $lean -R . -o "LiveReceipt.olean" "LiveReceipt.lean" 2>&1
        if ($LASTEXITCODE -ne 0) { throw "Lean 4 LiveReceipt compile failed!" }

        $leanOut = & $lean -R . "BeastProofs.lean" 2>&1
        $leanRc = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    foreach ($line in $leanOut) { Write-Host $line }
    if ($leanRc -ne 0) { throw "Lean 4 BeastProofs verification failed (rc=$leanRc)!" }

    $allowed = @("propext", "Classical.choice", "Quot.sound")
    $audited = 0
    $cleanLines = @()
    foreach ($raw in $leanOut) {
        $line = "$raw".Trim()
        if ($line -eq "") { continue }
        if ($line -match "(?i)\b(warning|error)\b|sorryAx|declaration uses 'sorry'|bv_decide|native_decide|ofReduceBool|trustCompiler") {
            throw "Lean 4 kernel integrity violation: $line"
        }
        if ($line -match "^'([^']+)' does not depend on any axioms$") {
            $audited++
            $cleanLines += $line
            continue
        }
        if ($line -match "^'([^']+)' depends on axioms: \[([^\]]*)\]$") {
            $thm = $Matches[1]
            $axList = $Matches[2].Split(",") | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }
            foreach ($ax in $axList) {
                if ($allowed -notcontains $ax) {
                    throw "Forbidden axiom '$ax' on '$thm'!"
                }
            }
            $audited++
            $cleanLines += $line
            continue
        }
        throw "Unexpected Lean 4 output line: $line"
    }
    if ($audited -lt 24) {
        throw "Expected >= 24 audited Lean 4 theorems, got $audited"
    }
    [System.IO.File]::WriteAllText("$resDir\lean4-audit.txt", ($cleanLines -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
    Write-Host "[PASS] $audited/24 Pure Lean 4 theorems audited cleanly (0 sorry, 0 custom axioms, 0 warnings)"
}

Write-Host "`n========================================================================="
Write-Host "  BEAST CIVILIZATION v0.2.0 FULL-STACK BUILD & VERIFICATION: ALL GREEN!"
Write-Host "========================================================================="
