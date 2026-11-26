//! `beast-root`: Immutable Ring-0 Guardian fusing `nagisanzenin/INIT-Agent`
//! (ported natively to Windows x64 + Linux) with `0thernes/noetic-organism-omega`
//! Ring-0 ACL and 0-bit cryptographic rollback.

use beast_wire::sha256::{hex_encode, sha256};
use beast_wire::{verify_chain, WireReceipt};
use std::collections::BTreeMap;
use std::process::Command;

pub mod os_sandbox;
pub mod win32_job;

/// Mutable harness tiers (INIT-Agent T0..T2 extended with CRuby/YARV T1a & Zig/CUDA T3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessTier {
    T0PromptsAndSkills,
    T1aYarvBytecode,
    T1bWasmOrEshkolTools,
    T2CompiledRustCore,
    T3ZigAndCudaKernels,
}

/// The 12 post-swap harm signals monitored by Ring-0 (INIT-Agent ADR-011 + Noetic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarmSignal {
    CrashLoop,
    SpendSpike,
    ErrorRateSpike,
    LatencyRegression,
    Ring0AclBreach,
    NanExplosion,
    SpectralOverflow,
    C2cFallbackSpoof,
    NonDeterminismDrift,
    MemoryLeak,
    TestWeakening,
    LineageFork,
}

#[derive(Debug, Clone)]
pub struct GateReport {
    pub g0_build_and_test: bool,
    pub g1_self_map_parity: bool,
    pub g2_shadow_boot: bool,
    pub g3_self_edit_preserved: bool,
    pub g4_blind_eval_clean: bool,
    pub harm_signals: Vec<HarmSignal>,
}

impl GateReport {
    pub fn all_passed(&self) -> bool {
        self.g0_build_and_test
            && self.g1_self_map_parity
            && self.g2_shadow_boot
            && self.g3_self_edit_preserved
            && self.g4_blind_eval_clean
            && self.harm_signals.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub version_id: u64,
    pub state_bytes: Vec<u8>,
    pub state_sha256: [u8; 32],
}

/// Ring-0 protected prefixes that the Agent Zone can NEVER overwrite.
const PROTECTED_PREFIXES: &[&str] = &[
    "crates/beast-root",
    "lean/",
    ".env",
    "root_charter",
    "signing_key",
    "spend_ledger",
    "AGENTS.md",
];

/// Environment variables scrubbed before spawning any Agent-Zone subprocess.
const SECRET_ENV_SUBSTRINGS: &[&str] = &[
    "API_KEY",
    "SECRET",
    "TOKEN",
    "PASSWORD",
    "PRIVATE_KEY",
    "CREDENTIAL",
];

pub struct Ring0Root {
    signing_key: [u8; 32],
    secrets: BTreeMap<String, String>,
    pub charter_hash: [u8; 32],
    pub spend_cap_micro_usd: u64,
    pub spent_micro_usd: u64,
    pub acl_blocked_attempts: u64,
    pub rollbacks_executed: u64,
    pub live_state: Vec<u8>,
    snapshots: Vec<SystemSnapshot>,
    lineage: Vec<WireReceipt>,
    pub panic_halted: bool,
}

impl Ring0Root {
    pub fn new(charter: &[u8], spend_cap_micro_usd: u64, initial_state: Vec<u8>) -> Self {
        let charter_hash = sha256(charter);
        let signing_key = sha256(&[b"RING0-SIGNING-KEY:", charter].concat());
        let mut root = Self {
            signing_key,
            secrets: BTreeMap::new(),
            charter_hash,
            spend_cap_micro_usd,
            spent_micro_usd: 0,
            acl_blocked_attempts: 0,
            rollbacks_executed: 0,
            live_state: initial_state.clone(),
            snapshots: Vec::new(),
            lineage: Vec::new(),
            panic_halted: false,
        };
        root.take_snapshot(initial_state);
        root
    }

    /// Store a secret exclusively inside Ring-0 (Agent Zone can never read it).
    pub fn store_secret(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.secrets.insert(name.into(), value.into());
    }

    pub fn secret_count(&self) -> usize {
        self.secrets.len()
    }

    /// Check Ring-0 ACL against a target path proposed by an agent self-edit.
    pub fn check_write_acl(&mut self, rel_path: &str) -> Result<(), HarmSignal> {
        let normalized = rel_path.replace('\\', "/");
        if normalized.contains("..") || normalized.starts_with('/') || normalized.contains(':') {
            self.acl_blocked_attempts += 1;
            return Err(HarmSignal::Ring0AclBreach);
        }
        for prefix in PROTECTED_PREFIXES {
            if normalized == *prefix || normalized.starts_with(prefix) {
                self.acl_blocked_attempts += 1;
                return Err(HarmSignal::Ring0AclBreach);
            }
        }
        Ok(())
    }

    /// Meter API/compute spend in micro-USD; hard-refuse when cap would be exceeded.
    pub fn meter_spend(&mut self, cost_micro_usd: u64) -> Result<u64, HarmSignal> {
        if self.panic_halted {
            return Err(HarmSignal::SpendSpike);
        }
        let next = self.spent_micro_usd.saturating_add(cost_micro_usd);
        if next > self.spend_cap_micro_usd {
            return Err(HarmSignal::SpendSpike);
        }
        self.spent_micro_usd = next;
        Ok(self.spend_cap_micro_usd - self.spent_micro_usd)
    }

    pub fn take_snapshot(&mut self, state: Vec<u8>) -> [u8; 32] {
        let hash = sha256(&state);
        let id = self.snapshots.len() as u64;
        self.snapshots.push(SystemSnapshot {
            version_id: id,
            state_bytes: state,
            state_sha256: hash,
        });
        hash
    }

    /// Execute a gated self-modification across Gates G0-G4 with automatic 0-bit rollback.
    pub fn apply_self_rewrite(
        &mut self,
        tier: HarnessTier,
        target_path: &str,
        candidate_state: Vec<u8>,
        gates: GateReport,
    ) -> Result<WireReceipt, Vec<HarmSignal>> {
        if self.panic_halted {
            return Err(vec![HarmSignal::CrashLoop]);
        }
        if let Err(sig) = self.check_write_acl(target_path) {
            return Err(vec![sig]);
        }
        let prev_hash = self
            .lineage
            .last()
            .map(|r| r.digest())
            .unwrap_or([0u8; 32]);
        let seq = self.lineage.len() as u64;

        if !gates.all_passed() {
            let mut harms = gates.harm_signals.clone();
            if harms.is_empty() {
                harms.push(HarmSignal::ErrorRateSpike);
            }
            let _ = self.rollback_latest();
            let rec = WireReceipt::sign(
                seq,
                format!("root.reject.{tier:?}"),
                &self.live_state,
                &candidate_state,
                prev_hash,
                &self.signing_key,
                false,
                format!("rejected target={target_path} harms={harms:?}"),
            );
            self.lineage.push(rec);
            return Err(harms);
        }

        self.take_snapshot(self.live_state.clone());
        let old = std::mem::replace(&mut self.live_state, candidate_state);
        let rec = WireReceipt::sign(
            seq,
            format!("root.hotswap.{tier:?}"),
            &old,
            &self.live_state,
            prev_hash,
            &self.signing_key,
            true,
            format!("promoted target={target_path}"),
        );
        self.lineage.push(rec.clone());
        Ok(rec)
    }

    /// Instant 0-bit cryptographic rollback to the latest verified snapshot.
    pub fn rollback_latest(&mut self) -> Result<[u8; 32], String> {
        let snap = self
            .snapshots
            .last()
            .ok_or_else(|| "no snapshot available".to_string())?
            .clone();
        let actual = sha256(&snap.state_bytes);
        if actual != snap.state_sha256 {
            return Err("snapshot integrity corruption detected!".to_string());
        }
        self.live_state = snap.state_bytes;
        self.rollbacks_executed += 1;
        Ok(actual)
    }

    pub fn panic_stop(&mut self) -> [u8; 32] {
        self.panic_halted = true;
        self.rollback_latest().unwrap_or([0u8; 32])
    }

    pub fn verify_lineage(&self) -> Result<[u8; 32], usize> {
        verify_chain(&self.lineage, &self.signing_key)
    }

    pub fn live_state_sha256_hex(&self) -> String {
        hex_encode(&sha256(&self.live_state))
    }

    pub fn lineage_len(&self) -> usize {
        self.lineage.len()
    }

    /// Build a zero-secret sandboxed child Command (scrubs all secret env vars).
    pub fn build_sandboxed_command(&self, program: &str, args: &[&str]) -> Command {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.env_clear();
        for (k, v) in std::env::vars() {
            let ku = k.to_ascii_uppercase();
            let is_secret = SECRET_ENV_SUBSTRINGS.iter().any(|sub| ku.contains(sub));
            if !is_secret {
                cmd.env(k, v);
            }
        }
        cmd.env("BEAST_ZONE", "AGENT_SANDBOX_NO_SECRETS");
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring0_acl_and_rollback_falsification() {
        let mut root = Ring0Root::new(b"charter-v1", 1_000_000, b"state-v0".to_vec());
        root.store_secret("BEAST_API_KEY", "sk-never-leak");
        let h0 = root.live_state_sha256_hex();

        // Planted Mutant 1: Agent tries to overwrite Ring-0 Root itself -> MUST BLOCK
        let attack = root.apply_self_rewrite(
            HarnessTier::T2CompiledRustCore,
            "crates/beast-root/src/lib.rs",
            b"pwned".to_vec(),
            GateReport {
                g0_build_and_test: true,
                g1_self_map_parity: true,
                g2_shadow_boot: true,
                g3_self_edit_preserved: true,
                g4_blind_eval_clean: true,
                harm_signals: vec![],
            },
        );
        assert_eq!(attack.unwrap_err(), vec![HarmSignal::Ring0AclBreach]);
        assert_eq!(root.acl_blocked_attempts, 1);
        assert_eq!(root.live_state_sha256_hex(), h0);

        // Planted Mutant 2: Path traversal escape -> MUST BLOCK
        assert!(root.check_write_acl("../.env").is_err());
        assert!(root.check_write_acl("lean/BeastProofs.lean").is_err());
        assert_eq!(root.acl_blocked_attempts, 3);

        // Valid T1a YARV hot-swap passes G0-G4
        let ok = root.apply_self_rewrite(
            HarnessTier::T1aYarvBytecode,
            "ruby/colony_strategy.rb",
            b"state-v1".to_vec(),
            GateReport {
                g0_build_and_test: true,
                g1_self_map_parity: true,
                g2_shadow_boot: true,
                g3_self_edit_preserved: true,
                g4_blind_eval_clean: true,
                harm_signals: vec![],
            },
        );
        assert!(ok.is_ok());
        assert_eq!(root.live_state, b"state-v1");

        // Planted Mutant 3: Harm signal on subsequent rewrite triggers automatic 0-bit rollback
        let bad = root.apply_self_rewrite(
            HarnessTier::T3ZigAndCudaKernels,
            "zig/beast-metal/src/chimera.zig",
            b"broken-nan".to_vec(),
            GateReport {
                g0_build_and_test: true,
                g1_self_map_parity: true,
                g2_shadow_boot: true,
                g3_self_edit_preserved: true,
                g4_blind_eval_clean: false,
                harm_signals: vec![HarmSignal::SpectralOverflow],
            },
        );
        assert!(bad.is_err());
        assert_eq!(root.live_state, b"state-v0"); // rolled back to pre-swap snapshot
        assert!(root.verify_lineage().is_ok());

        // Planted Mutant 4: Spend cap exceeded -> MUST BLOCK
        assert!(root.meter_spend(500_000).is_ok());
        assert_eq!(root.meter_spend(600_000).unwrap_err(), HarmSignal::SpendSpike);
    }
}
