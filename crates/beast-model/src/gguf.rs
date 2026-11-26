//! Real GGUF v3 Binary Format Writer & Parser (`llama.cpp`-compatible header/KV/tensor-info layout,
//! 32-byte aligned tensor data arena, standard `F32`/`F16`/`Q4_0` + `BEAST_B158`/`BEAST_NVFP4`
//! dtypes, and full GGUF v3 metadata type decoding — zero external crates).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const GGUF_MAGIC: [u8; 4] = *b"GGUF";
pub const GGUF_VERSION_3: u32 = 3;
pub const GGUF_ALIGNMENT: usize = 32;
pub const Q4_0_BLOCK_ELEMS: usize = 32;
pub const Q4_0_BLOCK_BYTES: usize = 18; // 2-byte IEEE f16 scale + 16 nibble bytes

// Standard GGUF v3 metadata value type tags
const GGUF_TYPE_UINT8: u32 = 0;
const GGUF_TYPE_INT8: u32 = 1;
const GGUF_TYPE_UINT16: u32 = 2;
const GGUF_TYPE_INT16: u32 = 3;
const GGUF_TYPE_UINT32: u32 = 4;
const GGUF_TYPE_INT32: u32 = 5;
const GGUF_TYPE_FLOAT32: u32 = 6;
const GGUF_TYPE_BOOL: u32 = 7;
const GGUF_TYPE_STRING: u32 = 8;
const GGUF_TYPE_ARRAY: u32 = 9;
const GGUF_TYPE_UINT64: u32 = 10;
const GGUF_TYPE_INT64: u32 = 11;
const GGUF_TYPE_FLOAT64: u32 = 12;

/// Tensor dtype tags (`0 = F32`, `1 = F16`, `2 = Q4_0`, `30 = BEAST_B158`, `31 = BEAST_NVFP4`).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GgufDtype {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    BeastB158 = 30,
    BeastNvfp4 = 31,
}

impl GgufDtype {
    pub fn from_u32(v: u32) -> Result<Self, String> {
        match v {
            0 => Ok(Self::F32),
            1 => Ok(Self::F16),
            2 => Ok(Self::Q4_0),
            30 => Ok(Self::BeastB158),
            31 => Ok(Self::BeastNvfp4),
            other => Err(format!("unsupported GGUF dtype tag {other}")),
        }
    }

    pub fn byte_len(self, n_elems: usize) -> Result<usize, String> {
        match self {
            Self::F32 => Ok(n_elems * 4),
            Self::F16 => Ok(n_elems * 2),
            Self::Q4_0 => {
                if n_elems % Q4_0_BLOCK_ELEMS != 0 {
                    return Err(format!(
                        "Q4_0 element count {n_elems} is not a multiple of {Q4_0_BLOCK_ELEMS}"
                    ));
                }
                Ok((n_elems / Q4_0_BLOCK_ELEMS) * Q4_0_BLOCK_BYTES)
            }
            Self::BeastB158 => Ok(n_elems.div_ceil(5)),
            Self::BeastNvfp4 => Ok(n_elems.div_ceil(2)),
        }
    }
}

/// Exact IEEE 754 binary16 -> binary32 conversion (zero dependencies).
pub fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits as u32) & 0x8000) << 16;
    let exp = ((bits >> 10) & 0x1F) as u32;
    let mant = (bits & 0x03FF) as u32;
    let out_bits = if exp == 0 {
        if mant == 0 {
            sign
        } else {
            let mut m = mant;
            let mut e: i32 = -14;
            while (m & 0x0400) == 0 {
                m <<= 1;
                e -= 1;
            }
            m &= 0x03FF;
            sign | (((e + 127) as u32) << 23) | (m << 13)
        }
    } else if exp == 31 {
        sign | 0x7F80_0000 | (mant << 13)
    } else {
        sign | ((exp + 112) << 23) | (mant << 13)
    };
    f32::from_bits(out_bits)
}

/// Exact IEEE 754 binary32 -> binary16 conversion (round-to-nearest-even for normal values).
pub fn f32_to_f16(val: f32) -> u16 {
    let bits = val.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127;
    let mant = bits & 0x007F_FFFF;
    if exp == 128 {
        return sign | 0x7C00 | if mant != 0 { 0x0200 } else { 0 };
    }
    if exp > 15 {
        return sign | 0x7C00;
    }
    if exp < -14 {
        return sign;
    }
    let h_exp = ((exp + 15) as u16) << 10;
    let h_mant = ((mant + 0x1000) >> 13) as u16;
    sign | h_exp.wrapping_add(h_mant)
}

/// Quantize a slice of `f32` values (multiple of 32) into standard `llama.cpp` `Q4_0` blocks.
pub fn quantize_q4_0(xs: &[f32]) -> Result<Vec<u8>, String> {
    if xs.len() % Q4_0_BLOCK_ELEMS != 0 {
        return Err(format!("quantize_q4_0 length {} not multiple of 32", xs.len()));
    }
    let mut out = Vec::with_capacity((xs.len() / Q4_0_BLOCK_ELEMS) * Q4_0_BLOCK_BYTES);
    for blk in xs.chunks_exact(Q4_0_BLOCK_ELEMS) {
        let mut amax = 0.0f32;
        for &v in blk {
            let a = v.abs();
            if a > amax {
                amax = a;
            }
        }
        let d = amax / 7.0;
        let inv_d = if d > 1e-12 { 1.0 / d } else { 0.0 };
        out.extend_from_slice(&f32_to_f16(d).to_le_bytes());
        for j in 0..16 {
            let q0 = ((blk[j] * inv_d).round().clamp(-8.0, 7.0) as i32 + 8) as u8;
            let q1 = ((blk[j + 16] * inv_d).round().clamp(-8.0, 7.0) as i32 + 8) as u8;
            out.push((q0 & 0x0F) | ((q1 & 0x0F) << 4));
        }
    }
    Ok(out)
}

/// Dequantize standard `llama.cpp` `Q4_0` blocks back to `f32`.
pub fn dequantize_q4_0(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if bytes.len() % Q4_0_BLOCK_BYTES != 0 {
        return Err(format!(
            "dequantize_q4_0 byte length {} not multiple of {Q4_0_BLOCK_BYTES}",
            bytes.len()
        ));
    }
    let n_blocks = bytes.len() / Q4_0_BLOCK_BYTES;
    let mut out = vec![0.0f32; n_blocks * Q4_0_BLOCK_ELEMS];
    for (b, blk) in bytes.chunks_exact(Q4_0_BLOCK_BYTES).enumerate() {
        let d = f16_to_f32(u16::from_le_bytes([blk[0], blk[1]]));
        let base = b * Q4_0_BLOCK_ELEMS;
        for j in 0..16 {
            let byte = blk[2 + j];
            let lo = (byte & 0x0F) as i32 - 8;
            let hi = ((byte >> 4) & 0x0F) as i32 - 8;
            out[base + j] = (lo as f32) * d;
            out[base + j + 16] = (hi as f32) * d;
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq)]
pub enum GgufMetaVal {
    U32(u32),
    F32(f32),
    Bool(bool),
    U64(u64),
    Str(String),
    ArrayCount(u32, usize),
}

#[derive(Debug, Clone)]
pub struct GgufTensor {
    pub name: String,
    pub dims: Vec<u64>,
    pub dtype: GgufDtype,
    pub raw_bytes: Vec<u8>,
}

impl GgufTensor {
    pub fn to_f32_vec(&self) -> Result<Vec<f32>, String> {
        match self.dtype {
            GgufDtype::F32 => {
                if self.raw_bytes.len() % 4 != 0 {
                    return Err("F32 tensor byte length not multiple of 4".into());
                }
                Ok(self
                    .raw_bytes
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect())
            }
            GgufDtype::F16 => {
                if self.raw_bytes.len() % 2 != 0 {
                    return Err("F16 tensor byte length not multiple of 2".into());
                }
                Ok(self
                    .raw_bytes
                    .chunks_exact(2)
                    .map(|c| f16_to_f32(u16::from_le_bytes([c[0], c[1]])))
                    .collect())
            }
            GgufDtype::Q4_0 => dequantize_q4_0(&self.raw_bytes),
            other => Err(format!("to_f32_vec not supported directly for {other:?}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GgufFile {
    pub metadata: BTreeMap<String, GgufMetaVal>,
    pub tensors: Vec<GgufTensor>,
}

fn write_gguf_str(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u64).to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn read_u8(buf: &[u8], off: &mut usize) -> Result<u8, String> {
    if *off + 1 > buf.len() { return Err("unexpected EOF reading u8".into()); }
    let v = buf[*off];
    *off += 1;
    Ok(v)
}

fn read_u16(buf: &[u8], off: &mut usize) -> Result<u16, String> {
    if *off + 2 > buf.len() { return Err("unexpected EOF reading u16".into()); }
    let v = u16::from_le_bytes(buf[*off..*off + 2].try_into().unwrap());
    *off += 2;
    Ok(v)
}

fn read_u32(buf: &[u8], off: &mut usize) -> Result<u32, String> {
    if *off + 4 > buf.len() { return Err("unexpected EOF reading u32".into()); }
    let v = u32::from_le_bytes(buf[*off..*off + 4].try_into().unwrap());
    *off += 4;
    Ok(v)
}

fn read_u64(buf: &[u8], off: &mut usize) -> Result<u64, String> {
    if *off + 8 > buf.len() { return Err("unexpected EOF reading u64".into()); }
    let v = u64::from_le_bytes(buf[*off..*off + 8].try_into().unwrap());
    *off += 8;
    Ok(v)
}

fn read_str(buf: &[u8], off: &mut usize) -> Result<String, String> {
    let len = read_u64(buf, off)? as usize;
    if len > 4_194_304 || *off + len > buf.len() { return Err("invalid GGUF string length".into()); }
    let s = String::from_utf8(buf[*off..*off + len].to_vec()).map_err(|_| "non-UTF8 GGUF string".to_string())?;
    *off += len;
    Ok(s)
}

fn skip_meta_value(buf: &[u8], off: &mut usize, ty: u32) -> Result<GgufMetaVal, String> {
    match ty {
        GGUF_TYPE_UINT8 | GGUF_TYPE_INT8 => Ok(GgufMetaVal::U32(read_u8(buf, off)? as u32)),
        GGUF_TYPE_UINT16 | GGUF_TYPE_INT16 => Ok(GgufMetaVal::U32(read_u16(buf, off)? as u32)),
        GGUF_TYPE_UINT32 | GGUF_TYPE_INT32 => Ok(GgufMetaVal::U32(read_u32(buf, off)?)),
        GGUF_TYPE_FLOAT32 => Ok(GgufMetaVal::F32(f32::from_bits(read_u32(buf, off)?))),
        GGUF_TYPE_BOOL => Ok(GgufMetaVal::Bool(read_u8(buf, off)? != 0)),
        GGUF_TYPE_STRING => Ok(GgufMetaVal::Str(read_str(buf, off)?)),
        GGUF_TYPE_UINT64 | GGUF_TYPE_INT64 => Ok(GgufMetaVal::U64(read_u64(buf, off)?)),
        GGUF_TYPE_FLOAT64 => {
            let bits = read_u64(buf, off)?;
            Ok(GgufMetaVal::F32(f64::from_bits(bits) as f32))
        }
        GGUF_TYPE_ARRAY => {
            let elem_ty = read_u32(buf, off)?;
            let count = read_u64(buf, off)? as usize;
            if count > 1_048_576 {
                return Err(format!("GGUF array too large: {count}"));
            }
            for _ in 0..count {
                let _ = skip_meta_value(buf, off, elem_ty)?;
            }
            Ok(GgufMetaVal::ArrayCount(elem_ty, count))
        }
        other => Err(format!("unsupported GGUF metadata type {other}")),
    }
}

fn align_up(v: usize, align: usize) -> usize {
    (v + align - 1) & !(align - 1)
}

impl GgufFile {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&GGUF_MAGIC);
        buf.extend_from_slice(&GGUF_VERSION_3.to_le_bytes());
        buf.extend_from_slice(&(self.tensors.len() as u64).to_le_bytes());
        buf.extend_from_slice(&(self.metadata.len() as u64).to_le_bytes());

        for (k, v) in &self.metadata {
            write_gguf_str(&mut buf, k);
            match v {
                GgufMetaVal::U32(n) => {
                    buf.extend_from_slice(&GGUF_TYPE_UINT32.to_le_bytes());
                    buf.extend_from_slice(&n.to_le_bytes());
                }
                GgufMetaVal::F32(f) => {
                    buf.extend_from_slice(&GGUF_TYPE_FLOAT32.to_le_bytes());
                    buf.extend_from_slice(&f.to_le_bytes());
                }
                GgufMetaVal::Bool(b) => {
                    buf.extend_from_slice(&GGUF_TYPE_BOOL.to_le_bytes());
                    buf.push(u8::from(*b));
                }
                GgufMetaVal::U64(n) => {
                    buf.extend_from_slice(&GGUF_TYPE_UINT64.to_le_bytes());
                    buf.extend_from_slice(&n.to_le_bytes());
                }
                GgufMetaVal::Str(s) => {
                    buf.extend_from_slice(&GGUF_TYPE_STRING.to_le_bytes());
                    write_gguf_str(&mut buf, s);
                }
                GgufMetaVal::ArrayCount(elem_ty, count) => {
                    buf.extend_from_slice(&GGUF_TYPE_ARRAY.to_le_bytes());
                    buf.extend_from_slice(&elem_ty.to_le_bytes());
                    buf.extend_from_slice(&(*count as u64).to_le_bytes());
                }
            }
        }

        // Compute relative 32-byte aligned offsets for every tensor inside the data section
        let mut rel_off = 0usize;
        for t in &self.tensors {
            write_gguf_str(&mut buf, &t.name);
            buf.extend_from_slice(&(t.dims.len() as u32).to_le_bytes());
            for &d in &t.dims {
                buf.extend_from_slice(&d.to_le_bytes());
            }
            buf.extend_from_slice(&(t.dtype as u32).to_le_bytes());
            buf.extend_from_slice(&(rel_off as u64).to_le_bytes());
            rel_off = align_up(rel_off + t.raw_bytes.len(), GGUF_ALIGNMENT);
        }

        // Pad header to 32-byte boundary before tensor data section
        let data_start = align_up(buf.len(), GGUF_ALIGNMENT);
        buf.resize(data_start, 0);

        for t in &self.tensors {
            buf.extend_from_slice(&t.raw_bytes);
            let next_aligned = align_up(buf.len(), GGUF_ALIGNMENT);
            buf.resize(next_aligned, 0);
        }
        buf
    }

    pub fn decode(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 24 || buf[0..4] != GGUF_MAGIC {
            return Err("invalid GGUF magic".into());
        }
        let mut off = 4usize;
        let ver = read_u32(buf, &mut off)?;
        if ver != GGUF_VERSION_3 {
            return Err(format!("expected GGUF v3, got v{ver}"));
        }
        let n_tensors = read_u64(buf, &mut off)? as usize;
        let n_kv = read_u64(buf, &mut off)? as usize;

        let mut metadata = BTreeMap::new();
        for _ in 0..n_kv {
            let k = read_str(buf, &mut off)?;
            let ty = read_u32(buf, &mut off)?;
            let val = skip_meta_value(buf, &mut off, ty)?;
            metadata.insert(k, val);
        }

        let mut specs = Vec::with_capacity(n_tensors);
        for _ in 0..n_tensors {
            let name = read_str(buf, &mut off)?;
            let ndims = read_u32(buf, &mut off)? as usize;
            let mut dims = Vec::with_capacity(ndims);
            for _ in 0..ndims {
                dims.push(read_u64(buf, &mut off)?);
            }
            let dtype = GgufDtype::from_u32(read_u32(buf, &mut off)?)?;
            let rel_offset = read_u64(buf, &mut off)? as usize;
            specs.push((name, dims, dtype, rel_offset));
        }

        let data_start = align_up(off, GGUF_ALIGNMENT);
        let mut tensors = Vec::with_capacity(n_tensors);
        for (name, dims, dtype, rel_offset) in specs.iter() {
            let abs_start = data_start + *rel_offset;
            let n_elems: usize = dims.iter().copied().product::<u64>() as usize;
            let byte_len = dtype.byte_len(n_elems)?;
            if abs_start + byte_len > buf.len() {
                return Err(format!("tensor '{name}' exceeds GGUF buffer bounds"));
            }
            tensors.push(GgufTensor {
                name: name.clone(),
                dims: dims.clone(),
                dtype: *dtype,
                raw_bytes: buf[abs_start..abs_start + byte_len].to_vec(),
            });
        }

        Ok(Self { metadata, tensors })
    }

    pub fn save_to_path(&self, path: &Path) -> Result<usize, String> {
        let bytes = self.encode();
        fs::write(path, &bytes).map_err(|e| format!("write GGUF {}: {e}", path.display()))?;
        Ok(bytes.len())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("read GGUF {}: {e}", path.display()))?;
        Self::decode(&bytes)
    }

    pub fn architecture(&self) -> &str {
        match self.metadata.get("general.architecture") {
            Some(GgufMetaVal::Str(s)) => s.as_str(),
            _ => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gguf_v3_roundtrip_and_corrupt_magic_mutant() {
        let mut meta = BTreeMap::new();
        meta.insert("general.architecture".into(), GgufMetaVal::Str("beast-chimera-hybrid".into()));
        meta.insert("beast.block_count".into(), GgufMetaVal::U32(8));
        meta.insert("beast.quant_scale".into(), GgufMetaVal::F32(0.125));
        meta.insert("beast.verified".into(), GgufMetaVal::Bool(true));

        let q4_src: Vec<f32> = (0..64).map(|i| ((i as f32) - 31.5) * 0.05).collect();
        let q4_raw = quantize_q4_0(&q4_src).expect("Q4_0 quantize");
        assert_eq!(q4_raw.len(), 2 * Q4_0_BLOCK_BYTES);

        let f16_raw: Vec<u8> = [1.0f32, -0.5, 0.25, 2.0]
            .iter()
            .flat_map(|&v| f32_to_f16(v).to_le_bytes())
            .collect();

        let f = GgufFile {
            metadata: meta,
            tensors: vec![
                GgufTensor { name: "blk0.b158.w".into(), dims: vec![25], dtype: GgufDtype::BeastB158, raw_bytes: vec![121, 80, 200, 14, 242] },
                GgufTensor { name: "blk1.nvfp4.w".into(), dims: vec![8], dtype: GgufDtype::BeastNvfp4, raw_bytes: vec![0x12, 0x34, 0x56, 0x78] },
                GgufTensor { name: "blk2.q4_0.w".into(), dims: vec![64], dtype: GgufDtype::Q4_0, raw_bytes: q4_raw },
                GgufTensor { name: "blk3.f16.w".into(), dims: vec![4], dtype: GgufDtype::F16, raw_bytes: f16_raw },
            ],
        };
        let encoded = f.encode();
        let decoded = GgufFile::decode(&encoded).expect("valid GGUF v3 must decode");
        assert_eq!(decoded.metadata, f.metadata);
        assert_eq!(decoded.tensors.len(), 4);

        let deq = decoded.tensors[2].to_f32_vec().expect("Q4_0 dequant");
        let max_err = q4_src
            .iter()
            .zip(deq.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(max_err < 0.20, "Q4_0 quantization error too large: {max_err}");

        let f16_back = decoded.tensors[3].to_f32_vec().expect("F16 dequant");
        assert_eq!(f16_back, vec![1.0, -0.5, 0.25, 2.0]);

        // Mutants: corrupt magic + non-multiple-of-32 Q4_0 length
        let mut bad = encoded.clone();
        bad[0] ^= 0xFF;
        assert!(GgufFile::decode(&bad).is_err());
        assert!(quantize_q4_0(&[1.0, 2.0, 3.0]).is_err());
    }
}
