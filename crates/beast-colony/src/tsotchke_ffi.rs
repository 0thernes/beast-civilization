//! Live FFI bridge into compiled upstream `vendor/tsotchke__quantum_rng` (`bin/tsotchke_qrng_bridge.dll`).

use beast_wire::dynlib::DynLib;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TsotchkeBellReport {
    pub available: bool,
    pub chsh_s: f64,
    pub violates_classical: bool,
}

pub fn run_upstream_tsotchke_bell(dll_path: &Path, seed: u64, shots: usize) -> TsotchkeBellReport {
    let lib = match DynLib::open(dll_path) {
        Ok(l) => l,
        Err(_) => return TsotchkeBellReport { available: false, chsh_s: 0.0, violates_classical: false },
    };
    let f: extern "C" fn(u64, usize, *mut i32) -> f64 = match lib.symbol_raw("beast_tsotchke_run_bell_chsh") {
        Ok(p) => unsafe {
            std::mem::transmute::<*mut std::ffi::c_void, extern "C" fn(u64, usize, *mut i32) -> f64>(p)
        },
        Err(_) => return TsotchkeBellReport { available: false, chsh_s: 0.0, violates_classical: false },
    };
    let mut violates = 0i32;
    let s = f(seed, shots, &mut violates as *mut i32);
    TsotchkeBellReport {
        available: true,
        chsh_s: s,
        violates_classical: violates != 0 && s > 2.0,
    }
}
