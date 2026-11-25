//! Dynamic C-ABI bridge to `zig/beast-metal` (`beast_metal.dll` / `libbeast_metal.so`).

use beast_wire::dynlib::DynLib;
use std::path::Path;

pub const BEAST_CHIMERA_MAX_BLOCKS: usize = 16;
pub const BEAST_CHIMERA_DIM: usize = 16;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BeastChimeraBlock {
    pub kind: u8,
    pub precision: u8,
    pub active: u8,
    pub reserved: u8,
    pub alpha_rezero: f32,
    pub tau_liquid: f32,
    pub spectral_est: f32,
    pub ribosome_f2: u64,
    pub weights_q: [i8; BEAST_CHIMERA_DIM * BEAST_CHIMERA_DIM],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BeastChimeraGenome {
    pub magic: u32,
    pub generation: u32,
    pub num_blocks: u32,
    pub mutation_ops: u32,
    pub lineage_seed: u64,
    pub max_spectral_norm: f32,
    pub fitness_score: f32,
    pub blocks: [BeastChimeraBlock; BEAST_CHIMERA_MAX_BLOCKS],
}

impl Default for BeastChimeraGenome {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub struct ZigMetalLib {
    _lib: DynLib,
    pub abi_version: extern "C" fn() -> u32,
    pub philox_u64: extern "C" fn(u64, u64, u32) -> u64,
    pub pack_5trits: extern "C" fn(*const i8) -> u8,
    pub unpack_5trits: extern "C" fn(u8, *mut i8),
    pub dot_b158_ref: extern "C" fn(*const u8, *const i32, usize) -> i64,
    pub dot_b158_simd: extern "C" fn(*const u8, *const i32, usize) -> i64,
    pub dot_u1_ref: extern "C" fn(*const u8, *const i32, usize) -> i64,
    pub dot_u1_simd: extern "C" fn(*const u8, *const i32, usize) -> i64,
    pub dot_i2_ref: extern "C" fn(*const u8, *const i32, usize) -> i64,
    pub dot_i2_simd: extern "C" fn(*const u8, *const i32, usize) -> i64,
    pub dot_nvfp4_ref: extern "C" fn(*const u8, *const f32, usize) -> f32,
    pub dot_nvfp4_simd: extern "C" fn(*const u8, *const f32, usize) -> f32,
    pub c2c_augment_attn_f32: extern "C" fn(
        *const f32, *const f32, *const f32, usize,
        *const f32, *const f32, usize, usize, *mut f32,
    ) -> i32,
    pub chimera_init: extern "C" fn(*mut BeastChimeraGenome, u64),
    pub chimera_op_insert_block: extern "C" fn(*mut BeastChimeraGenome, u8, u8, u64) -> i32,
    pub chimera_op_prec_morph: extern "C" fn(*mut BeastChimeraGenome, u32, u8) -> i32,
    pub chimera_op_xenograft: extern "C" fn(*mut BeastChimeraGenome, *const BeastChimeraGenome, u32) -> i32,
    pub chimera_op_spectral_clamp: extern "C" fn(*mut BeastChimeraGenome, f32) -> f32,
    pub chimera_forward: extern "C" fn(
        *const BeastChimeraGenome, *const f32, *const f32, *mut f32, *mut f32,
    ) -> i32,
    pub self_test_all: extern "C" fn(u32) -> u64,
}

impl ZigMetalLib {
    pub fn load(path: &Path) -> Result<Self, String> {
        let lib = DynLib::open(path)?;
        macro_rules! sym {
            ($name:literal) => {
                unsafe { std::mem::transmute(lib.symbol_raw($name)?) }
            };
        }
        Ok(Self {
            abi_version: sym!("beast_metal_abi_version"),
            philox_u64: sym!("beast_philox_u64"),
            pack_5trits: sym!("beast_pack_5trits"),
            unpack_5trits: sym!("beast_unpack_5trits"),
            dot_b158_ref: sym!("beast_dot_b158_ref"),
            dot_b158_simd: sym!("beast_dot_b158_simd"),
            dot_u1_ref: sym!("beast_dot_u1_ref"),
            dot_u1_simd: sym!("beast_dot_u1_simd"),
            dot_i2_ref: sym!("beast_dot_i2_ref"),
            dot_i2_simd: sym!("beast_dot_i2_simd"),
            dot_nvfp4_ref: sym!("beast_dot_nvfp4_ref"),
            dot_nvfp4_simd: sym!("beast_dot_nvfp4_simd"),
            c2c_augment_attn_f32: sym!("beast_c2c_augment_attn_f32"),
            chimera_init: sym!("beast_chimera_init"),
            chimera_op_insert_block: sym!("beast_chimera_op_insert_block"),
            chimera_op_prec_morph: sym!("beast_chimera_op_prec_morph"),
            chimera_op_xenograft: sym!("beast_chimera_op_xenograft"),
            chimera_op_spectral_clamp: sym!("beast_chimera_op_spectral_clamp"),
            chimera_forward: sym!("beast_chimera_forward"),
            self_test_all: sym!("beast_metal_self_test_all"),
            _lib: lib,
        })
    }
}
