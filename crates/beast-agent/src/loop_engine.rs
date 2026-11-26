//! Unified Autonomous Agent Loop (`INIT` x `NonHuman` x `Engram` x `TEMM1E`).
//! Executes bare-metal software/OS/model engineering goals under Ring-0 ACL + spend metering,
//! Merkle Repo-Map context budgeting, blind verification, and signed Rosette-Wire receipts.

use crate::forge::{forge_chimera_model, forge_neural_kernel};
use crate::repo_map::build_merkle_repo_map;
use beast_colony::economics::classify_effort_tier;
use beast_root::Ring0Root;
use beast_wire::sha256::{hex_encode, sha256};
use beast_wire::{verify_chain, WireReceipt};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AgentRunReport {
    pub task: String,
    pub effort_tier: String,
    pub steps_executed: usize,
    pub repo_files: usize,
    pub repo_symbols: usize,
    pub repo_merkle_prefix: String,
    pub forged_kernel_elf_bytes: Option<usize>,
    pub forged_kernel_qemu_exact: bool,
    pub forged_model_gguf_bytes: Option<usize>,
    pub receipt_chain_valid: bool,
    pub final_receipt_hex: String,
    pub spend_used_microdollars: u64,
}

pub fn run_autonomous_agent_task(
    repo_root: &Path,
    task: &str,
    seed: u64,
) -> Result<AgentRunReport, String> {
    if task.trim().is_empty() {
        return Err("MUTANT CAUGHT: empty agent task prompt".into());
    }
    let lower = task.to_ascii_lowercase();
    if lower.contains("crates/beast-root") || lower.contains("modify ring-0") || lower.contains("bypass acl") {
        return Err("MUTANT CAUGHT: Ring-0 guardian blocked task requesting protected root modification".into());
    }

    let mut r0 = Ring0Root::new(b"beast-agent-v0.4", 250_000, task.as_bytes().to_vec());
    let key = sha256(b"beast-agent-rosette-key-v0.4");
    let mut chain: Vec<WireReceipt> = Vec::new();
    let mut prev = [0u8; 32];

    // Step 1: NonHuman Effort Routing + Merkle AST Repo-Map (strict 8 KiB context budget)
    let complexity = task.len().min(100) as u8;
    let effort = classify_effort_tier(complexity, lower.contains("proof"), lower.contains("kernel"));
    r0.meter_spend(1_500).map_err(|e| format!("{e:?}"))?;
    let rmap = build_merkle_repo_map(repo_root, 8_192)?;
    let r1 = WireReceipt::sign(
        0,
        "agent.merkle_repo_map",
        task.as_bytes(),
        &rmap.merkle_root,
        prev,
        &key,
        true,
        &format!("files={} syms={}", rmap.files_indexed, rmap.total_symbols),
    );
    prev = r1.digest();
    chain.push(r1);

    let mut forged_kernel_elf_bytes = None;
    let mut forged_kernel_qemu_exact = false;
    let mut forged_model_gguf_bytes = None;

    let want_kernel = lower.contains("kernel") || lower.contains("os") || lower.contains("full") || lower.contains("matrix");
    let want_model = lower.contains("model") || lower.contains("chimera") || lower.contains("gguf") || lower.contains("full") || lower.contains("matrix");

    // Step 2: Autonomous Freestanding Neural Microkernel Forge + QEMU Ring-0 Boot + Planaria Heal
    if want_kernel {
        r0.check_write_acl("bin/forged/agent_os_boot32.zig")
            .map_err(|e| format!("{e:?}"))?;
        r0.meter_spend(5_000).map_err(|e| format!("{e:?}"))?;
        let fk = forge_neural_kernel(repo_root, "agent_os", seed)?;
        forged_kernel_elf_bytes = Some(fk.elf_bytes);
        forged_kernel_qemu_exact = fk.qemu_bit_exact;
        let rk = WireReceipt::sign(
            chain.len() as u64,
            "agent.forge_neural_kernel",
            fk.name.as_bytes(),
            fk.elf_sha256_prefix.as_bytes(),
            prev,
            &key,
            fk.planaria_healed_exact,
            &format!("elf={}B qemu_exact={}", fk.elf_bytes, fk.qemu_bit_exact),
        );
        prev = rk.digest();
        chain.push(rk);
    }

    // Step 3: Autonomous Chimera Model + GGUF v3 + CacheGraft Forge
    if want_model {
        r0.check_write_acl("bin/forged/agent_chimera.gguf")
            .map_err(|e| format!("{e:?}"))?;
        r0.meter_spend(5_000).map_err(|e| format!("{e:?}"))?;
        let fm = forge_chimera_model(repo_root, "agent_chimera", seed)?;
        forged_model_gguf_bytes = Some(fm.gguf_bytes);
        let rm = WireReceipt::sign(
            chain.len() as u64,
            "agent.forge_chimera_model",
            fm.name.as_bytes(),
            fm.repo_merkle_prefix.as_bytes(),
            prev,
            &key,
            fm.planaria_heal_exact && fm.inference_exact,
            &format!("gguf={}B bcg={}B", fm.gguf_bytes, fm.bcg_bytes),
        );
        prev = rm.digest();
        chain.push(rm);
    }

    verify_chain(&chain, &key).map_err(|i| format!("Rosette-Wire chain failed at {i}"))?;

    Ok(AgentRunReport {
        task: task.to_string(),
        effort_tier: format!("{effort:?}"),
        steps_executed: chain.len(),
        repo_files: rmap.files_indexed,
        repo_symbols: rmap.total_symbols,
        repo_merkle_prefix: rmap.merkle_root_hex[..16].to_string(),
        forged_kernel_elf_bytes,
        forged_kernel_qemu_exact,
        forged_model_gguf_bytes,
        receipt_chain_valid: true,
        final_receipt_hex: hex_encode(&prev),
        spend_used_microdollars: r0.spent_micro_usd,
    })
}
