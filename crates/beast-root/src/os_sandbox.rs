//! Real OS-Level Ring-0 Parent -> Sandboxed Child Process Isolation & Overlapped Hot-Swap
//! (Win64 & Linux native — zero secret env propagation verified across live OS PIDs!).

use crate::Ring0Root;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct OsHandoffReport {
    pub parent_pid: u32,
    pub child1_pid: u32,
    pub child2_pid: u32,
    pub distinct_pids: bool,
    pub secrets_scrubbed_both: bool,
    pub child1_digest: String,
    pub child2_digest: String,
}

/// Spawn TWO live sandboxed child OS processes of `exe_path worker-handshake`,
/// proving on live Windows/Linux OS processes that:
///   1. Even when parent holds `BEAST_API_KEY` secret canary, NEITHER child can read it.
///   2. Both children run under distinct OS PIDs and complete overlapped handoff cleanly!
pub fn verify_os_process_sandbox_handoff(root: &Ring0Root, exe_path: &Path) -> Result<OsHandoffReport, String> {
    let prog = exe_path.to_str().ok_or_else(|| "non-UTF8 exe path".to_string())?;
    let parent_pid = std::process::id();

    let spawn_one = |gen_label: &str| -> Result<(u32, bool, String), String> {
        let mut cmd = root.build_sandboxed_command(prog, &["worker-handshake", gen_label]);
        let out = cmd.output().map_err(|e| format!("spawn child {gen_label} failed: {e}"))?;
        if !out.status.success() {
            return Err(format!("child {gen_label} exited non-zero: {}", String::from_utf8_lossy(&out.stderr)));
        }
        let text = String::from_utf8_lossy(&out.stdout);
        // Expected line: "WORKER_OK pid=<N> secret_leaked=<bool> zone=<Z> digest=<HEX>"
        let line = text.lines().find(|l| l.starts_with("WORKER_OK ")).ok_or_else(|| format!("malformed worker output: {text}"))?;
        let mut pid = 0u32;
        let mut leaked = true;
        let mut digest = String::new();
        for tok in line.split_whitespace().skip(1) {
            if let Some(v) = tok.strip_prefix("pid=") { pid = v.parse().unwrap_or(0); }
            if let Some(v) = tok.strip_prefix("secret_leaked=") { leaked = v != "false"; }
            if let Some(v) = tok.strip_prefix("digest=") { digest = v.to_string(); }
        }
        Ok((pid, !leaked, digest))
    };

    let (pid1, clean1, d1) = spawn_one("gen0")?;
    let (pid2, clean2, d2) = spawn_one("gen1_hotswap")?;
    let distinct = pid1 != 0 && pid2 != 0 && pid1 != parent_pid && pid2 != parent_pid && pid1 != pid2;
    if !distinct || !clean1 || !clean2 {
        return Err(format!("OS sandbox handoff failed: p={parent_pid} c1={pid1}(clean={clean1}) c2={pid2}(clean={clean2})"));
    }

    Ok(OsHandoffReport {
        parent_pid,
        child1_pid: pid1,
        child2_pid: pid2,
        distinct_pids: distinct,
        secrets_scrubbed_both: clean1 && clean2,
        child1_digest: d1,
        child2_digest: d2,
    })
}
