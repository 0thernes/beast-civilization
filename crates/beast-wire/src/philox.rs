//! Domain-isolated Philox4x32-10 counter-based PRNG (matches Zig & AUTOPHAGE P4).

const PHILOX_M0: u64 = 0xD2511F53;
const PHILOX_M1: u64 = 0xCD9E8D57;
const PHILOX_W0: u32 = 0x9E3779B9;
const PHILOX_W1: u32 = 0xBB67AE85;

pub fn philox4x32_10(ctr: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    let mut c = ctr;
    let mut k = key;
    for _ in 0..10 {
        let p0 = PHILOX_M0.wrapping_mul(c[0] as u64);
        let p1 = PHILOX_M1.wrapping_mul(c[2] as u64);
        let hi0 = (p0 >> 32) as u32;
        let lo0 = p0 as u32;
        let hi1 = (p1 >> 32) as u32;
        let lo1 = p1 as u32;
        c = [hi1 ^ c[1] ^ k[0], lo1, hi0 ^ c[3] ^ k[1], lo0];
        k[0] = k[0].wrapping_add(PHILOX_W0);
        k[1] = k[1].wrapping_add(PHILOX_W1);
    }
    c
}

pub fn philox_u64(seed: u64, counter: u64, domain_id: u32) -> u64 {
    let ctr = [
        counter as u32,
        (counter >> 32) as u32,
        domain_id,
        0x4245_4153, // "BEAS"
    ];
    let key = [seed as u32, (seed >> 32) as u32];
    let out = philox4x32_10(ctr, key);
    (out[0] as u64) | ((out[1] as u64) << 32)
}

pub fn philox_f64_unit(seed: u64, counter: u64, domain_id: u32) -> f64 {
    let bits = philox_u64(seed, counter, domain_id) >> 11;
    (bits as f64) * (1.0 / ((1u64 << 53) as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_isolation_holds() {
        let a = philox_u64(5000, 1, 1);
        let b = philox_u64(5000, 1, 2);
        assert_ne!(a, b);
        assert_eq!(a, philox_u64(5000, 1, 1));
    }
}
