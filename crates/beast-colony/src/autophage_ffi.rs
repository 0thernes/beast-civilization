//! Live dynamic C-ABI bridge to `Z:\[Vibe Coded (AI)]\ANTIGRAVITY\autophage\bin\autophage.dll`
//! (0thernes AUTOPHAGE v1.3.0 polymorphic x86-64 RWX JIT genetic engine).

use beast_wire::dynlib::DynLib;
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
        Self {
            vendor: [0; 16],
            brand: [0; 64],
            has_avx2: 0,
            has_bmi1: 0,
            has_bmi2: 0,
            has_popcnt: 0,
            has_pclmulqdq: 0,
            has_vpclmulqdq: 0,
            has_gfni: 0,
            has_rdseed: 0,
            has_rdrand: 0,
            has_rdtscp: 0,
            jit_rwx_verified: 0,
            reserved: 0,
        }
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
