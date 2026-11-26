//! Autonomous Matrix Forge (`crates/beast-agent/src/forge.rs`):
//!   1. `boot_kernel_in_qemu`: Live-boots a 32-bit Multiboot1 freestanding neural kernel in QEMU Ring-0,
//!      captures COM1 serial (`0x3F8`), parses exact `B158_REFLEX` and `NVFP4_PRIO`, and proves bit-exact
//!      agreement against the Host CPU Oracle!
//!   2. `forge_neural_kernel`: Autonomously synthesizes a brand-new Zig freestanding Multiboot1 AI microkernel
//!      with custom evolved `b1.58` + `NVFP4` weights, compiles it via `zig`, audits ELF32/Multiboot1,
//!      live-boots it in QEMU (if installed), wraps it in a Planaria GF(2^8)+Merkle self-healing capsule,
//!      annihilates 50% of its shards + injects a Trojan, and proves 0-bit exact binary recovery!
//!   3. `forge_chimera_model`: Autonomously trains a cross-arch SwaT C2C Chimera, exports GGUF v3 + `.bcg`
//!      CacheGraft bank seeded by the live Merkle Repo-Map, and verifies Planaria self-healing reload!

use crate::repo_map::build_merkle_repo_map;
use beast_c2c::graft_io::{build_repo_cachegraft_bank, load_bcg, save_bcg};
use beast_c2c::trainer::train_and_verify_swat_mapper;
use beast_model::omni_engine::run_cross_arch_c2c_gguf_planaria;
use beast_wire::elf_audit::audit_elf32_file;
use beast_wire::hbin_capsule::PlanariaCapsule;
use beast_wire::philox::philox_u64;
use beast_wire::sha256::{hex_encode, sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_B158_WEIGHTS: [u8; 4] = [161, 82, 200, 41];
pub const DEFAULT_NVFP4_WEIGHTS: [u8; 8] = [0x25, 0xA3, 0x41, 0xB6, 0x17, 0x92, 0x34, 0xC5];
pub const DEFAULT_SENSORS: [i32; 20] = [
    3, -1, 4, 1, 5, -9, 2, 6, -5, 3, 5, -8, 9, 7, -9, 3, 2, 3, -8, 4,
];
pub const DEFAULT_FEATS: [i32; 16] = [5, 2, -3, 7, 1, -4, 6, 3, 2, -1, 8, -2, 4, 5, -6, 3];
const NVFP4_HALF_UNITS: [i32; 16] = [
    0, 1, 2, 3, 4, 6, 8, 12, 0, -1, -2, -3, -4, -6, -8, -12,
];

/// Host CPU oracle matching freestanding Zig `kernel_b158_reflex_infer` & `kernel_nvfp4_priority_infer`.
pub fn host_kernel_oracle(b158_w: &[u8; 4], nvfp4_w: &[u8; 8]) -> (i32, i32) {
    let mut b158_acc = 0i32;
    let mut idx = 0usize;
    for &byte in b158_w {
        let mut code = byte as u32;
        for _ in 0..5 {
            let trit = (code % 3) as i32 - 1;
            code /= 3;
            b158_acc += trit * DEFAULT_SENSORS[idx];
            idx += 1;
        }
    }
    let mut nvfp4_acc = 0i32;
    for (bi, &byte) in nvfp4_w.iter().enumerate() {
        let lo = (byte & 0x0F) as usize;
        let hi = (byte >> 4) as usize;
        nvfp4_acc += NVFP4_HALF_UNITS[lo] * DEFAULT_FEATS[bi * 2];
        nvfp4_acc += NVFP4_HALF_UNITS[hi] * DEFAULT_FEATS[bi * 2 + 1];
    }
    (b158_acc, nvfp4_acc)
}

pub fn find_qemu_binary() -> Option<PathBuf> {
    let mut candidates = vec![PathBuf::from("qemu-system-x86_64.exe")];
    if let Ok(home) = std::env::var("USERPROFILE") {
        candidates.push(PathBuf::from(&home).join("scoop").join("shims").join("qemu-system-x86_64.exe"));
        candidates.push(PathBuf::from(&home).join("scoop").join("apps").join("qemu").join("current").join("qemu-system-x86_64.exe"));
    }
    candidates.push(PathBuf::from(r"C:\Program Files\qemu\qemu-system-x86_64.exe"));
    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }
    // Check PATH fallback
    if Command::new("qemu-system-x86_64").arg("--version").output().is_ok() {
        return Some(PathBuf::from("qemu-system-x86_64"));
    }
    None
}

fn find_zig_binary() -> PathBuf {
    let win_default = PathBuf::from(r"C:\tools\zig\zig.exe");
    if win_default.is_file() {
        return win_default;
    }
    PathBuf::from("zig")
}

#[derive(Debug, Clone)]
pub struct QemuBootReport {
    pub qemu_available: bool,
    pub booted_cleanly: bool,
    pub multiboot1_ok: bool,
    pub serial_b158: i32,
    pub serial_nvfp4: i32,
    pub expected_b158: i32,
    pub expected_nvfp4: i32,
    pub bit_exact_match: bool,
    pub raw_serial: String,
}

pub fn boot_kernel_in_qemu(
    elf32_path: &Path,
    expected_b158: i32,
    expected_nvfp4: i32,
) -> Result<QemuBootReport, String> {
    let Some(qemu) = find_qemu_binary() else {
        return Ok(QemuBootReport {
            qemu_available: false,
            booted_cleanly: false,
            multiboot1_ok: false,
            serial_b158: 0,
            serial_nvfp4: 0,
            expected_b158,
            expected_nvfp4,
            bit_exact_match: false,
            raw_serial: String::new(),
        });
    };

    let output = Command::new(&qemu)
        .arg("-kernel")
        .arg(elf32_path)
        .arg("-display")
        .arg("none")
        .arg("-serial")
        .arg("stdio")
        .arg("-monitor")
        .arg("none")
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
        .arg("-no-reboot")
        .arg("-m")
        .arg("32M")
        .output()
        .map_err(|e| format!("failed to launch {}: {e}", qemu.display()))?;

    let serial = String::from_utf8_lossy(&output.stdout).replace('\r', "");
    let multiboot1_ok = serial.contains("MULTIBOOT1_OK") && serial.contains("SELF_TEST=PASS");

    let mut serial_b158 = i32::MIN;
    let mut serial_nvfp4 = i32::MIN;
    for tok in serial.split_whitespace() {
        if let Some(v) = tok.strip_prefix("B158_REFLEX=") {
            if let Ok(n) = v.parse::<i32>() {
                serial_b158 = n;
            }
        } else if let Some(v) = tok.strip_prefix("NVFP4_PRIO=") {
            if let Ok(n) = v.parse::<i32>() {
                serial_nvfp4 = n;
            }
        }
    }

    let bit_exact_match =
        multiboot1_ok && serial_b158 == expected_b158 && serial_nvfp4 == expected_nvfp4;
    if !bit_exact_match {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "QEMU Ring-0 boot mismatch: expected ({expected_b158}, {expected_nvfp4}), got ({serial_b158}, {serial_nvfp4}) | stdout={serial:?} | stderr={stderr:?}"
        ));
    }

    Ok(QemuBootReport {
        qemu_available: true,
        booted_cleanly: true,
        multiboot1_ok,
        serial_b158,
        serial_nvfp4,
        expected_b158,
        expected_nvfp4,
        bit_exact_match,
        raw_serial: serial,
    })
}

#[derive(Debug, Clone)]
pub struct ForgedKernelReport {
    pub name: String,
    pub elf_path: PathBuf,
    pub elf_bytes: usize,
    pub elf_sha256_prefix: String,
    pub entry_addr: u32,
    pub oracle_b158: i32,
    pub oracle_nvfp4: i32,
    pub qemu_booted: bool,
    pub qemu_bit_exact: bool,
    pub planaria_healed_exact: bool,
    pub planaria_trojans_quarantined: usize,
}

fn sanitize_name(name: &str) -> Result<String, String> {
    if name.is_empty() || name.len() > 32 || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return Err(format!("MUTANT CAUGHT: unsafe forge artifact name '{name}'"));
    }
    Ok(name.to_string())
}

/// Autonomously synthesizes, compiles, ELF32-audits, QEMU-boots, and Planaria-protects a neural OS microkernel!
pub fn forge_neural_kernel(
    repo_root: &Path,
    raw_name: &str,
    seed: u64,
) -> Result<ForgedKernelReport, String> {
    let name = sanitize_name(raw_name)?;
    let forged_dir = repo_root.join("bin").join("forged");
    fs::create_dir_all(&forged_dir).map_err(|e| format!("mkdir forged: {e}"))?;

    // Synthesize evolved b1.58 (mod 243) + NVFP4 (full byte) neural weights deterministically via Philox4x32-10
    let mut b158_w = [0u8; 4];
    for (i, w) in b158_w.iter_mut().enumerate() {
        *w = (philox_u64(seed, i as u64, 0xF026_E001) % 243) as u8;
    }
    let mut nvfp4_w = [0u8; 8];
    for (i, w) in nvfp4_w.iter_mut().enumerate() {
        *w = (philox_u64(seed, i as u64, 0xF026_E002) & 0xFF) as u8;
    }
    let (oracle_b158, oracle_nvfp4) = host_kernel_oracle(&b158_w, &nvfp4_w);

    // Emit complete freestanding Zig 0.17.1 Multiboot1 kernel source
    let template = fs::read_to_string(repo_root.join("zig").join("freestanding_kernel").join("src").join("boot32.zig"))
        .map_err(|e| format!("read boot32.zig template: {e}"))?;
    let custom_b158 = format!(
        "const REFLEX_WEIGHTS_B158: [4]u8 = .{{ {}, {}, {}, {} }};",
        b158_w[0], b158_w[1], b158_w[2], b158_w[3]
    );
    let custom_nvfp4 = format!(
        "const PRIORITY_WEIGHTS_NVFP4: [8]u8 = .{{ 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X}, 0x{:02X} }};",
        nvfp4_w[0], nvfp4_w[1], nvfp4_w[2], nvfp4_w[3], nvfp4_w[4], nvfp4_w[5], nvfp4_w[6], nvfp4_w[7]
    );
    let zig_src = template
        .replace(
            "const REFLEX_WEIGHTS_B158: [4]u8 = .{ 161, 82, 200, 41 };",
            &custom_b158,
        )
        .replace(
            "const PRIORITY_WEIGHTS_NVFP4: [8]u8 = .{ 0x25, 0xA3, 0x41, 0xB6, 0x17, 0x92, 0x34, 0xC5 };",
            &custom_nvfp4,
        );

    let zig_path = forged_dir.join(format!("{name}_boot32.zig"));
    let elf_path = forged_dir.join(format!("{name}_boot32.elf"));
    let linker_path = repo_root.join("zig").join("freestanding_kernel").join("linker32.ld");
    fs::write(&zig_path, &zig_src).map_err(|e| format!("write forged zig: {e}"))?;

    let zig = find_zig_binary();
    let status = Command::new(&zig)
        .arg("build-exe")
        .arg(&zig_path)
        .arg("-target")
        .arg("x86-freestanding-none")
        .arg("-OReleaseSmall")
        .arg("-T")
        .arg(&linker_path)
        .arg(format!("-femit-bin={}", elf_path.display()))
        .output()
        .map_err(|e| format!("zig compile forged kernel: {e}"))?;
    if !status.status.success() {
        return Err(format!(
            "forged kernel zig compile failed: {}",
            String::from_utf8_lossy(&status.stderr)
        ));
    }

    // Audit compiled ELF32 + Multiboot1 header checksum
    let audit = audit_elf32_file(&elf_path)?;
    let elf_raw = fs::read(&elf_path).map_err(|e| format!("read forged elf: {e}"))?;
    let elf_sha = hex_encode(&sha256(&elf_raw));

    // Live-boot freshly forged kernel inside QEMU Ring-0 (if installed)
    let qemu = boot_kernel_in_qemu(&elf_path, oracle_b158, oracle_nvfp4)?;

    // Wrap the real compiled OS kernel binary inside a Planaria GF(2^8) + Depth-3 Merkle capsule,
    // annihilate 50% of its data shards, inject a Byzantine Trojan shard, and prove 0-bit exact heal!
    let capsule = PlanariaCapsule::encode(&elf_raw);
    let mut survivors = vec![
        capsule.shards[4].clone(),
        capsule.shards[5].clone(),
        capsule.shards[6].clone(),
        capsule.shards[7].clone(),
    ];
    let mut trojan = capsule.shards[1].clone();
    trojan.data[0] ^= 0xA5;
    survivors.insert(0, trojan);
    let (healed, quarantined) = capsule.heal(&survivors).map_err(|e| e.to_string())?;
    let planaria_healed_exact = healed == elf_raw && quarantined == 1;
    if !planaria_healed_exact {
        return Err("forged kernel Planaria self-healing failed!".into());
    }

    Ok(ForgedKernelReport {
        name,
        elf_path,
        elf_bytes: audit.file_bytes,
        elf_sha256_prefix: elf_sha[..16].to_string(),
        entry_addr: audit.entry_addr,
        oracle_b158,
        oracle_nvfp4,
        qemu_booted: qemu.booted_cleanly,
        qemu_bit_exact: qemu.bit_exact_match,
        planaria_healed_exact,
        planaria_trojans_quarantined: quarantined,
    })
}

#[derive(Debug, Clone)]
pub struct ForgedModelReport {
    pub name: String,
    pub gguf_bytes: usize,
    pub bcg_bytes: usize,
    pub repo_merkle_prefix: String,
    pub swat_cos: f64,
    pub planaria_heal_exact: bool,
    pub inference_exact: bool,
}

/// Autonomously breeds & exports a cross-architecture Chimera GGUF v3 + Merkle-grounded `.bcg` CacheGraft bank!
pub fn forge_chimera_model(
    repo_root: &Path,
    raw_name: &str,
    seed: u64,
) -> Result<ForgedModelReport, String> {
    let name = sanitize_name(raw_name)?;
    let forged_dir = repo_root.join("bin").join("forged");
    fs::create_dir_all(&forged_dir).map_err(|e| format!("mkdir forged: {e}"))?;

    let rmap = build_merkle_repo_map(repo_root, 16_384)?;
    let gguf_path = forged_dir.join(format!("{name}.gguf"));
    let xarch = run_cross_arch_c2c_gguf_planaria(&gguf_path, seed)?;

    let (trained_mapper, _) = train_and_verify_swat_mapper(32, 24, 90, seed);
    let files = [
        "AGENTS.md",
        "Cargo.toml",
        "zig/beast-metal/src/root.zig",
        "zig/freestanding_kernel/src/boot32.zig",
        "lean/BeastProofs.lean",
    ];
    let bank = build_repo_cachegraft_bank(repo_root, &files, &trained_mapper, 12)?;
    let bcg_path = forged_dir.join(format!("{name}.bcg"));
    let bcg_bytes = save_bcg(&bank, &bcg_path)?;
    let reloaded = load_bcg(&bcg_path)?;
    if reloaded.sha256_digest != bank.sha256_digest {
        return Err("forged .bcg reload digest mismatch!".into());
    }

    Ok(ForgedModelReport {
        name,
        gguf_bytes: xarch.gguf_bytes,
        bcg_bytes,
        repo_merkle_prefix: rmap.merkle_root_hex[..16].to_string(),
        swat_cos: xarch.swat_heldout_cos,
        planaria_heal_exact: xarch.heal_bit_exact,
        inference_exact: xarch.inference_after_heal_exact,
    })
}
