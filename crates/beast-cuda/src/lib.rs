//! `beast-cuda`: Direct CUDA Driver API (`nvcuda.dll` / `libcuda.so.1`) 5-Kernel PTX JIT Suite
//! + Pinned Host-RAM <-> VRAM Async KV Paging (`W059`, `F058`) on NVIDIA RTX 5070 Ti (`sm_12.0` Blackwell)
//!   with ZERO nvcc and ZERO MSVC:
//!   1. `beast_b158_rowdot`         — 5-trit/byte ternary matvec (bit-exact int64 CPU parity)
//!   2. `beast_nvfp4_rowdot`        — OCP Blackwell FP4 E2M1 matvec (eps < 1e-4 CPU parity)
//!   3. `beast_c2c_augment_scores`  — MIT Augment-Cache parallel [K_self; K_peer] scoring (eps < 1e-4)
//!   4. `beast_mla_latent_compress` — DeepSeek/Qwen MLA 4x low-rank KV latent compression (eps < 1e-4)
//!   5. `beast_moe_swiglu_sparse`   — Sparse MoE Top-2 Gated SwiGLU FFN activation (eps < 1e-4)
//!   + Pinned Host Memory (`cuMemAllocHost_v2` / `cuMemcpyHtoDAsync_v2` / `cuMemcpyDtoHAsync_v2` on `CUstream`)

use beast_wire::dynlib::DynLib;
use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::path::Path;
use std::time::Instant;

const CUDA_PTX_SRC: &str = include_str!("../../../cuda/beast_blackwell.ptx");
const NVFP4_LUT: [f32; 16] = [
    0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0,
    -0.0, -0.5, -1.0, -1.5, -2.0, -3.0, -4.0, -6.0,
];

type CuResult = c_int;
type CuDevice = c_int;
type CuContext = *mut c_void;
type CuModule = *mut c_void;
type CuFunction = *mut c_void;
type CuStream = *mut c_void;
type CuDevicePtr = u64;

const CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR: c_int = 75;
const CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR: c_int = 76;
const CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT: c_int = 16;

#[derive(Debug, Clone)]
pub struct CudaGpuReport {
    pub available: bool,
    pub device_name: String,
    pub compute_cap: String,
    pub sm_count: i32,
    pub rows_tested: u32,
    pub n_trits_per_row: u32,
    pub gpu_kernel_us: u64,
    pub nvfp4_kernel_us: u64,
    pub c2c_scores_us: u64,
    pub mla_compress_us: u64,
    pub moe_swiglu_us: u64,
    pub pinned_paging_ok: bool,
    pub pinned_paging_gbps: f64,
    pub bit_exact_cpu_match: bool,
    pub nvfp4_exact_match: bool,
    pub c2c_scores_match: bool,
    pub mla_exact_match: bool,
    pub moe_exact_match: bool,
    pub kernels_verified: u32,
    pub error_msg: Option<String>,
}

fn cpu_b158_rowdot(packed_w: &[u8], x: &[i32], rows: usize, bytes_per_row: usize) -> Vec<i64> {
    let mut out = vec![0i64; rows];
    for r in 0..rows {
        let mut acc = 0i64;
        for b in 0..bytes_per_row {
            let mut code = packed_w[r * bytes_per_row + b] as u32;
            for t in 0..5 {
                let trit = (code % 3) as i64 - 1;
                code /= 3;
                acc += trit * (x[b * 5 + t] as i64);
            }
        }
        out[r] = acc;
    }
    out
}

fn cpu_nvfp4_rowdot(packed_w: &[u8], x: &[f32], rows: usize, bytes_per_row: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; rows];
    for r in 0..rows {
        let mut acc = 0.0f64;
        for b in 0..bytes_per_row {
            let byte = packed_w[r * bytes_per_row + b];
            let lo = (byte & 0x0F) as usize;
            let hi = (byte >> 4) as usize;
            acc += (NVFP4_LUT[lo] as f64) * (x[b * 2] as f64);
            acc += (NVFP4_LUT[hi] as f64) * (x[b * 2 + 1] as f64);
        }
        out[r] = acc as f32;
    }
    out
}

fn cpu_c2c_scores(q: &[f32], k_self: &[f32], k_peer: &[f32], len_self: usize, len_peer: usize, dim: usize) -> Vec<f32> {
    let inv = 1.0 / (dim as f32).sqrt();
    let mut out = Vec::with_capacity(len_self + len_peer);
    for t in 0..len_self {
        let mut dot = 0.0f32;
        for d in 0..dim { dot += q[d] * k_self[t * dim + d]; }
        out.push(dot * inv);
    }
    for t in 0..len_peer {
        let mut dot = 0.0f32;
        for d in 0..dim { dot += q[d] * k_peer[t * dim + d]; }
        out.push(dot * inv);
    }
    out
}

fn cpu_mla_compress(h: &[f32], w_dkv: &[f32], seq_len: usize, d_in: usize, d_lat: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; seq_len * d_lat];
    for t in 0..seq_len {
        for c in 0..d_lat {
            let mut acc = 0.0f32;
            for d in 0..d_in {
                acc += h[t * d_in + d] * w_dkv[c * d_in + d];
            }
            out[t * d_lat + c] = acc;
        }
    }
    out
}

fn cpu_moe_swiglu(g0: &[f32], u0: &[f32], g1: &[f32], u1: &[f32], w0: f32, w1: f32) -> Vec<f32> {
    let silu = |x: f32| x / (1.0 + (-x).exp());
    (0..g0.len())
        .map(|i| w0 * silu(g0[i]) * u0[i] + w1 * silu(g1[i]) * u1[i])
        .collect()
}

pub fn run_blackwell_b158_verify(rows: u32, bytes_per_row: u32) -> CudaGpuReport {
    #[cfg(windows)]
    let lib_path = Path::new("C:\\Windows\\System32\\nvcuda.dll");
    #[cfg(not(windows))]
    let lib_path = Path::new("libcuda.so.1");

    let lib = match DynLib::open(lib_path) {
        Ok(l) => l,
        Err(e) => return err_report(&e),
    };

    macro_rules! cu {
        ($name:literal, $ty:ty) => {
            match lib.symbol_raw($name) {
                Ok(p) => unsafe { std::mem::transmute::<*mut c_void, $ty>(p) },
                Err(e) => return err_report(&e),
            }
        };
    }

    let cu_init = cu!("cuInit", unsafe extern "C" fn(c_uint) -> CuResult);
    let cu_device_get = cu!("cuDeviceGet", unsafe extern "C" fn(*mut CuDevice, c_int) -> CuResult);
    let cu_device_name = cu!("cuDeviceGetName", unsafe extern "C" fn(*mut c_char, c_int, CuDevice) -> CuResult);
    let cu_device_attr = cu!("cuDeviceGetAttribute", unsafe extern "C" fn(*mut c_int, c_int, CuDevice) -> CuResult);
    let cu_ctx_create = cu!("cuCtxCreate_v2", unsafe extern "C" fn(*mut CuContext, c_uint, CuDevice) -> CuResult);
    let cu_ctx_destroy = cu!("cuCtxDestroy_v2", unsafe extern "C" fn(CuContext) -> CuResult);
    let cu_ctx_sync = cu!("cuCtxSynchronize", unsafe extern "C" fn() -> CuResult);
    let cu_mod_load = cu!("cuModuleLoadData", unsafe extern "C" fn(*mut CuModule, *const c_void) -> CuResult);
    let cu_mod_unload = cu!("cuModuleUnload", unsafe extern "C" fn(CuModule) -> CuResult);
    let cu_mod_fn = cu!("cuModuleGetFunction", unsafe extern "C" fn(*mut CuFunction, CuModule, *const c_char) -> CuResult);
    let cu_mem_alloc = cu!("cuMemAlloc_v2", unsafe extern "C" fn(*mut CuDevicePtr, usize) -> CuResult);
    let cu_mem_free = cu!("cuMemFree_v2", unsafe extern "C" fn(CuDevicePtr) -> CuResult);
    let cu_mem_alloc_host = cu!("cuMemAllocHost_v2", unsafe extern "C" fn(*mut *mut c_void, usize) -> CuResult);
    let cu_mem_free_host = cu!("cuMemFreeHost", unsafe extern "C" fn(*mut c_void) -> CuResult);
    let cu_stream_create = cu!("cuStreamCreate", unsafe extern "C" fn(*mut CuStream, c_uint) -> CuResult);
    let cu_stream_sync = cu!("cuStreamSynchronize", unsafe extern "C" fn(CuStream) -> CuResult);
    let cu_stream_destroy = cu!("cuStreamDestroy_v2", unsafe extern "C" fn(CuStream) -> CuResult);
    let cu_htod = cu!("cuMemcpyHtoD_v2", unsafe extern "C" fn(CuDevicePtr, *const c_void, usize) -> CuResult);
    let cu_dtoh = cu!("cuMemcpyDtoH_v2", unsafe extern "C" fn(*mut c_void, CuDevicePtr, usize) -> CuResult);
    let cu_htod_async = cu!("cuMemcpyHtoDAsync_v2", unsafe extern "C" fn(CuDevicePtr, *const c_void, usize, CuStream) -> CuResult);
    let cu_dtoh_async = cu!("cuMemcpyDtoHAsync_v2", unsafe extern "C" fn(*mut c_void, CuDevicePtr, usize, CuStream) -> CuResult);
    let cu_launch = cu!(
        "cuLaunchKernel",
        unsafe extern "C" fn(
            CuFunction, c_uint, c_uint, c_uint, c_uint, c_uint, c_uint,
            c_uint, *mut c_void, *mut *mut c_void, *mut *mut c_void,
        ) -> CuResult
    );

    unsafe {
        if cu_init(0) != 0 { return err_report("cuInit failed"); }
        let mut dev: CuDevice = 0;
        if cu_device_get(&mut dev, 0) != 0 { return err_report("cuDeviceGet(0) failed"); }

        let mut name_buf = [0u8; 128];
        let _ = cu_device_name(name_buf.as_mut_ptr() as *mut c_char, 128, dev);
        let nul = name_buf.iter().position(|&b| b == 0).unwrap_or(128);
        let device_name = String::from_utf8_lossy(&name_buf[..nul]).to_string();

        let (mut maj, mut min, mut sm_count) = (0, 0, 0);
        let _ = cu_device_attr(&mut maj, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR, dev);
        let _ = cu_device_attr(&mut min, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR, dev);
        let _ = cu_device_attr(&mut sm_count, CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT, dev);

        let mut ctx: CuContext = std::ptr::null_mut();
        if cu_ctx_create(&mut ctx, 0, dev) != 0 { return err_report("cuCtxCreate_v2 failed"); }

        let ptx_c = CString::new(CUDA_PTX_SRC).unwrap();
        let mut module: CuModule = std::ptr::null_mut();
        let rc_mod = cu_mod_load(&mut module, ptx_c.as_ptr() as *const c_void);
        if rc_mod != 0 {
            cu_ctx_destroy(ctx);
            return err_report(&format!("cuModuleLoadData failed rc={rc_mod}"));
        }

        let rows_u = rows as usize;
        let bpr_u = bytes_per_row as usize;
        let threads = 128u32;
        let blocks = rows.div_ceil(threads);

        // ---- KERNEL 1: b1.58 5-trit/byte ----
        let fn1_name = CString::new("beast_b158_rowdot").unwrap();
        let mut fn1: CuFunction = std::ptr::null_mut();
        assert_eq!(cu_mod_fn(&mut fn1, module, fn1_name.as_ptr()), 0);
        let n_trits = bpr_u * 5;
        let packed_w: Vec<u8> = (0..(rows_u * bpr_u)).map(|i| ((i.wrapping_mul(37).wrapping_add(13)) % 243) as u8).collect();
        let x_i32: Vec<i32> = (0..n_trits).map(|i| ((i as i32 * 19) % 127) - 63).collect();
        let cpu_ref1 = cpu_b158_rowdot(&packed_w, &x_i32, rows_u, bpr_u);

        let (mut d_w, mut d_x, mut d_out) = (0u64, 0u64, 0u64);
        assert_eq!(cu_mem_alloc(&mut d_w, packed_w.len()), 0);
        assert_eq!(cu_mem_alloc(&mut d_x, x_i32.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_out, rows_u * 8), 0);
        assert_eq!(cu_htod(d_w, packed_w.as_ptr() as *const c_void, packed_w.len()), 0);
        assert_eq!(cu_htod(d_x, x_i32.as_ptr() as *const c_void, x_i32.len() * 4), 0);

        let (mut p_w, mut p_x, mut p_out, mut p_rows, mut p_bpr) = (d_w, d_x, d_out, rows, bytes_per_row);
        let mut params1: [*mut c_void; 5] = [
            &mut p_w as *mut u64 as *mut c_void,
            &mut p_x as *mut u64 as *mut c_void,
            &mut p_out as *mut u64 as *mut c_void,
            &mut p_rows as *mut u32 as *mut c_void,
            &mut p_bpr as *mut u32 as *mut c_void,
        ];
        let t0 = Instant::now();
        let rc1 = cu_launch(fn1, blocks, 1, 1, threads, 1, 1, 0, std::ptr::null_mut(), params1.as_mut_ptr(), std::ptr::null_mut());
        let rc1s = cu_ctx_sync();
        let gpu_kernel_us = t0.elapsed().as_micros().max(1) as u64;
        let mut gpu_out1 = vec![0i64; rows_u];
        if rc1 == 0 && rc1s == 0 {
            assert_eq!(cu_dtoh(gpu_out1.as_mut_ptr() as *mut c_void, d_out, rows_u * 8), 0);
        }
        cu_mem_free(d_w); cu_mem_free(d_x); cu_mem_free(d_out);
        let k1_ok = rc1 == 0 && rc1s == 0 && gpu_out1 == cpu_ref1;

        // ---- KERNEL 2: OCP Blackwell NVFP4 E2M1 ----
        let fn2_name = CString::new("beast_nvfp4_rowdot").unwrap();
        let mut fn2: CuFunction = std::ptr::null_mut();
        assert_eq!(cu_mod_fn(&mut fn2, module, fn2_name.as_ptr()), 0);
        let packed_fp4: Vec<u8> = (0..(rows_u * bpr_u)).map(|i| ((i.wrapping_mul(53).wrapping_add(7)) & 0xFF) as u8).collect();
        let x_f32: Vec<f32> = (0..(bpr_u * 2)).map(|i| ((i % 29) as f32 - 14.0) * 0.05).collect();
        let cpu_ref2 = cpu_nvfp4_rowdot(&packed_fp4, &x_f32, rows_u, bpr_u);

        let (mut d_w2, mut d_x2, mut d_lut, mut d_out2) = (0u64, 0u64, 0u64, 0u64);
        assert_eq!(cu_mem_alloc(&mut d_w2, packed_fp4.len()), 0);
        assert_eq!(cu_mem_alloc(&mut d_x2, x_f32.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_lut, 16 * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_out2, rows_u * 4), 0);
        assert_eq!(cu_htod(d_w2, packed_fp4.as_ptr() as *const c_void, packed_fp4.len()), 0);
        assert_eq!(cu_htod(d_x2, x_f32.as_ptr() as *const c_void, x_f32.len() * 4), 0);
        assert_eq!(cu_htod(d_lut, NVFP4_LUT.as_ptr() as *const c_void, 16 * 4), 0);

        let (mut p_w2, mut p_x2, mut p_lut, mut p_out2, mut p_r2, mut p_b2) = (d_w2, d_x2, d_lut, d_out2, rows, bytes_per_row);
        let mut params2: [*mut c_void; 6] = [
            &mut p_w2 as *mut u64 as *mut c_void,
            &mut p_x2 as *mut u64 as *mut c_void,
            &mut p_lut as *mut u64 as *mut c_void,
            &mut p_out2 as *mut u64 as *mut c_void,
            &mut p_r2 as *mut u32 as *mut c_void,
            &mut p_b2 as *mut u32 as *mut c_void,
        ];
        let t1 = Instant::now();
        let rc2 = cu_launch(fn2, blocks, 1, 1, threads, 1, 1, 0, std::ptr::null_mut(), params2.as_mut_ptr(), std::ptr::null_mut());
        let rc2s = cu_ctx_sync();
        let nvfp4_kernel_us = t1.elapsed().as_micros().max(1) as u64;
        let mut gpu_out2 = vec![0.0f32; rows_u];
        if rc2 == 0 && rc2s == 0 {
            assert_eq!(cu_dtoh(gpu_out2.as_mut_ptr() as *mut c_void, d_out2, rows_u * 4), 0);
        }
        cu_mem_free(d_w2); cu_mem_free(d_x2); cu_mem_free(d_lut); cu_mem_free(d_out2);
        let max_err2 = gpu_out2.iter().zip(cpu_ref2.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        let k2_ok = rc2 == 0 && rc2s == 0 && max_err2 < 1e-4;

        // ---- KERNEL 3: MIT Augment-Cache Parallel [K_self; K_peer] Scoring ----
        let fn3_name = CString::new("beast_c2c_augment_scores").unwrap();
        let mut fn3: CuFunction = std::ptr::null_mut();
        assert_eq!(cu_mod_fn(&mut fn3, module, fn3_name.as_ptr()), 0);
        let (len_self, len_peer, dim) = (512u32, 256u32, 64u32);
        let total_tok = (len_self + len_peer) as usize;
        let q_vec: Vec<f32> = (0..dim as usize).map(|d| (d as f32) * 0.031 - 1.0).collect();
        let k_self: Vec<f32> = (0..(len_self * dim) as usize).map(|i| (((i * 13) % 41) as f32 - 20.0) * 0.04).collect();
        let k_peer: Vec<f32> = (0..(len_peer * dim) as usize).map(|i| (((i * 17) % 37) as f32 - 18.0) * 0.04).collect();
        let cpu_ref3 = cpu_c2c_scores(&q_vec, &k_self, &k_peer, len_self as usize, len_peer as usize, dim as usize);

        let (mut d_q, mut d_ks, mut d_kp, mut d_sc) = (0u64, 0u64, 0u64, 0u64);
        assert_eq!(cu_mem_alloc(&mut d_q, q_vec.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_ks, k_self.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_kp, k_peer.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_sc, total_tok * 4), 0);
        assert_eq!(cu_htod(d_q, q_vec.as_ptr() as *const c_void, q_vec.len() * 4), 0);
        assert_eq!(cu_htod(d_ks, k_self.as_ptr() as *const c_void, k_self.len() * 4), 0);
        assert_eq!(cu_htod(d_kp, k_peer.as_ptr() as *const c_void, k_peer.len() * 4), 0);

        let (mut pq, mut pks, mut pkp, mut psc) = (d_q, d_ks, d_kp, d_sc);
        let (mut pls, mut plp, mut pd, mut pinv) = (len_self, len_peer, dim, 1.0f32 / (dim as f32).sqrt());
        let mut params3: [*mut c_void; 8] = [
            &mut pq as *mut u64 as *mut c_void,
            &mut pks as *mut u64 as *mut c_void,
            &mut pkp as *mut u64 as *mut c_void,
            &mut psc as *mut u64 as *mut c_void,
            &mut pls as *mut u32 as *mut c_void,
            &mut plp as *mut u32 as *mut c_void,
            &mut pd as *mut u32 as *mut c_void,
            &mut pinv as *mut f32 as *mut c_void,
        ];
        let blk3 = (len_self + len_peer).div_ceil(threads);
        let t2 = Instant::now();
        let rc3 = cu_launch(fn3, blk3, 1, 1, threads, 1, 1, 0, std::ptr::null_mut(), params3.as_mut_ptr(), std::ptr::null_mut());
        let rc3s = cu_ctx_sync();
        let c2c_scores_us = t2.elapsed().as_micros().max(1) as u64;
        let mut gpu_out3 = vec![0.0f32; total_tok];
        if rc3 == 0 && rc3s == 0 {
            assert_eq!(cu_dtoh(gpu_out3.as_mut_ptr() as *mut c_void, d_sc, total_tok * 4), 0);
        }
        cu_mem_free(d_q); cu_mem_free(d_ks); cu_mem_free(d_kp); cu_mem_free(d_sc);
        let max_err3 = gpu_out3.iter().zip(cpu_ref3.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        let k3_ok = rc3 == 0 && rc3s == 0 && max_err3 < 1e-4;

        // ---- KERNEL 4: DeepSeek/Qwen MLA 4x Low-Rank KV Compression ----
        let fn4_name = CString::new("beast_mla_latent_compress").unwrap();
        let mut fn4: CuFunction = std::ptr::null_mut();
        assert_eq!(cu_mod_fn(&mut fn4, module, fn4_name.as_ptr()), 0);
        let (mla_seq, mla_din, mla_dlat) = (256u32, 64u32, 16u32);
        let mla_h: Vec<f32> = (0..(mla_seq * mla_din) as usize).map(|i| (((i * 23) % 53) as f32 - 26.0) * 0.03).collect();
        let mla_w: Vec<f32> = (0..(mla_dlat * mla_din) as usize).map(|i| (((i * 11) % 31) as f32 - 15.0) * 0.05).collect();
        let cpu_ref4 = cpu_mla_compress(&mla_h, &mla_w, mla_seq as usize, mla_din as usize, mla_dlat as usize);

        let (mut d_mh, mut d_mw, mut d_mo) = (0u64, 0u64, 0u64);
        assert_eq!(cu_mem_alloc(&mut d_mh, mla_h.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_mw, mla_w.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_mo, cpu_ref4.len() * 4), 0);
        assert_eq!(cu_htod(d_mh, mla_h.as_ptr() as *const c_void, mla_h.len() * 4), 0);
        assert_eq!(cu_htod(d_mw, mla_w.as_ptr() as *const c_void, mla_w.len() * 4), 0);

        let (mut pmh, mut pmw, mut pmo) = (d_mh, d_mw, d_mo);
        let (mut pms, mut pmd, mut pml) = (mla_seq, mla_din, mla_dlat);
        let mut params4: [*mut c_void; 6] = [
            &mut pmh as *mut u64 as *mut c_void,
            &mut pmw as *mut u64 as *mut c_void,
            &mut pmo as *mut u64 as *mut c_void,
            &mut pms as *mut u32 as *mut c_void,
            &mut pmd as *mut u32 as *mut c_void,
            &mut pml as *mut u32 as *mut c_void,
        ];
        let blk4 = (mla_seq * mla_dlat).div_ceil(threads);
        let t3 = Instant::now();
        let rc4 = cu_launch(fn4, blk4, 1, 1, threads, 1, 1, 0, std::ptr::null_mut(), params4.as_mut_ptr(), std::ptr::null_mut());
        let rc4s = cu_ctx_sync();
        let mla_compress_us = t3.elapsed().as_micros().max(1) as u64;
        let mut gpu_out4 = vec![0.0f32; cpu_ref4.len()];
        if rc4 == 0 && rc4s == 0 {
            assert_eq!(cu_dtoh(gpu_out4.as_mut_ptr() as *mut c_void, d_mo, cpu_ref4.len() * 4), 0);
        }
        cu_mem_free(d_mh); cu_mem_free(d_mw); cu_mem_free(d_mo);
        let max_err4 = gpu_out4.iter().zip(cpu_ref4.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        let k4_ok = rc4 == 0 && rc4s == 0 && max_err4 < 1e-4;

        // ---- KERNEL 5: Sparse MoE Top-2 Gated SwiGLU FFN ----
        let fn5_name = CString::new("beast_moe_swiglu_sparse").unwrap();
        let mut fn5: CuFunction = std::ptr::null_mut();
        assert_eq!(cu_mod_fn(&mut fn5, module, fn5_name.as_ptr()), 0);
        let moe_n = 16_384u32;
        let g0: Vec<f32> = (0..moe_n as usize).map(|i| (((i * 7) % 101) as f32 - 50.0) * 0.06).collect();
        let u0: Vec<f32> = (0..moe_n as usize).map(|i| (((i * 11) % 89) as f32 - 44.0) * 0.04).collect();
        let g1: Vec<f32> = (0..moe_n as usize).map(|i| (((i * 13) % 97) as f32 - 48.0) * 0.05).collect();
        let u1: Vec<f32> = (0..moe_n as usize).map(|i| (((i * 17) % 83) as f32 - 41.0) * 0.04).collect();
        let (mut w0, mut w1) = (0.68f32, 0.32f32);
        let cpu_ref5 = cpu_moe_swiglu(&g0, &u0, &g1, &u1, w0, w1);

        let (mut dg0, mut du0, mut dg1, mut du1, mut dmo5) = (0u64, 0u64, 0u64, 0u64, 0u64);
        let nb5 = (moe_n as usize) * 4;
        assert_eq!(cu_mem_alloc(&mut dg0, nb5), 0);
        assert_eq!(cu_mem_alloc(&mut du0, nb5), 0);
        assert_eq!(cu_mem_alloc(&mut dg1, nb5), 0);
        assert_eq!(cu_mem_alloc(&mut du1, nb5), 0);
        assert_eq!(cu_mem_alloc(&mut dmo5, nb5), 0);
        assert_eq!(cu_htod(dg0, g0.as_ptr() as *const c_void, nb5), 0);
        assert_eq!(cu_htod(du0, u0.as_ptr() as *const c_void, nb5), 0);
        assert_eq!(cu_htod(dg1, g1.as_ptr() as *const c_void, nb5), 0);
        assert_eq!(cu_htod(du1, u1.as_ptr() as *const c_void, nb5), 0);

        let (mut pg0, mut pu0, mut pg1, mut pu1, mut po5, mut pn5) = (dg0, du0, dg1, du1, dmo5, moe_n);
        let mut params5: [*mut c_void; 8] = [
            &mut pg0 as *mut u64 as *mut c_void,
            &mut pu0 as *mut u64 as *mut c_void,
            &mut pg1 as *mut u64 as *mut c_void,
            &mut pu1 as *mut u64 as *mut c_void,
            &mut po5 as *mut u64 as *mut c_void,
            &mut w0 as *mut f32 as *mut c_void,
            &mut w1 as *mut f32 as *mut c_void,
            &mut pn5 as *mut u32 as *mut c_void,
        ];
        let blk5 = moe_n.div_ceil(threads);
        let t4 = Instant::now();
        let rc5 = cu_launch(fn5, blk5, 1, 1, threads, 1, 1, 0, std::ptr::null_mut(), params5.as_mut_ptr(), std::ptr::null_mut());
        let rc5s = cu_ctx_sync();
        let moe_swiglu_us = t4.elapsed().as_micros().max(1) as u64;
        let mut gpu_out5 = vec![0.0f32; moe_n as usize];
        if rc5 == 0 && rc5s == 0 {
            assert_eq!(cu_dtoh(gpu_out5.as_mut_ptr() as *mut c_void, dmo5, nb5), 0);
        }
        cu_mem_free(dg0); cu_mem_free(du0); cu_mem_free(dg1); cu_mem_free(du1); cu_mem_free(dmo5);
        let max_err5 = gpu_out5.iter().zip(cpu_ref5.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        let k5_ok = rc5 == 0 && rc5s == 0 && max_err5 < 1e-4;

        // ---- PINNED HOST-RAM <-> VRAM ASYNC KV PAGING (`W059`, 8 MiB round-trip on CUstream) ----
        let slab_bytes: usize = 8 * 1024 * 1024;
        let mut h_src: *mut c_void = std::ptr::null_mut();
        let mut h_dst: *mut c_void = std::ptr::null_mut();
        let mut d_slab: CuDevicePtr = 0;
        let mut stream: CuStream = std::ptr::null_mut();
        let mut pinned_paging_ok = false;
        let mut pinned_paging_gbps = 0.0f64;
        if cu_mem_alloc_host(&mut h_src, slab_bytes) == 0
            && cu_mem_alloc_host(&mut h_dst, slab_bytes) == 0
            && cu_mem_alloc(&mut d_slab, slab_bytes) == 0
            && cu_stream_create(&mut stream, 0) == 0
        {
            let src_slice = std::slice::from_raw_parts_mut(h_src as *mut u8, slab_bytes);
            for (i, b) in src_slice.iter_mut().enumerate() {
                *b = ((i.wrapping_mul(131) ^ (i >> 7)) & 0xFF) as u8;
            }
            let tp = Instant::now();
            let r_up = cu_htod_async(d_slab, h_src, slab_bytes, stream);
            let r_dn = cu_dtoh_async(h_dst, d_slab, slab_bytes, stream);
            let r_sy = cu_stream_sync(stream);
            let elapsed_s = tp.elapsed().as_secs_f64().max(1e-9);
            let dst_slice = std::slice::from_raw_parts(h_dst as *const u8, slab_bytes);
            pinned_paging_ok = r_up == 0 && r_dn == 0 && r_sy == 0 && src_slice == dst_slice;
            pinned_paging_gbps = ((2 * slab_bytes) as f64) / 1e9 / elapsed_s;
            cu_stream_destroy(stream);
            cu_mem_free(d_slab);
            cu_mem_free_host(h_src);
            cu_mem_free_host(h_dst);
        }

        cu_mod_unload(module);
        cu_ctx_destroy(ctx);

        let kernels_verified = u32::from(k1_ok) + u32::from(k2_ok) + u32::from(k3_ok) + u32::from(k4_ok) + u32::from(k5_ok);
        CudaGpuReport {
            available: true,
            device_name,
            compute_cap: format!("{maj}.{min}"),
            sm_count,
            rows_tested: rows,
            n_trits_per_row: n_trits as u32,
            gpu_kernel_us,
            nvfp4_kernel_us,
            c2c_scores_us,
            mla_compress_us,
            moe_swiglu_us,
            pinned_paging_ok,
            pinned_paging_gbps,
            bit_exact_cpu_match: k1_ok,
            nvfp4_exact_match: k2_ok,
            c2c_scores_match: k3_ok,
            mla_exact_match: k4_ok,
            moe_exact_match: k5_ok,
            kernels_verified,
            error_msg: if kernels_verified == 5 && pinned_paging_ok {
                None
            } else {
                Some(format!(
                    "k1={k1_ok} k2={k2_ok}({max_err2:.2e}) k3={k3_ok}({max_err3:.2e}) k4={k4_ok}({max_err4:.2e}) k5={k5_ok}({max_err5:.2e}) pinned={pinned_paging_ok}"
                ))
            },
        }
    }
}

fn err_report(msg: &str) -> CudaGpuReport {
    CudaGpuReport {
        available: false,
        device_name: "ERR".into(),
        compute_cap: "0.0".into(),
        sm_count: 0,
        rows_tested: 0,
        n_trits_per_row: 0,
        gpu_kernel_us: 0,
        nvfp4_kernel_us: 0,
        c2c_scores_us: 0,
        mla_compress_us: 0,
        moe_swiglu_us: 0,
        pinned_paging_ok: false,
        pinned_paging_gbps: 0.0,
        bit_exact_cpu_match: false,
        nvfp4_exact_match: false,
        c2c_scores_match: false,
        mla_exact_match: false,
        moe_exact_match: false,
        kernels_verified: 0,
        error_msg: Some(msg.to_string()),
    }
}
