//! Merkle-Hashed Multi-Language AST/Symbol Repo-Map (`crates/beast-agent/src/repo_map.rs`).
//! Scans `.rs`, `.zig`, `.c`, `.h`, `.rb`, `.lean`, `.ptx` source trees (excluding `target/`, `vendor/`,
//! `.git/`, `bin/`, `results/`), extracts exported/public symbols, computes per-file SHA-256 + deterministic
//! sorted Merkle root, and emits a NonHuman-budgeted Repo Map for immediate `.bcg` CacheGraft ingestion.

use beast_wire::sha256::{hex_encode, sha256};
use std::fs;
use std::path::{Path, PathBuf};

const TRACKED_EXTS: &[&str] = &["rs", "zig", "c", "h", "rb", "lean", "ptx"];
const SKIP_DIRS: &[&str] = &["target", "vendor", ".git", "bin", "results", ".lake", "_site", "dist"];

#[derive(Debug, Clone)]
pub struct FileSymbolEntry {
    pub rel_path: String,
    pub sha256_hex: String,
    pub line_count: usize,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MerkleRepoMap {
    pub files_indexed: usize,
    pub total_symbols: usize,
    pub merkle_root: [u8; 32],
    pub merkle_root_hex: String,
    pub entries: Vec<FileSymbolEntry>,
    pub compact_prompt: String,
}

fn collect_source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return; };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if let Some(name) = p.file_name().and_then(|n| n.to_str())
                && (SKIP_DIRS.contains(&name) || name.starts_with('.'))
            {
                continue;
            }
            collect_source_files(&p, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str())
            && TRACKED_EXTS.contains(&ext)
        {
            out.push(p);
        }
    }
}

fn extract_symbols(ext: &str, text: &str) -> Vec<String> {
    let mut syms = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        let sym = match ext {
            "rs" => {
                if let Some(rest) = line
                    .strip_prefix("pub fn ")
                    .or_else(|| line.strip_prefix("pub struct "))
                    .or_else(|| line.strip_prefix("pub enum "))
                {
                    rest.split(['(', '<', '{', ' ', ';']).next()
                } else {
                    None
                }
            }
            "zig" => {
                if let Some(rest) = line
                    .strip_prefix("export fn ")
                    .or_else(|| line.strip_prefix("pub fn "))
                {
                    rest.split(['(', ' ']).next()
                } else {
                    None
                }
            }
            "c" | "h" => {
                if line.contains("_EXPORT ") || line.starts_with("IRREP_API ") || line.starts_with("MOONLAB_API ") {
                    line.split('(').next().and_then(|lhs| lhs.split_whitespace().last()).map(|s| s.trim_start_matches('*'))
                } else {
                    None
                }
            }
            "lean" => {
                if let Some(rest) = line
                    .strip_prefix("public theorem ")
                    .or_else(|| line.strip_prefix("theorem "))
                {
                    rest.split([' ', ':', '(']).next()
                } else {
                    None
                }
            }
            "rb" => {
                if let Some(rest) = line.strip_prefix("def ") {
                    rest.split(['(', ' ']).next()
                } else {
                    None
                }
            }
            "ptx" => {
                if let Some(rest) = line.strip_prefix(".visible .entry ") {
                    rest.split(['(', ' ']).next()
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(s) = sym
            && !s.is_empty()
            && syms.len() < 48
        {
            syms.push(s.to_string());
        }
    }
    syms
}

/// Binary Merkle root over sorted leaf SHA-256 hashes (deterministic, tamper-evident).
pub fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return sha256(b"BEAST-EMPTY-MERKLE");
    }
    let mut level: Vec<[u8; 32]> = leaves.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let mut buf = [0u8; 64];
            buf[..32].copy_from_slice(&pair[0]);
            if pair.len() == 2 {
                buf[32..].copy_from_slice(&pair[1]);
            } else {
                buf[32..].copy_from_slice(&pair[0]);
            }
            next.push(sha256(&buf));
        }
        level = next;
    }
    level[0]
}

pub fn build_merkle_repo_map(repo_root: &Path, byte_budget: usize) -> Result<MerkleRepoMap, String> {
    let mut paths = Vec::new();
    collect_source_files(repo_root, &mut paths);
    paths.sort();

    let mut entries = Vec::with_capacity(paths.len());
    let mut leaf_hashes = Vec::with_capacity(paths.len());
    let mut total_symbols = 0usize;

    for path in paths {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let raw = fs::read(&path).map_err(|e| format!("read {rel}: {e}"))?;
        let digest = sha256(&raw);
        let text = String::from_utf8_lossy(&raw);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let symbols = extract_symbols(ext, &text);
        total_symbols += symbols.len();

        // Bind both relative path and content hash into the leaf so renames also change Merkle root!
        let mut leaf_preimage = Vec::with_capacity(rel.len() + 33);
        leaf_preimage.extend_from_slice(rel.as_bytes());
        leaf_preimage.push(0);
        leaf_preimage.extend_from_slice(&digest);
        leaf_hashes.push(sha256(&leaf_preimage));

        entries.push(FileSymbolEntry {
            rel_path: rel,
            sha256_hex: hex_encode(&digest),
            line_count: text.lines().count(),
            symbols,
        });
    }

    let merkle_root = compute_merkle_root(&leaf_hashes);
    let merkle_root_hex = hex_encode(&merkle_root);

    let mut compact_prompt = format!(
        "# BEAST MERKLE REPO-MAP root={} files={} syms={}\n",
        &merkle_root_hex[..16],
        entries.len(),
        total_symbols
    );
    for e in &entries {
        if e.symbols.is_empty() {
            continue;
        }
        let row = format!(
            "{} [{}L]: {}\n",
            e.rel_path,
            e.line_count,
            e.symbols.iter().take(10).cloned().collect::<Vec<_>>().join(", ")
        );
        if compact_prompt.len() + row.len() > byte_budget {
            break;
        }
        compact_prompt.push_str(&row);
    }

    Ok(MerkleRepoMap {
        files_indexed: entries.len(),
        total_symbols,
        merkle_root,
        merkle_root_hex,
        entries,
        compact_prompt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merkle_root_is_deterministic_and_tamper_sensitive() {
        let a = sha256(b"file-a");
        let b = sha256(b"file-b");
        let c = sha256(b"file-c");
        let r1 = compute_merkle_root(&[a, b, c]);
        let r2 = compute_merkle_root(&[a, b, c]);
        assert_eq!(r1, r2);
        let mut c_bad = c;
        c_bad[0] ^= 0x01;
        let r_bad = compute_merkle_root(&[a, b, c_bad]);
        assert_ne!(r1, r_bad);
    }
}
