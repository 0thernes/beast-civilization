//! `beast-wire`: Rosette-Wire inspired verifiable component contracts,
//! chained cryptographic receipts, Philox4x32-10 deterministic PRNG,
//! and zero-dependency Win64/POSIX dynamic FFI loader.

pub mod dynlib;
pub mod philox;
pub mod sha256;

use sha256::{hex_encode, hmac_sha256, sha256};

/// Capabilities a component may request under Ring-0 arbitration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    PureCompute,
    ReadRepo,
    WriteSandbox,
    EmitKernel,
    LocalC2C,
    CloudEscalation,
    Ring0Root,
}

impl Capability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::PureCompute => "pure_compute",
            Capability::ReadRepo => "read_repo",
            Capability::WriteSandbox => "write_sandbox",
            Capability::EmitKernel => "emit_kernel",
            Capability::LocalC2C => "local_c2c",
            Capability::CloudEscalation => "cloud_escalation",
            Capability::Ring0Root => "ring0_root",
        }
    }
}

/// Typed component contract (Rosette-Wire pattern).
#[derive(Debug, Clone)]
pub struct ComponentContract {
    pub id: String,
    pub language: String,
    pub required_caps: Vec<Capability>,
    pub input_schema: String,
    pub output_schema: String,
}

/// Cryptographically chained execution receipt verified by an independent verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireReceipt {
    pub seq: u64,
    pub component_id: String,
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32],
    pub prev_receipt_hash: [u8; 32],
    pub verifier_mac: [u8; 32],
    pub passed: bool,
    pub summary: String,
}

impl WireReceipt {
    pub fn digest(&self) -> [u8; 32] {
        let mut h = sha256::Sha256::new();
        h.update(&self.seq.to_le_bytes());
        h.update(self.component_id.as_bytes());
        h.update(&self.input_hash);
        h.update(&self.output_hash);
        h.update(&self.prev_receipt_hash);
        h.update(&[u8::from(self.passed)]);
        h.update(self.summary.as_bytes());
        h.finalize()
    }

    pub fn sign(
        seq: u64,
        component_id: impl Into<String>,
        input_bytes: &[u8],
        output_bytes: &[u8],
        prev_receipt_hash: [u8; 32],
        root_key: &[u8; 32],
        passed: bool,
        summary: impl Into<String>,
    ) -> Self {
        let mut r = Self {
            seq,
            component_id: component_id.into(),
            input_hash: sha256(input_bytes),
            output_hash: sha256(output_bytes),
            prev_receipt_hash,
            verifier_mac: [0u8; 32],
            passed,
            summary: summary.into(),
        };
        let d = r.digest();
        r.verifier_mac = hmac_sha256(root_key, &d);
        r
    }

    pub fn verify(&self, expected_prev: &[u8; 32], root_key: &[u8; 32]) -> bool {
        if &self.prev_receipt_hash != expected_prev {
            return false;
        }
        let d = self.digest();
        let expected_mac = hmac_sha256(root_key, &d);
        // Constant-time comparison
        let mut diff = 0u8;
        for i in 0..32 {
            diff |= self.verifier_mac[i] ^ expected_mac[i];
        }
        diff == 0
    }

    pub fn digest_hex(&self) -> String {
        hex_encode(&self.digest())
    }
}

/// Verify an entire Rosette-Wire receipt chain from genesis ([0u8; 32]).
pub fn verify_chain(chain: &[WireReceipt], root_key: &[u8; 32]) -> Result<[u8; 32], usize> {
    let mut prev = [0u8; 32];
    for (idx, r) in chain.iter().enumerate() {
        if r.seq != idx as u64 || !r.verify(&prev, root_key) {
            return Err(idx);
        }
        prev = r.digest();
    }
    Ok(prev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_chain_and_tamper_mutant() {
        let key = sha256(b"beast-root-signing-key-v1");
        let r0 = WireReceipt::sign(0, "zig.b158", b"in0", b"out0", [0u8; 32], &key, true, "ok");
        let r1 = WireReceipt::sign(1, "c2c.augment", b"in1", b"out1", r0.digest(), &key, true, "ok");
        let mut chain = vec![r0, r1];
        assert!(verify_chain(&chain, &key).is_ok());

        // Planted negative mutant: flip 1 bit in output_hash of receipt 0
        chain[0].output_hash[0] ^= 0x01;
        assert_eq!(verify_chain(&chain, &key), Err(0));
    }
}
