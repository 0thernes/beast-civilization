//! Persistent `.bcg` Binary CacheGraft Repository Indexer (MIT CacheGraft pattern).
//! Scans real local repository source files, encodes chunks into fused `(K, V)` tensors,
//! writes a SHA-256-authenticated binary `.bcg` bank to disk, and rejects tampered banks.

use crate::{KvCache, ModelFamily, SwatCrossFamilyMapper};
use beast_wire::sha256::sha256;
use std::fs;
use std::path::Path;

pub const BCG_MAGIC: [u8; 4] = *b"BCGB";

#[derive(Debug, Clone)]
pub struct DiskCacheGraftBank {
    pub repo_label: String,
    pub chunks_indexed: usize,
    pub kv: KvCache,
    pub sha256_digest: [u8; 32],
}

/// Deterministic content-addressed token embedding from raw source chunk bytes -> `src_dim` vector.
fn embed_source_chunk(chunk: &[u8], dim: usize) -> (Vec<f32>, Vec<f32>) {
    let h0 = sha256(chunk);
    let mut k = vec![0.0f32; dim];
    let mut v = vec![0.0f32; dim];
    for d in 0..dim {
        let b = h0[d % 32];
        let b2 = h0[(d * 7 + 13) % 32];
        // Also fold byte n-gram statistics so different source files have distinct geometry
        let gram = chunk.get(d % chunk.len().max(1)).copied().unwrap_or(0);
        k[d] = (((b ^ gram) as f32) / 127.5) - 1.0;
        v[d] = (((b2.wrapping_add(gram)) as f32) / 127.5) - 1.0;
    }
    (k, v)
}

/// Index real source files under `repo_root` into a fused `.bcg` CacheGraft KV bank via a sharer->receiver mapper.
pub fn build_repo_cachegraft_bank(
    repo_root: &Path,
    rel_files: &[&str],
    mapper: &SwatCrossFamilyMapper,
    max_compact_tokens: usize,
) -> Result<DiskCacheGraftBank, String> {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut chunks = 0usize;

    for rel in rel_files {
        let path = repo_root.join(rel);
        let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        for slice in bytes.chunks(512).take(4) {
            let (k, v) = embed_source_chunk(slice, mapper.src_dim);
            keys.extend_from_slice(&k);
            values.extend_from_slice(&v);
            chunks += 1;
        }
    }
    if chunks == 0 {
        return Err("no source chunks indexed".into());
    }

    let sharer_kv = KvCache {
        family: mapper.src_family,
        seq_len: chunks,
        dim: mapper.src_dim,
        keys,
        values,
    };
    let fused = mapper.project_and_compact(&sharer_kv, max_compact_tokens);
    let digest = sha256(&fused.to_bytes());
    Ok(DiskCacheGraftBank {
        repo_label: repo_root.file_name().and_then(|s| s.to_str()).unwrap_or("repo").to_string(),
        chunks_indexed: chunks,
        kv: fused,
        sha256_digest: digest,
    })
}

pub fn save_bcg(bank: &DiskCacheGraftBank, out_path: &Path) -> Result<usize, String> {
    let payload = bank.kv.to_bytes();
    let mut buf = Vec::with_capacity(4 + 4 + 4 + 4 + 32 + payload.len());
    buf.extend_from_slice(&BCG_MAGIC);
    buf.extend_from_slice(&(bank.chunks_indexed as u32).to_le_bytes());
    buf.extend_from_slice(&(bank.kv.seq_len as u32).to_le_bytes());
    buf.extend_from_slice(&(bank.kv.dim as u32).to_le_bytes());
    buf.extend_from_slice(&bank.sha256_digest);
    buf.extend_from_slice(&payload);
    fs::write(out_path, &buf).map_err(|e| format!("write {}: {e}", out_path.display()))?;
    Ok(buf.len())
}

pub fn load_bcg_bytes(buf: &[u8], label: &str) -> Result<DiskCacheGraftBank, String> {
    if buf.len() < 48 || buf[0..4] != BCG_MAGIC {
        return Err("invalid .bcg magic or truncated header".into());
    }
    let chunks_indexed = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    let seq_len = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]) as usize;
    let dim = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as usize;
    if seq_len > 1_000_000 || dim > 65_536 {
        return Err(format!(".bcg dimensions exceed safety caps: seq_len={seq_len} dim={dim}"));
    }
    let mut expected_sha = [0u8; 32];
    expected_sha.copy_from_slice(&buf[16..48]);

    let payload = &buf[48..];
    let n = seq_len.checked_mul(dim).ok_or_else(|| ".bcg seq_len*dim overflow".to_string())?;
    let expected_bytes = n.checked_mul(8).ok_or_else(|| ".bcg byte count overflow".to_string())?;
    if payload.len() != expected_bytes {
        return Err("payload byte count mismatch in .bcg".into());
    }
    if sha256(payload) != expected_sha {
        return Err("MUTANT CAUGHT: .bcg SHA-256 integrity failure!".into());
    }
    let half = n * 4;
    let keys: Vec<f32> = payload[..half]
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let values: Vec<f32> = payload[half..]
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    Ok(DiskCacheGraftBank {
        repo_label: label.to_string(),
        chunks_indexed,
        kv: KvCache { family: ModelFamily::TemRust, seq_len, dim, keys, values },
        sha256_digest: expected_sha,
    })
}

pub fn load_bcg(path: &Path) -> Result<DiskCacheGraftBank, String> {
    let buf = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let label = path.file_name().and_then(|s| s.to_str()).unwrap_or("bank");
    load_bcg_bytes(&buf, label)
}
