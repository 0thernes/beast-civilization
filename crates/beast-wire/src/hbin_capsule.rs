//! Planaria-inspired Self-Healing Holographic Colony Capsule (`.hbin`-style):
//! - Splits any colony payload (YARV bytecode, Chimera genome, CacheGraft metadata) into `K = 4`
//!   data shards + `K = 4` Cauchy MDS parity shards over `GF(2^8)` (irreducible poly `x^8+x^4+x^3+x^2+1`, `0x11D`).
//! - Authenticates every shard against a domain-separated SHA-256 Merkle root (`0x00` leaf / `0x01` node).
//! - Survives **50% total shard annihilation** (any 4 of 8 shards recover 100.000% bit-exact original payload!)
//!   AND quarantines **forged Byzantine Trojan shards** BEFORE touching the GF(2^8) Cauchy decoder!

use crate::sha256::sha256;

const GF_POLY: u16 = 0x11D; // Primitive irreducible polynomial in F_2[x]

fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut res = 0u8;
    for _ in 0..8 {
        if (b & 1) != 0 {
            res ^= a;
        }
        let hi = (a & 0x80) != 0;
        a <<= 1;
        if hi {
            a ^= (GF_POLY & 0xFF) as u8;
        }
        b >>= 1;
    }
    res
}

fn gf_pow(mut base: u8, mut exp: u32) -> u8 {
    let mut res = 1u8;
    while exp > 0 {
        if (exp & 1) != 0 {
            res = gf_mul(res, base);
        }
        base = gf_mul(base, base);
        exp >>= 1;
    }
    res
}

fn gf_inv(a: u8) -> u8 {
    assert_ne!(a, 0, "cannot invert 0 in GF(2^8)");
    gf_pow(a, 254)
}

/// Systematic 8x4 Cauchy MDS encoding matrix over GF(2^8):
/// Rows 0..4 are identity; rows 4..8 are Cauchy `1 / (x_i ^ y_j)` with disjoint x={1..4}, y={5..8}.
fn encode_row(shard_idx: usize) -> [u8; 4] {
    if shard_idx < 4 {
        let mut r = [0u8; 4];
        r[shard_idx] = 1;
        r
    } else {
        let x = (shard_idx - 3) as u8; // 1, 2, 3, 4
        let mut r = [0u8; 4];
        for (j, slot) in r.iter_mut().enumerate() {
            let y = (j + 5) as u8; // 5, 6, 7, 8
            *slot = gf_inv(x ^ y);
        }
        r
    }
}

/// Invert any 4x4 submatrix of the systematic Cauchy MDS matrix over GF(2^8) (always nonsingular!).
fn invert_4x4(mut a: [[u8; 4]; 4]) -> Result<[[u8; 4]; 4], &'static str> {
    let mut inv = [[0u8; 4]; 4];
    for (i, row) in inv.iter_mut().enumerate() {
        row[i] = 1;
    }
    for col in 0..4 {
        let pivot = (col..4).find(|&r| a[r][col] != 0).ok_or("singular matrix")?;
        if pivot != col {
            a.swap(pivot, col);
            inv.swap(pivot, col);
        }
        let inv_p = gf_inv(a[col][col]);
        for j in 0..4 {
            a[col][j] = gf_mul(a[col][j], inv_p);
            inv[col][j] = gf_mul(inv[col][j], inv_p);
        }
        for r in 0..4 {
            if r == col {
                continue;
            }
            let f = a[r][col];
            if f != 0 {
                for j in 0..4 {
                    a[r][j] ^= gf_mul(f, a[col][j]);
                    inv[r][j] ^= gf_mul(f, inv[col][j]);
                }
            }
        }
    }
    Ok(inv)
}

fn leaf_hash(shard_idx: u8, data: &[u8]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(2 + data.len());
    buf.push(0x00); // domain separator: leaf
    buf.push(shard_idx);
    buf.extend_from_slice(data);
    sha256(&buf)
}

fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut buf = [0u8; 65];
    buf[0] = 0x01; // domain separator: internal node
    buf[1..33].copy_from_slice(left);
    buf[33..65].copy_from_slice(right);
    sha256(&buf)
}

#[derive(Debug, Clone)]
pub struct CapsuleShard {
    pub index: u8,
    pub data: Vec<u8>,
    pub merkle_proof: [[u8; 32]; 3], // depth-3 tree for 8 shards
}

#[derive(Debug, Clone)]
pub struct PlanariaCapsule {
    pub orig_len: usize,
    pub shard_len: usize,
    pub merkle_root: [u8; 32],
    pub payload_sha256: [u8; 32],
    pub shards: Vec<CapsuleShard>,
}

impl PlanariaCapsule {
    /// Encode arbitrary payload into 8 Merkle-authenticated Cauchy GF(2^8) shards (any 4 regenerate 100%).
    pub fn encode(payload: &[u8]) -> Self {
        let orig_len = payload.len();
        let shard_len = orig_len.div_ceil(4).max(1);
        let mut data_shards = vec![vec![0u8; shard_len]; 4];
        for (i, &b) in payload.iter().enumerate() {
            data_shards[i / shard_len][i % shard_len] = b;
        }

        let mut raw_shards: Vec<Vec<u8>> = Vec::with_capacity(8);
        for idx in 0..8 {
            let coeffs = encode_row(idx);
            let mut s = vec![0u8; shard_len];
            for pos in 0..shard_len {
                let mut v = 0u8;
                for j in 0..4 {
                    v ^= gf_mul(coeffs[j], data_shards[j][pos]);
                }
                s[pos] = v;
            }
            raw_shards.push(s);
        }

        // Depth-3 Merkle tree over 8 shards
        let leaves: [[u8; 32]; 8] = std::array::from_fn(|i| leaf_hash(i as u8, &raw_shards[i]));
        let lvl1: [[u8; 32]; 4] = std::array::from_fn(|i| node_hash(&leaves[2 * i], &leaves[2 * i + 1]));
        let lvl2: [[u8; 32]; 2] = std::array::from_fn(|i| node_hash(&lvl1[2 * i], &lvl1[2 * i + 1]));
        let merkle_root = node_hash(&lvl2[0], &lvl2[1]);

        let shards = (0..8)
            .map(|i| {
                let sib0 = leaves[i ^ 1];
                let sib1 = lvl1[(i >> 1) ^ 1];
                let sib2 = lvl2[(i >> 2) ^ 1];
                CapsuleShard {
                    index: i as u8,
                    data: raw_shards[i].clone(),
                    merkle_proof: [sib0, sib1, sib2],
                }
            })
            .collect();

        Self {
            orig_len,
            shard_len,
            merkle_root,
            payload_sha256: sha256(payload),
            shards,
        }
    }

    pub fn verify_shard(&self, shard: &CapsuleShard) -> bool {
        if shard.index >= 8 || shard.data.len() != self.shard_len {
            return false;
        }
        let mut h = leaf_hash(shard.index, &shard.data);
        let mut idx = shard.index as usize;
        for sib in &shard.merkle_proof {
            h = if (idx & 1) == 0 {
                node_hash(&h, sib)
            } else {
                node_hash(sib, &h)
            };
            idx >>= 1;
        }
        h == self.merkle_root
    }

    /// Quarantine Byzantine Trojan shards via Merkle verification, then heal from any >=4 authentic shards!
    /// Returns `(recovered_payload, quarantined_trojan_count)`.
    pub fn heal(&self, candidate_shards: &[CapsuleShard]) -> Result<(Vec<u8>, usize), &'static str> {
        let mut clean: Vec<&CapsuleShard> = Vec::new();
        let mut seen = [false; 8];
        let mut quarantined = 0usize;

        for s in candidate_shards {
            if !self.verify_shard(s) {
                quarantined += 1;
                continue;
            }
            if !seen[s.index as usize] {
                seen[s.index as usize] = true;
                clean.push(s);
            }
        }
        if clean.len() < 4 {
            return Err("insufficient authentic shards (< 4) for Planaria Cauchy recovery");
        }
        let chosen = &clean[..4];
        let mat: [[u8; 4]; 4] = std::array::from_fn(|r| encode_row(chosen[r].index as usize));
        let inv = invert_4x4(mat)?;

        let mut recovered = Vec::with_capacity(self.shard_len * 4);
        for inv_row in &inv {
            let mut block = vec![0u8; self.shard_len];
            for (r, &coeff) in inv_row.iter().enumerate() {
                for (dst, &src) in block.iter_mut().zip(&chosen[r].data) {
                    *dst ^= gf_mul(coeff, src);
                }
            }
            recovered.extend_from_slice(&block);
        }
        recovered.truncate(self.orig_len);
        if sha256(&recovered) != self.payload_sha256 {
            return Err("post-recovery SHA-256 mismatch");
        }
        Ok((recovered, quarantined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mitosis_50pct_annihilation_and_byzantine_trojan_quarantine() {
        let payload = b"BEAST-CIVILIZATION-PLANARIA-HBIN-COLONY-GENOME-V0.2.0-SELF-HEALING-PAYLOAD!!";
        let cap = PlanariaCapsule::encode(payload);

        // Slice capsule in half: destroy shards 0,1,2,3 completely (all 4 original data shards GONE!)
        // Keep only parity shards 4,5,6,7 + inject 2 forged Byzantine Trojan shards!
        let mut tortured = vec![
            cap.shards[4].clone(),
            cap.shards[5].clone(),
            cap.shards[6].clone(),
            cap.shards[7].clone(),
        ];
        let mut trojan0 = cap.shards[0].clone();
        trojan0.data[0] ^= 0xFF; // Byzantine poison!
        let mut trojan1 = cap.shards[1].clone();
        trojan1.data[3] ^= 0xAA;
        tortured.insert(0, trojan0);
        tortured.insert(2, trojan1);

        let (healed, quarantined) = cap.heal(&tortured).expect("Planaria heal must succeed");
        assert_eq!(quarantined, 2);
        assert_eq!(healed.as_slice(), payload);
    }
}
