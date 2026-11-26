//! Real GGUF v3 Binary Format Writer & Parser (llama.cpp-compatible header/KV/tensor-info layout,
//! 32-byte aligned tensor data arena, plus zero-dependency verification).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const GGUF_MAGIC: [u8; 4] = *b"GGUF";
pub const GGUF_VERSION_3: u32 = 3;
pub const GGUF_ALIGNMENT: usize = 32;

// Standard GGUF metadata value type tags
const GGUF_TYPE_UINT32: u32 = 4;
const GGUF_TYPE_STRING: u32 = 8;

// Tensor dtype tags (0 = F32, 30 = custom BEAST_B158 5-trit/byte, 31 = custom BEAST_NVFP4)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GgufDtype {
    F32 = 0,
    BeastB158 = 30,
    BeastNvfp4 = 31,
}

impl GgufDtype {
    pub fn from_u32(v: u32) -> Result<Self, String> {
        match v {
            0 => Ok(Self::F32),
            30 => Ok(Self::BeastB158),
            31 => Ok(Self::BeastNvfp4),
            other => Err(format!("unsupported GGUF dtype tag {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GgufMetaVal {
    U32(u32),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct GgufTensor {
    pub name: String,
    pub dims: Vec<u64>,
    pub dtype: GgufDtype,
    pub raw_bytes: Vec<u8>,
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
    if len > 1_048_576 || *off + len > buf.len() { return Err("invalid GGUF string length".into()); }
    let s = String::from_utf8(buf[*off..*off + len].to_vec()).map_err(|_| "non-UTF8 GGUF string".to_string())?;
    *off += len;
    Ok(s)
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
                GgufMetaVal::Str(s) => {
                    buf.extend_from_slice(&GGUF_TYPE_STRING.to_le_bytes());
                    write_gguf_str(&mut buf, s);
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
            let val = match ty {
                GGUF_TYPE_UINT32 => GgufMetaVal::U32(read_u32(buf, &mut off)?),
                GGUF_TYPE_STRING => GgufMetaVal::Str(read_str(buf, &mut off)?),
                other => return Err(format!("unsupported GGUF metadata type {other}")),
            };
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
        for (idx, (name, dims, dtype, rel_offset)) in specs.iter().enumerate() {
            let abs_start = data_start + *rel_offset;
            let n_elems: usize = dims.iter().copied().product::<u64>() as usize;
            let byte_len = match dtype {
                GgufDtype::F32 => n_elems * 4,
                GgufDtype::BeastB158 => n_elems.div_ceil(5),
                GgufDtype::BeastNvfp4 => n_elems.div_ceil(2),
            };
            let _ = idx;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gguf_v3_roundtrip_and_corrupt_magic_mutant() {
        let mut meta = BTreeMap::new();
        meta.insert("general.architecture".into(), GgufMetaVal::Str("beast-chimera-hybrid".into()));
        meta.insert("beast.block_count".into(), GgufMetaVal::U32(8));
        let f = GgufFile {
            metadata: meta,
            tensors: vec![
                GgufTensor { name: "blk0.b158.w".into(), dims: vec![25], dtype: GgufDtype::BeastB158, raw_bytes: vec![121, 80, 200, 14, 242] },
                GgufTensor { name: "blk1.nvfp4.w".into(), dims: vec![8], dtype: GgufDtype::BeastNvfp4, raw_bytes: vec![0x12, 0x34, 0x56, 0x78] },
            ],
        };
        let encoded = f.encode();
        let decoded = GgufFile::decode(&encoded).expect("valid GGUF v3 must decode");
        assert_eq!(decoded.metadata, f.metadata);
        assert_eq!(decoded.tensors.len(), 2);
        assert_eq!(decoded.tensors[0].raw_bytes, f.tensors[0].raw_bytes);

        let mut bad = encoded.clone();
        bad[0] ^= 0xFF;
        assert!(GgufFile::decode(&bad).is_err());
    }
}
