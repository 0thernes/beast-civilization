//! Live dynamic C-ABI bridge to `Z:\[Vibe Coded (AI)]\ANTIGRAVITY\autophage\bin\autophage.dll`
//! (0thernes AUTOPHAGE v1.3.0 polymorphic x86-64 RWX JIT genetic engine).
//! Supports BOTH hardware capability probing AND live bare-metal RWX JIT world stepping!

use beast_wire::dynlib::DynLib;
use std::ffi::c_void;
use std::path::Path;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AutophageHwCaps {
    pub vendor: [u8; 16],
    pub brand: [u8; 64],
    pub has_avx2: u32,
    pub has_bmi1: u32,
    pub has_bmi2: u32,
    pub has_popcnt: u32,
    pub has_pclmulqdq: u32,
    pub has_vpclmulqdq: u32,
    pub has_gfni: u32,
    pub has_rdseed: u32,
    pub has_rdrand: u32,
    pub has_rdtscp: u32,
    pub jit_rwx_verified: u32,
    pub reserved: u32,
}

impl Default for AutophageHwCaps {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AutophageConfig {
    pub width: u32,
    pub height: u32,
    pub threads: u32,
    pub regime: u32,
    pub kernel_mode: u32,
    pub base_gas: u32,
    pub seed: u64,
    pub energy_inflow_uq: u32,
    pub thermal_threshold_uq: u32,
    pub restraints: [u8; 13],
    pub pad: [u8; 3],
}

impl Default for AutophageConfig {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[derive(Debug, Clone)]
pub struct AutophageProbeReport {
    pub available: bool,
    pub abi_version: u32,
    pub cpu_brand: String,
    pub has_avx2: bool,
    pub has_gfni: bool,
    pub has_pclmulqdq: bool,
    pub has_rdseed: bool,
    pub jit_rwx_verified: bool,
}

#[derive(Debug, Clone)]
pub struct AutophageLiveRunReport {
    pub available: bool,
    pub epochs_stepped: u32,
    pub world_cells: u32,
    pub state_sha256_jit: String,
    pub state_sha256_scalar: String,
    pub bit_exact_jit_vs_scalar: bool,
}

pub fn probe_local_autophage(dll_path: &Path) -> AutophageProbeReport {
    let lib = match DynLib::open(dll_path) {
        Ok(l) => l,
        Err(_) => {
            return AutophageProbeReport {
                available: false,
                abi_version: 0,
                cpu_brand: "autophage.dll not present".to_string(),
                has_avx2: false,
                has_gfni: false,
                has_pclmulqdq: false,
                has_rdseed: false,
                jit_rwx_verified: false,
            };
        }
    };

    let ver_fn: extern "C" fn() -> u32 = match lib.symbol_raw("autophage_abi_version") {
        Ok(p) => unsafe { std::mem::transmute(p) },
        Err(_) => {
            return AutophageProbeReport {
                available: false,
                abi_version: 0,
                cpu_brand: "missing autophage_abi_version".to_string(),
                has_avx2: false,
                has_gfni: false,
                has_pclmulqdq: false,
                has_rdseed: false,
                jit_rwx_verified: false,
            };
        }
    };
    let abi_version = ver_fn();

    let mut caps = AutophageHwCaps::default();
    if let Ok(p) = lib.symbol_raw("autophage_probe_hardware") {
        let probe_fn: extern "C" fn(*mut AutophageHwCaps) = unsafe { std::mem::transmute(p) };
        probe_fn(&mut caps as *mut AutophageHwCaps);
    }
    let nul = caps.brand.iter().position(|&b| b == 0).unwrap_or(caps.brand.len());
    let brand = String::from_utf8_lossy(&caps.brand[..nul]).trim().to_string();

    AutophageProbeReport {
        available: true,
        abi_version,
        cpu_brand: brand,
        has_avx2: caps.has_avx2 != 0,
        has_gfni: caps.has_gfni != 0,
        has_pclmulqdq: caps.has_pclmulqdq != 0,
        has_rdseed: caps.has_rdseed != 0,
        jit_rwx_verified: caps.jit_rwx_verified != 0,
    }
}

/// Step a real live AUTOPHAGE v1.3.0 world in BOTH Native x86-64 RWX JIT mode AND Scalar Ref mode
/// from inside Rust, proving 0-bit cross-kernel SHA-256 parity!
pub fn step_live_autophage_world(dll_path: &Path, seed: u64, epochs: u32) -> AutophageLiveRunReport {
    let lib = match DynLib::open(dll_path) {
        Ok(l) => l,
        Err(_) => {
            return AutophageLiveRunReport {
                available: false,
                epochs_stepped: 0,
                world_cells: 0,
                state_sha256_jit: String::new(),
                state_sha256_scalar: String::new(),
                bit_exact_jit_vs_scalar: false,
            };
        }
    };

    macro_rules! sym {
        ($n:literal, $t:ty) => {
            match lib.symbol_raw($n) {
                Ok(p) => unsafe { std::mem::transmute::<*mut c_void, $t>(p) },
                Err(_) => {
                    return AutophageLiveRunReport {
                        available: false,
                        epochs_stepped: 0,
                        world_cells: 0,
                        state_sha256_jit: String::new(),
                        state_sha256_scalar: String::new(),
                        bit_exact_jit_vs_scalar: false,
                    };
                }
            }
        };
    }

    let default_cfg = sym!("autophage_default_config", extern "C" fn(*mut AutophageConfig));
    let create = sym!("autophage_create", extern "C" fn(*const AutophageConfig) -> *mut c_void);
    let step = sym!("autophage_step_epochs", extern "C" fn(*mut c_void, u32));
    let state_hash = sym!("autophage_state_sha256", extern "C" fn(*const c_void, *mut u8, *mut u8));
    let world_size = sym!("autophage_world_size", extern "C" fn(*const c_void) -> u32);
    let destroy = sym!("autophage_destroy", extern "C" fn(*mut c_void));

    let run_mode = |kernel_mode: u32| -> (u32, String) {
        let mut cfg = AutophageConfig::default();
        default_cfg(&mut cfg);
        cfg.width = 16;
        cfg.height = 16;
        cfg.threads = 1;
        cfg.regime = 0; // Feral Abiogenesis (Track B, 0% castration!)
        cfg.kernel_mode = kernel_mode;
        cfg.seed = seed;
        let w = create(&cfg);
        assert!(!w.is_null(), "autophage_create returned null");
        let n_cells = world_size(w);
        step(w, epochs);
        let mut digest = [0u8; 32];
        let mut hex = [0u8; 65];
        state_hash(w, digest.as_mut_ptr(), hex.as_mut_ptr());
        destroy(w);
        let nul = hex.iter().position(|&b| b == 0).unwrap_or(64);
        (n_cells, String::from_utf8_lossy(&hex[..nul]).to_string())
    };

    let (cells, sha_jit) = run_mode(0);    // AUTOPHAGE_KERNEL_NATIVE_JIT (Live RWX x86-64!)
    let (_, sha_scalar) = run_mode(1);     // AUTOPHAGE_KERNEL_SCALAR_REF
    let bit_exact = !sha_jit.is_empty() && sha_jit == sha_scalar;

    AutophageLiveRunReport {
        available: true,
        epochs_stepped: epochs,
        world_cells: cells,
        state_sha256_jit: sha_jit,
        state_sha256_scalar: sha_scalar,
        bit_exact_jit_vs_scalar: bit_exact,
    }
}
