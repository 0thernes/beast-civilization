//! Unified Live Native DLL FFI Bridge across Tsotchke & Local 0thernes Bare-Metal Ecosystems (12-DLL Stack):
//!   - `bin/tsotchke_pinn_bridge.dll`          (compiled upstream `tsotchke/PINN`)
//!   - `bin/tsotchke_libirrep_bridge.dll`      (compiled upstream `tsotchke/libirrep` SO(3) AVX2+FMA spherical harmonics)
//!   - `bin/tsotchke_moonlab_qgt_bridge.dll`   (compiled upstream `tsotchke/moonlab` QGT Chern/SSH/Kitaev topology engine)
//!   - `bin/tsotchke_moonlab_mlkem_bridge.dll` (compiled upstream `tsotchke/moonlab` FIPS-203 ML-KEM-768 + FIPS-197 AES-256)
//!   - `bin/tsotchke_sbnn_bridge.dll`          (compiled upstream `tsotchke/spin_based_neural_network` Fibonacci Anyons + p-bit Ising)
//!   - `bin/tsotchke_qgtl_bridge.dll`          (compiled upstream `tsotchke/quantum_geometric_tensor` Bloch complex fidelity)
//!   - `Z:\[Vibe Coded (AI)]\ANTIGRAVITY\cellular-engine-omega\bin\omega_cell.dll`
//!   - `Z:\[Vibe Coded (AI)]\ANTIGRAVITY\cellulogonic-mecha\bin\cellulogonic.dll`

use beast_wire::dynlib::DynLib;
use beast_wire::sha256::hex_encode;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EcosystemFfiReport {
    pub pinn_available: bool,
    pub pinn_out: [f64; 3],
    pub libirrep_available: bool,
    pub libirrep_unsold_ok: bool,
    pub libirrep_total_norm_sq: f64,
    pub moonlab_qgt_available: bool,
    pub moonlab_qwz_chern_topo: i32,
    pub moonlab_qwz_chern_triv: i32,
    pub moonlab_ssh_winding: i32,
    pub moonlab_kitaev_z2: i32,
    pub moonlab_fs_trace: f64,
    pub moonlab_berry_omega: f64,
    pub mlkem768_available: bool,
    pub mlkem768_fo_tamper_rejected: bool,
    pub mlkem768_secret_prefix: String,
    pub sbnn_available: bool,
    pub sbnn_yang_baxter_err: f64,
    pub sbnn_fibo_phi: f64,
    pub sbnn_pbit_energy: f64,
    pub sbnn_fibo_dim_8: i32,
    pub qgtl_available: bool,
    pub qgtl_fidelity: f32,
    pub qgtl_ortho_err: f32,
    pub omega_available: bool,
    pub omega_sha256_match: bool,
    pub cellulogonic_available: bool,
    pub cellulogonic_abi: u32,
}

pub fn probe_full_ecosystem_dlls(repo_root: &Path) -> EcosystemFfiReport {
    let parent_dir = repo_root.parent().unwrap_or(repo_root);
    let bin_dir = repo_root.join("bin");

    // 1. Compiled upstream `tsotchke/PINN`
    let (pinn_available, pinn_out) = match DynLib::open(&bin_dir.join("tsotchke_pinn_bridge.dll")) {
        Ok(lib) => match lib.symbol_raw("beast_tsotchke_pinn_verify") {
            Ok(p) => {
                let f: extern "C" fn(f64, f64, *mut f64) -> i32 = unsafe { std::mem::transmute(p) };
                let mut out = [0.0f64; 3];
                let rc = f(0.42, -0.73, out.as_mut_ptr());
                (rc == 0 && out.iter().all(|v| v.is_finite()), out)
            }
            Err(_) => (false, [0.0; 3]),
        },
        Err(_) => (false, [0.0; 3]),
    };

    // 2. Compiled upstream `tsotchke/libirrep`
    let (libirrep_available, libirrep_unsold_ok, libirrep_total_norm_sq) =
        match DynLib::open(&bin_dir.join("tsotchke_libirrep_bridge.dll")) {
            Ok(lib) => match lib.symbol_raw("beast_tsotchke_libirrep_sph_verify") {
                Ok(p) => {
                    let f: extern "C" fn(f64, f64, f64, *mut f64, *mut f64) -> i32 =
                        unsafe { std::mem::transmute(p) };
                    let mut sh = [0.0f64; 9];
                    let mut norm_sq = 0.0f64;
                    let rc = f(0.3, -0.7, 0.65, sh.as_mut_ptr(), &mut norm_sq);
                    (true, rc == 0, norm_sq)
                }
                Err(_) => (false, false, 0.0),
            },
            Err(_) => (false, false, 0.0),
        };

    // 3. Compiled upstream `tsotchke/moonlab` QGT Topology
    let (
        moonlab_qgt_available,
        moonlab_qwz_chern_topo,
        moonlab_qwz_chern_triv,
        moonlab_ssh_winding,
        moonlab_kitaev_z2,
        moonlab_fs_trace,
        moonlab_berry_omega,
    ) = match DynLib::open(&bin_dir.join("tsotchke_moonlab_qgt_bridge.dll")) {
        Ok(lib) => match lib.symbol_raw("beast_tsotchke_moonlab_qgt_verify") {
            Ok(p) => {
                let f: extern "C" fn(
                    *mut i32, *mut i32, *mut i32, *mut i32, *mut f64, *mut f64,
                ) -> i32 = unsafe { std::mem::transmute(p) };
                let (mut c_topo, mut c_triv, mut w_ssh, mut z2) = (0, -1, 0, 0);
                let (mut fs_tr, mut omega) = (0.0, 0.0);
                let rc = f(&mut c_topo, &mut c_triv, &mut w_ssh, &mut z2, &mut fs_tr, &mut omega);
                (rc == 0, c_topo, c_triv, w_ssh, z2, fs_tr, omega)
            }
            Err(_) => (false, 0, -1, 0, 0, 0.0, 0.0),
        },
        Err(_) => (false, 0, -1, 0, 0, 0.0, 0.0),
    };

    // 4. Compiled upstream `tsotchke/moonlab` FIPS-203 ML-KEM-768 + FIPS-197 AES-256
    let (mlkem768_available, mlkem768_fo_tamper_rejected, mlkem768_secret_prefix) =
        match DynLib::open(&bin_dir.join("tsotchke_moonlab_mlkem_bridge.dll")) {
            Ok(lib) => match lib.symbol_raw("beast_tsotchke_mlkem768_verify") {
                Ok(p) => {
                    let f: extern "C" fn(u64, *mut u8, *mut i32) -> i32 =
                        unsafe { std::mem::transmute(p) };
                    let mut secret = [0u8; 32];
                    let mut fo_rej = 0i32;
                    let rc = f(5000, secret.as_mut_ptr(), &mut fo_rej);
                    let hex = hex_encode(&secret);
                    (rc == 0, fo_rej == 1, hex[..16].to_string())
                }
                Err(_) => (false, false, String::new()),
            },
            Err(_) => (false, false, String::new()),
        };

    // 5. Compiled upstream `tsotchke/spin_based_neural_network` Fibonacci Anyons + Spintronic p-bit
    let (sbnn_available, sbnn_yang_baxter_err, sbnn_fibo_phi, sbnn_pbit_energy, sbnn_fibo_dim_8) =
        match DynLib::open(&bin_dir.join("tsotchke_sbnn_bridge.dll")) {
            Ok(lib) => match lib.symbol_raw("beast_tsotchke_sbnn_verify") {
                Ok(p) => {
                    let f: extern "C" fn(u64, *mut f64, *mut f64, *mut f64, *mut i32) -> i32 =
                        unsafe { std::mem::transmute(p) };
                    let (mut yb, mut phi, mut pe) = (1.0, 0.0, 0.0);
                    let mut dim8 = 0i32;
                    let rc = f(5000, &mut yb, &mut phi, &mut pe, &mut dim8);
                    (rc == 0, yb, phi, pe, dim8)
                }
                Err(_) => (false, 1.0, 0.0, 0.0, 0),
            },
            Err(_) => (false, 1.0, 0.0, 0.0, 0),
        };

    // 6. Compiled upstream `tsotchke/quantum_geometric_tensor` (`QGTL`) Bloch complex fidelity
    let (qgtl_available, qgtl_fidelity, qgtl_ortho_err) =
        match DynLib::open(&bin_dir.join("tsotchke_qgtl_bridge.dll")) {
            Ok(lib) => match lib.symbol_raw("beast_tsotchke_qgtl_verify") {
                Ok(p) => {
                    let f: extern "C" fn(f32, f32, f32, *mut f32, *mut f32) -> i32 =
                        unsafe { std::mem::transmute(p) };
                    let (mut fid, mut oerr) = (0.0f32, 1.0f32);
                    let rc = f(0.5, 1.5471976, 0.8, &mut fid, &mut oerr);
                    (rc == 0, fid, oerr)
                }
                Err(_) => (false, 0.0, 1.0),
            },
            Err(_) => (false, 0.0, 1.0),
        };

    // 7. Local 0thernes `cellular-engine-omega` (`omega_cell.dll`)
    let omega_path = parent_dir
        .join("cellular-engine-omega")
        .join("bin")
        .join("omega_cell.dll");
    let (omega_available, omega_sha256_match) = match DynLib::open(&omega_path) {
        Ok(lib) => match lib.symbol_raw("omega_sha256_hex") {
            Ok(p) => {
                let f: extern "C" fn(*const u8, usize, *mut u8) = unsafe { std::mem::transmute(p) };
                let msg = b"abc";
                let mut hex = [0u8; 65];
                f(msg.as_ptr(), msg.len(), hex.as_mut_ptr());
                let s = String::from_utf8_lossy(&hex[..64]);
                (
                    true,
                    s == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                )
            }
            Err(_) => (false, false),
        },
        Err(_) => (false, false),
    };

    // 8. Local 0thernes `cellulogonic-mecha` (`cellulogonic.dll`)
    let cell_path = parent_dir
        .join("cellulogonic-mecha")
        .join("bin")
        .join("cellulogonic.dll");
    let (cellulogonic_available, cellulogonic_abi) = match DynLib::open(&cell_path) {
        Ok(lib) => match lib.symbol_raw("cellulogonic_abi_version") {
            Ok(p) => {
                let f: extern "C" fn() -> u32 = unsafe { std::mem::transmute(p) };
                (true, f())
            }
            Err(_) => (false, 0),
        },
        Err(_) => (false, 0),
    };

    EcosystemFfiReport {
        pinn_available,
        pinn_out,
        libirrep_available,
        libirrep_unsold_ok,
        libirrep_total_norm_sq,
        moonlab_qgt_available,
        moonlab_qwz_chern_topo,
        moonlab_qwz_chern_triv,
        moonlab_ssh_winding,
        moonlab_kitaev_z2,
        moonlab_fs_trace,
        moonlab_berry_omega,
        mlkem768_available,
        mlkem768_fo_tamper_rejected,
        mlkem768_secret_prefix,
        sbnn_available,
        sbnn_yang_baxter_err,
        sbnn_fibo_phi,
        sbnn_pbit_energy,
        sbnn_fibo_dim_8,
        qgtl_available,
        qgtl_fidelity,
        qgtl_ortho_err,
        omega_available,
        omega_sha256_match,
        cellulogonic_available,
        cellulogonic_abi,
    }
}
