//! `beast-cuda`: Direct CUDA Driver API (`nvcuda.dll` / `libcuda.so.1`) PTX JIT launcher.
//! Compiles and executes `cuda/beast_blackwell.ptx` directly on NVIDIA GPUs (including RTX 5070 Ti `sm_120`)
//! with ZERO nvcc, ZERO MSVC, and bit-exact int64 parity verification against CPU Zig/Rust oracles.

use beast_wire::dynlib::DynLib;
use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::path::Path;
use std::time::Instant;

const CUDA_PTX_SRC: &str = include_str!("../../../cuda/beast_blackwell.ptx");

type CuResult = c_int;
type CuDevice = c_int;
type CuContext = *mut c_void;
type CuModule = *mut c_void;
type CuFunction = *mut c_void;
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
    pub bit_exact_cpu_match: bool,
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

pub fn run_blackwell_b158_verify(rows: u32, bytes_per_row: u32) -> CudaGpuReport {
    #[cfg(windows)]
    let lib_path = Path::new("C:\\Windows\\System32\\nvcuda.dll");
    #[cfg(not(windows))]
    let lib_path = Path::new("libcuda.so.1");

    let lib = match DynLib::open(lib_path) {
        Ok(l) => l,
        Err(e) => {
            return CudaGpuReport {
                available: false,
                device_name: "NONE".into(),
                compute_cap: "0.0".into(),
                sm_count: 0,
                rows_tested: 0,
                n_trits_per_row: 0,
                gpu_kernel_us: 0,
                bit_exact_cpu_match: false,
                error_msg: Some(e),
            };
        }
    };

    macro_rules! cu {
        ($name:literal, $ty:ty) => {
            match lib.symbol_raw($name) {
                Ok(p) => unsafe { std::mem::transmute::<*mut c_void, $ty>(p) },
                Err(e) => {
                    return CudaGpuReport {
                        available: false,
                        device_name: "SYMBOL_ERR".into(),
                        compute_cap: "0.0".into(),
                        sm_count: 0,
                        rows_tested: 0,
                        n_trits_per_row: 0,
                        gpu_kernel_us: 0,
                        bit_exact_cpu_match: false,
                        error_msg: Some(e),
                    };
                }
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
    let cu_htod = cu!("cuMemcpyHtoD_v2", unsafe extern "C" fn(CuDevicePtr, *const c_void, usize) -> CuResult);
    let cu_dtoh = cu!("cuMemcpyDtoH_v2", unsafe extern "C" fn(*mut c_void, CuDevicePtr, usize) -> CuResult);
    let cu_launch = cu!(
        "cuLaunchKernel",
        unsafe extern "C" fn(
            CuFunction, c_uint, c_uint, c_uint, c_uint, c_uint, c_uint,
            c_uint, *mut c_void, *mut *mut c_void, *mut *mut c_void,
        ) -> CuResult
    );

    unsafe {
        if cu_init(0) != 0 {
            return err_report("cuInit failed");
        }
        let mut dev: CuDevice = 0;
        if cu_device_get(&mut dev, 0) != 0 {
            return err_report("cuDeviceGet(0) failed");
        }
        let mut name_buf = [0u8; 128];
        let _ = cu_device_name(name_buf.as_mut_ptr() as *mut c_char, 128, dev);
        let nul = name_buf.iter().position(|&b| b == 0).unwrap_or(128);
        let device_name = String::from_utf8_lossy(&name_buf[..nul]).to_string();

        let (mut maj, mut min, mut sm_count) = (0, 0, 0);
        let _ = cu_device_attr(&mut maj, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR, dev);
        let _ = cu_device_attr(&mut min, CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR, dev);
        let _ = cu_device_attr(&mut sm_count, CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT, dev);

        let mut ctx: CuContext = std::ptr::null_mut();
        if cu_ctx_create(&mut ctx, 0, dev) != 0 {
            return err_report("cuCtxCreate_v2 failed");
        }

        let ptx_c = CString::new(CUDA_PTX_SRC).unwrap();
        let mut module: CuModule = std::ptr::null_mut();
        let rc_mod = cu_mod_load(&mut module, ptx_c.as_ptr() as *const c_void);
        if rc_mod != 0 {
            cu_ctx_destroy(ctx);
            return err_report(&format!("cuModuleLoadData failed rc={rc_mod}"));
        }

        let fn_name = CString::new("beast_b158_rowdot").unwrap();
        let mut func: CuFunction = std::ptr::null_mut();
        let rc_fn = cu_mod_fn(&mut func, module, fn_name.as_ptr());
        if rc_fn != 0 {
            cu_mod_unload(module);
            cu_ctx_destroy(ctx);
            return err_report(&format!("cuModuleGetFunction failed rc={rc_fn}"));
        }

        let rows_u = rows as usize;
        let bpr_u = bytes_per_row as usize;
        let n_trits = bpr_u * 5;
        let mut packed_w = vec![0u8; rows_u * bpr_u];
        for (i, byte) in packed_w.iter_mut().enumerate() {
            // Deterministic valid 5-trit code in 0..242 (3^5 - 1)
            *byte = ((i.wrapping_mul(37).wrapping_add(13)) % 243) as u8;
        }
        let x: Vec<i32> = (0..n_trits).map(|i| ((i as i32 * 19) % 127) - 63).collect();
        let cpu_ref = cpu_b158_rowdot(&packed_w, &x, rows_u, bpr_u);

        let (mut d_w, mut d_x, mut d_out) = (0u64, 0u64, 0u64);
        assert_eq!(cu_mem_alloc(&mut d_w, packed_w.len()), 0);
        assert_eq!(cu_mem_alloc(&mut d_x, x.len() * 4), 0);
        assert_eq!(cu_mem_alloc(&mut d_out, rows_u * 8), 0);

        assert_eq!(cu_htod(d_w, packed_w.as_ptr() as *const c_void, packed_w.len()), 0);
        assert_eq!(cu_htod(d_x, x.as_ptr() as *const c_void, x.len() * 4), 0);

        let mut p_w = d_w;
        let mut p_x = d_x;
        let mut p_out = d_out;
        let mut p_rows = rows;
        let mut p_bpr = bytes_per_row;
        let mut params: [*mut c_void; 5] = [
            &mut p_w as *mut u64 as *mut c_void,
            &mut p_x as *mut u64 as *mut c_void,
            &mut p_out as *mut u64 as *mut c_void,
            &mut p_rows as *mut u32 as *mut c_void,
            &mut p_bpr as *mut u32 as *mut c_void,
        ];

        let threads = 128u32;
        let blocks = rows.div_ceil(threads);
        let t0 = Instant::now();
        let rc_launch = cu_launch(
            func, blocks, 1, 1, threads, 1, 1, 0,
            std::ptr::null_mut(), params.as_mut_ptr(), std::ptr::null_mut(),
        );
        let rc_sync = cu_ctx_sync();
        let gpu_kernel_us = t0.elapsed().as_micros().max(1) as u64;

        let mut gpu_out = vec![0i64; rows_u];
        if rc_launch == 0 && rc_sync == 0 {
            assert_eq!(cu_dtoh(gpu_out.as_mut_ptr() as *mut c_void, d_out, rows_u * 8), 0);
        }

        cu_mem_free(d_w);
        cu_mem_free(d_x);
        cu_mem_free(d_out);
        cu_mod_unload(module);
        cu_ctx_destroy(ctx);

        let bit_exact = rc_launch == 0 && rc_sync == 0 && gpu_out == cpu_ref;
        CudaGpuReport {
            available: true,
            device_name,
            compute_cap: format!("{maj}.{min}"),
            sm_count,
            rows_tested: rows,
            n_trits_per_row: n_trits as u32,
            gpu_kernel_us,
            bit_exact_cpu_match: bit_exact,
            error_msg: if bit_exact { None } else { Some(format!("launch={rc_launch} sync={rc_sync} mismatch")) },
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
        bit_exact_cpu_match: false,
        error_msg: Some(msg.to_string()),
    }
}
