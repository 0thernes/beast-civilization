//! Unified Autonomous Agent Loop (`INIT` x `NonHuman` x `Engram` x `TEMM1E` — `v0.5.0`).
//! Executes bare-metal Trinity Forge goals (OS Microkernel + Chimera Model + Standalone A-Life World)
//! under Ring-0 ACL + spend metering, Merkle Repo-Map budgeting, blind verification, and signed Rosette-Wire receipts.

use crate::alife_forge::forge_bare_metal_alife;
use crate::forge::{forge_chimera_model, forge_neural_kernel};
use crate::hyper_graph::{build_cross_repo_abi_graph, verify_confluent_scheduler_and_worktrees};
use crate::llm_client::post_chat_completion;
use crate::repo_map::build_merkle_repo_map;
use beast_colony::civilization::run_colony_civilization;
use beast_colony::economics::classify_effort_tier;
use beast_root::Ring0Root;
use beast_wire::sha256::{hex_encode, sha256};
use beast_wire::{verify_chain, WireReceipt};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EndpointMissionReport {
    pub endpoint: String,
    pub model: String,
    pub llm_plan: String,
    pub colony_agents: usize,
    pub cross_repo_edges: usize,
    pub confluent_tasks: usize,
    pub forged_model_gguf_bytes: Option<usize>,
    pub receipt_chain_len: usize,
    pub final_receipt_hex: String,
    pub spend_used_microdollars: u64,
}

/// Wire a live OpenAI/Ollama/llama.cpp/`beast serve` endpoint into the Ring-0 metered Colony mission loop.
pub fn run_endpoint_wired_agent_mission(
    repo_root: &Path,
    task: &str,
    endpoint_url: &str,
    model: &str,
    allow_remote: bool,
    seed: u64,
) -> Result<EndpointMissionReport, String> {
    if task.trim().is_empty() {
        return Err("MUTANT CAUGHT: empty mission task".into());
    }
    let low = task.to_ascii_lowercase();
    if low.contains("crates/beast-root") || low.contains("bypass acl") || low.contains("modify ring-0") {
        return Err("MUTANT CAUGHT: Ring-0 guardian blocked protected-root mission".into());
    }

    let mut r0 = Ring0Root::new(b"beast-endpoint-mission-v0.7", 250_000, task.as_bytes().to_vec());
    let key = sha256(b"beast-endpoint-rosette-key-v0.7");
    let mut chain: Vec<WireReceipt> = Vec::new();
    let mut prev = [0u8; 32];

    // Step 1: Query live OpenAI-compatible GGUF/C2C endpoint
    r0.meter_spend(2_000).map_err(|e| format!("{e:?}"))?;
    let reply = post_chat_completion(endpoint_url, model, task, allow_remote)?;
    let r_llm = WireReceipt::sign(
        0,
        "agent.llm_endpoint_plan",
        task.as_bytes(),
        reply.assistant_content.as_bytes(),
        prev,
        &key,
        reply.status_code == 200,
        &format!("http={} bytes={}", reply.status_code, reply.raw_bytes),
    );
    prev = r_llm.digest();
    chain.push(r_llm);

    // Step 2: Execute live Colony QUBO dispatch + Cross-Repo ABI graph + Confluent scheduler
    r0.meter_spend(3_000).map_err(|e| format!("{e:?}"))?;
    let ap_dll = repo_root.parent().unwrap_or(repo_root).join("autophage").join("bin").join("autophage.dll");
    let civ = run_colony_civilization(10, seed, &ap_dll);
    let xgraph = build_cross_repo_abi_graph(repo_root)?;
    let cw = verify_confluent_scheduler_and_worktrees(repo_root)?;
    let r_col = WireReceipt::sign(
        1,
        "agent.colony_and_cross_repo",
        civ.civilization_digest_hex.as_bytes(),
        xgraph.graph_merkle_prefix.as_bytes(),
        prev,
        &key,
        cw.church_rosser_confluent && cw.worktrees_isolated,
        &format!("agents={} edges={} tasks={}", civ.total_agents, xgraph.cross_repo_edges, cw.tasks_reduced),
    );
    prev = r_col.digest();
    chain.push(r_col);

    // Step 3: Optionally forge Chimera GGUF if LLM plan requests it
    let mut forged_model_gguf_bytes = None;
    if reply.assistant_content.contains("forge_model") || low.contains("model") || low.contains("chimera") {
        r0.check_write_acl("bin/forged/endpoint_chimera.gguf")
            .map_err(|e| format!("{e:?}"))?;
        r0.meter_spend(4_000).map_err(|e| format!("{e:?}"))?;
        let fm = forge_chimera_model(repo_root, "endpoint_chimera", seed)?;
        forged_model_gguf_bytes = Some(fm.gguf_bytes);
        let r_m = WireReceipt::sign(
            2,
            "agent.endpoint_forge_model",
            fm.name.as_bytes(),
            fm.repo_merkle_prefix.as_bytes(),
            prev,
            &key,
            fm.planaria_heal_exact && fm.inference_exact,
            &format!("gguf={}B", fm.gguf_bytes),
        );
        prev = r_m.digest();
        chain.push(r_m);
    }

    verify_chain(&chain, &key).map_err(|i| format!("Endpoint mission chain failed at {i}"))?;

    Ok(EndpointMissionReport {
        endpoint: reply.endpoint,
        model: reply.model,
        llm_plan: reply.assistant_content,
        colony_agents: civ.total_agents,
        cross_repo_edges: xgraph.cross_repo_edges,
        confluent_tasks: cw.tasks_reduced,
        forged_model_gguf_bytes,
        receipt_chain_len: chain.len(),
        final_receipt_hex: hex_encode(&prev),
        spend_used_microdollars: r0.spent_micro_usd,
    })
}

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
    pub forged_alife_exe_bytes: Option<usize>,
    pub forged_alife_p9_exact: bool,
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

    let mut r0 = Ring0Root::new(b"beast-agent-v0.5", 250_000, task.as_bytes().to_vec());
    let key = sha256(b"beast-agent-rosette-key-v0.5");
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
    let mut forged_alife_exe_bytes = None;
    let mut forged_alife_p9_exact = false;

    let want_kernel = lower.contains("kernel") || lower.contains("os") || lower.contains("full") || lower.contains("matrix");
    let want_model = lower.contains("model") || lower.contains("chimera") || lower.contains("gguf") || lower.contains("full") || lower.contains("matrix");
    let want_alife = lower.contains("alife") || lower.contains("a-life") || lower.contains("creature") || lower.contains("full") || lower.contains("matrix");

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

    // Step 4: Autonomous Standalone Bare-Metal Zig A-Life Creature Simulator Forge + P9 & Planaria Heal
    if want_alife {
        r0.check_write_acl("bin/forged/agent_alife_alife.zig")
            .map_err(|e| format!("{e:?}"))?;
        r0.meter_spend(5_000).map_err(|e| format!("{e:?}"))?;
        let fa = forge_bare_metal_alife(repo_root, "agent_alife", seed)?;
        forged_alife_exe_bytes = Some(fa.exe_bytes);
        forged_alife_p9_exact = fa.observer_independent_p9 && fa.energy_residual == 0;
        let ra = WireReceipt::sign(
            chain.len() as u64,
            "agent.forge_bare_metal_alife",
            fa.name.as_bytes(),
            fa.world_sha256_prefix.as_bytes(),
            prev,
            &key,
            fa.planaria_healed_exact && forged_alife_p9_exact,
            &format!("exe={}B res={} p9={}", fa.exe_bytes, fa.energy_residual, forged_alife_p9_exact),
        );
        prev = ra.digest();
        chain.push(ra);
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
        forged_alife_exe_bytes,
        forged_alife_p9_exact,
        receipt_chain_valid: true,
        final_receipt_hex: hex_encode(&prev),
        spend_used_microdollars: r0.spent_micro_usd,
    })
}
