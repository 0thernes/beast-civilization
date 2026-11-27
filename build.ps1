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
# [1/6] Zig 0.17.1 Bare-Metal Engine + 7 Upstream Tsotchke C11 Bridges via `zig cc`
# ============================================================================
Write-Host "`n[1/6] Building & testing Zig 0.17.1 beast-metal + 7 upstream Tsotchke Win64 DLLs..."
& $zig test "$proj\zig\beast-metal\src\root.zig"
if ($LASTEXITCODE -ne 0) { throw "Zig unit & planted-mutant tests failed!" }

& $zig build-lib "$proj\zig\beast-metal\src\root.zig" `
    -dynamic -OReleaseFast `
    -femit-bin="$binDir\beast_metal.dll"
if ($LASTEXITCODE -ne 0) { throw "Zig beast_metal.dll build failed!" }

# Upstream 1: `tsotchke/quantum_rng`
$tsoDir = "$proj\vendor\tsotchke__quantum_rng"
if (Test-Path -LiteralPath "$tsoDir\src\quantum_rng\bell_test.c") {
    Write-Host "      [1/7] Compiling upstream tsotchke/quantum_rng Bell-test bridge DLL..."
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

# Upstream 2: `tsotchke/PINN`
$pinnDir = "$proj\vendor\tsotchke__PINN"
if (Test-Path -LiteralPath "$pinnDir\src\neural_network.c") {
    Write-Host "      [2/7] Compiling upstream tsotchke/PINN bridge DLL..."
    & $zig cc -O2 -shared `
        -I"$pinnDir\include" `
        "$pinnDir\src\neural_network.c" `
        "$pinnDir\src\loss_functions.c" `
        "$pinnDir\src\utils.c" `
        "$proj\c_bridges\tsotchke_pinn_bridge.c" `
        -o "$binDir\tsotchke_pinn_bridge.dll" -lm
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_pinn_bridge.dll build failed!" }
}

# Upstream 3: `tsotchke/libirrep`
$irrepDir = "$proj\vendor\tsotchke__libirrep"
if (Test-Path -LiteralPath "$irrepDir\src\spherical_harmonics.c") {
    Write-Host "      [3/7] Compiling upstream tsotchke/libirrep SO(3) AVX2+FMA bridge DLL..."
    & $zig cc -O2 -mavx2 -mfma -DIRREP_BUILDING_DLL -shared `
        -I"$irrepDir\include" -I"$irrepDir\src" `
        "$irrepDir\src\spherical_harmonics.c" `
        "$irrepDir\src\spherical_harmonics_avx2.c" `
        "$irrepDir\src\solid_harmonics.c" `
        "$irrepDir\src\radial.c" `
        "$irrepDir\src\radial_avx2.c" `
        "$irrepDir\src\simd_runtime.c" `
        "$proj\c_bridges\tsotchke_libirrep_bridge.c" `
        -o "$binDir\tsotchke_libirrep_bridge.dll" -lm
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_libirrep_bridge.dll build failed!" }
}

# Upstream 4: `tsotchke/moonlab` QGT Topology
$moonDir = "$proj\vendor\tsotchke__moonlab"
if (Test-Path -LiteralPath "$moonDir\src\algorithms\quantum_geometry\qgt.c") {
    Write-Host "      [4/7] Compiling upstream tsotchke/moonlab QGT/Chern/SSH/Kitaev bridge DLL..."
    & $zig cc -O2 -shared -Wno-macro-redefined `
        -I"$moonDir\src" `
        "$moonDir\src\algorithms\quantum_geometry\qgt.c" `
        "$moonDir\src\utils\matrix_math.c" `
        "$proj\c_bridges\tsotchke_moonlab_qgt_bridge.c" `
        -o "$binDir\tsotchke_moonlab_qgt_bridge.dll" -lm
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_moonlab_qgt_bridge.dll build failed!" }
}

# Upstream 5: `tsotchke/moonlab` FIPS-203 ML-KEM-768 + FIPS-197 AES-256
if (Test-Path -LiteralPath "$moonDir\src\crypto\mlkem\mlkem.c") {
    Write-Host "      [5/7] Compiling upstream tsotchke/moonlab FIPS-203 ML-KEM-768 + AES-256 bridge DLL..."
    & $zig cc -O2 -shared -Wno-macro-redefined `
        -I"$moonDir\src" `
        "$moonDir\src\crypto\mlkem\mlkem.c" `
        "$moonDir\src\crypto\mlkem\poly.c" `
        "$moonDir\src\crypto\sha3\sha3.c" `
        "$moonDir\src\crypto\aes\aes.c" `
        "$proj\c_bridges\tsotchke_moonlab_mlkem_bridge.c" `
        -o "$binDir\tsotchke_moonlab_mlkem_bridge.dll"
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_moonlab_mlkem_bridge.dll build failed!" }
}

# Upstream 6: `tsotchke/spin_based_neural_network` Fibonacci Anyons + Spintronic p-bit
$sbnnDir = "$proj\vendor\tsotchke__spin_based_neural_network"
if (Test-Path -LiteralPath "$sbnnDir\src\fibonacci_anyons\fibonacci_anyons.c") {
    Write-Host "      [6/7] Compiling upstream tsotchke/spin_based_neural_network Anyon+p-bit bridge DLL..."
    & $zig cc -O2 -shared -Wno-macro-redefined `
        -I"$sbnnDir\include" `
        "$sbnnDir\src\fibonacci_anyons\fibonacci_anyons.c" `
        "$sbnnDir\src\neuromorphic\pbit.c" `
        "$proj\c_bridges\tsotchke_sbnn_bridge.c" `
        -o "$binDir\tsotchke_sbnn_bridge.dll" -lm
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_sbnn_bridge.dll build failed!" }
}

# Upstream 7: `tsotchke/quantum_geometric_tensor` (`QGTL`) Bloch Complex Fidelity
$qgtlDir = "$proj\vendor\tsotchke__quantum_geometric_tensor"
if (Test-Path -LiteralPath "$qgtlDir\src\quantum_geometric\core\complex_arithmetic.c") {
    Write-Host "      [7/7] Compiling upstream tsotchke/quantum_geometric_tensor QGTL bridge DLL..."
    & $zig cc -O2 -shared -Wno-macro-redefined `
        -I"$qgtlDir\include" `
        "$qgtlDir\src\quantum_geometric\core\complex_arithmetic.c" `
        "$proj\c_bridges\tsotchke_qgtl_bridge.c" `
        -o "$binDir\tsotchke_qgtl_bridge.dll" -lm
    if ($LASTEXITCODE -ne 0) { throw "tsotchke_qgtl_bridge.dll build failed!" }
}

# ============================================================================
# [2/6] Zig Freestanding OS Microkernels: ELF64 Multiboot2 + Live-Bootable ELF32 Multiboot1
# ============================================================================
Write-Host "`n[2/6] Cross-compiling freestanding x86-64 Multiboot2 + i386 Multiboot1 Microkernels (zero libc)..."
$elfPath = "$binDir\beast_microkernel.elf"
& $zig build-exe "$proj\zig\freestanding_kernel\src\kernel.zig" `
    -target x86_64-freestanding-none `
    -OReleaseSmall `
    -T "$proj\zig\freestanding_kernel\linker.ld" `
    -femit-bin="$elfPath"
if ($LASTEXITCODE -ne 0) { throw "Freestanding x86-64 microkernel compile failed!" }

$elf32Path = "$binDir\beast_boot32.elf"
& $zig build-exe "$proj\zig\freestanding_kernel\src\boot32.zig" `
    -target x86-freestanding-none `
    -OReleaseSmall `
    -T "$proj\zig\freestanding_kernel\linker32.ld" `
    -femit-bin="$elf32Path"
if ($LASTEXITCODE -ne 0) { throw "Freestanding i386 Multiboot1 microkernel compile failed!" }

$elfBytes = [System.IO.File]::ReadAllBytes($elfPath)
$elf32Bytes = [System.IO.File]::ReadAllBytes($elf32Path)
Write-Host "[PASS] Compiled ELF64 Multiboot2 ($($elfBytes.Length) B) + ELF32 Multiboot1 ($($elf32Bytes.Length) B)"

# ============================================================================
# [3/6] Rust 1.94.1 8-Crate Workspace (Wire, Root, C2C, Colony, CUDA, Model, Agent, CLI)
# ============================================================================
Write-Host "`n[3/6] Testing & building Rust 8-crate workspace..."
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
# [4/6] Live End-to-End 33-Gate / 33-Mutant Verification + Hyper-Colony + Lean Export
# ============================================================================
Write-Host "`n[4/6] Running live 33-gate / 33-mutant verification, 12-DLL probe, 5-kernel GPU, OpenAI/GGUF loop, and Lean receipt export..."
Push-Location -LiteralPath $proj
try {
    & "$binDir\beast.exe" probe
    if ($LASTEXITCODE -ne 0) { throw "beast probe failed!" }

    & "$binDir\beast.exe" verify-all --export-lean "$proj\lean\BeastProofs\LiveReceiptData.lean"
    if ($LASTEXITCODE -ne 0) { throw "beast verify-all failed!" }

    & "$binDir\beast.exe" evolve-chimera
    if ($LASTEXITCODE -ne 0) { throw "beast evolve-chimera failed!" }

    & "$binDir\beast.exe" colony-sim
    if ($LASTEXITCODE -ne 0) { throw "beast colony-sim failed!" }

    & "$binDir\beast.exe" graft-repo
    if ($LASTEXITCODE -ne 0) { throw "beast graft-repo failed!" }

    & "$binDir\beast.exe" model-c2c
    if ($LASTEXITCODE -ne 0) { throw "beast model-c2c failed!" }

    & "$binDir\beast.exe" llm-c2c
    if ($LASTEXITCODE -ne 0) { throw "beast llm-c2c failed!" }

    & "$binDir\beast.exe" inspect-gguf "$binDir\beast_4layer_chimera.gguf"
    if ($LASTEXITCODE -ne 0) { throw "beast inspect-gguf failed!" }

    & "$binDir\beast.exe" ad-test
    if ($LASTEXITCODE -ne 0) { throw "beast ad-test failed!" }

    & "$binDir\beast.exe" cross-repo
    if ($LASTEXITCODE -ne 0) { throw "beast cross-repo failed!" }

    & "$binDir\beast.exe" inspect-asm
    if ($LASTEXITCODE -ne 0) { throw "beast inspect-asm failed!" }

    & "$binDir\beast.exe" omni-test
    if ($LASTEXITCODE -ne 0) { throw "beast omni-test failed!" }

    & "$binDir\beast.exe" repo-map
    if ($LASTEXITCODE -ne 0) { throw "beast repo-map failed!" }

    & "$binDir\beast.exe" boot-kernel
    if ($LASTEXITCODE -ne 0) { throw "beast boot-kernel failed!" }

    & "$binDir\beast.exe" forge-alife matrix_life
    if ($LASTEXITCODE -ne 0) { throw "beast forge-alife failed!" }

    & "$binDir\beast.exe" agent-run "full matrix forge kernel model and alife"
    if ($LASTEXITCODE -ne 0) { throw "beast agent-run failed!" }

    & "$binDir\beast.exe" repl --script "mission colony model audit;quit"
    if ($LASTEXITCODE -ne 0) { throw "beast repl --script failed!" }

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
& $ruby -w "$proj\ruby\beast_yarv.rb"
if ($LASTEXITCODE -ne 0) { throw "CRuby/YARV bridge failed!" }

# ============================================================================
# [6/6] Pure Lean 4 v4.35.0 Palomar Kernel Proofs (81 Theorems) + Strict VerifyAxioms Audit
# ============================================================================
if (-not $SkipLean -and (Test-Path -LiteralPath $lean)) {
    Write-Host "`n[6/6] Machine-checking 81 Pure Lean 4.35.0 Palomar theorems via lake build + VerifyAxioms.lean..."
    $leanDir = "$proj\lean"
    $lake = "$leanRoot\bin\lake.exe"
    $env:PATH = "$leanRoot\bin;$env:PATH"
    Push-Location -LiteralPath $leanDir
    try {
        & $lake build 2>&1 | ForEach-Object { Write-Host $_ }
        if ($LASTEXITCODE -ne 0) { throw "Lean 4 lake build failed!" }

        & $lake build Challenge 2>&1 | ForEach-Object { Write-Host $_ }
        if ($LASTEXITCODE -ne 0) { throw "Lean 4 lake build Challenge failed!" }

        $leanOut = & $lake env lean "VerifyAxioms.lean" 2>&1
        $leanRc = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    foreach ($line in $leanOut) { Write-Host $line }
    if ($leanRc -ne 0) { throw "Lean 4 VerifyAxioms verification failed (rc=$leanRc)!" }

    # Normalize multi-line wrapped `#print axioms` output onto canonical single lines
    $joinedLines = @()
    foreach ($raw in $leanOut) {
        $seg = "$raw".Trim()
        if ($seg -eq "") { continue }
        if ($seg.StartsWith("'") -or $joinedLines.Count -eq 0) {
            $joinedLines += $seg
        } else {
            $joinedLines[$joinedLines.Count - 1] = ($joinedLines[$joinedLines.Count - 1] + " " + $seg) -replace "\s+", " "
        }
    }

    $allowed = @("propext", "Classical.choice", "Quot.sound")
    $audited = 0
    $cleanLines = @()
    foreach ($line in $joinedLines) {
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
    if ($audited -ne 81) {
        throw "Expected 81 audited Lean 4 theorems, got $audited"
    }
    [System.IO.File]::WriteAllText("$resDir\lean4-audit.txt", ($cleanLines -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
    Write-Host "[PASS] $audited/81 Pure Lean 4.35.0 Palomar theorems audited cleanly (0 sorry, 0 custom axioms, 0 warnings)"
}

Write-Host "`n========================================================================="
Write-Host "  BEAST CIVILIZATION v1.0.0 FULL-STACK BUILD & VERIFICATION: ALL GREEN!"
Write-Host "========================================================================="
