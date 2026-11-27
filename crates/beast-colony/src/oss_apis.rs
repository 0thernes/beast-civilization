//! Zero-dependency Live Public Open-Source API Super-Tool (`crates/beast-colony/src/oss_apis.rs`).
//! Enforces strict Ring-0 URL allowlisting over the 12 baked-in public scientific & systems APIs
//! (`arxiv`, `openalex`, `crates_io`, `osv`, `godbolt`, `github_public`, `huggingface`, etc.)
//! and keeps all offline tests 100% network-free.

use crate::economics::PUBLIC_OSS_APIS;
use std::process::Command;

pub fn build_allowlisted_url(provider_id: &str, query: &str) -> Result<String, String> {
    let api = PUBLIC_OSS_APIS
        .iter()
        .find(|a| a.id == provider_id)
        .ok_or_else(|| format!("Ring-0 blocked non-allowlisted API provider: {provider_id}"))?;

    // Strict input sanitization against shell/URL injection
    if query.is_empty() || query.len() > 200 || query.chars().any(|c| c.is_control() || matches!(c, '&' | '|' | ';' | '`' | '$')) {
        return Err("Ring-0 rejected malformed API query".into());
    }
    let encoded: String = query
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect();

    let url = match provider_id {
        "arxiv" => format!("{}?search_query=all:{}&max_results=3", api.endpoint, encoded),
        "openalex" => format!("{}?search={}&per-page=3", api.endpoint, encoded),
        "crates_io" => format!("{}?q={}&per_page=3", api.endpoint, encoded),
        "huggingface" => format!("{}?search={}&limit=3", api.endpoint, encoded),
        "godbolt" => api.endpoint.to_string(),
        _ => format!("{}?q={}", api.endpoint, encoded),
    };
    Ok(url)
}

/// Live query using native OS `curl` (strict 10s timeout, UTF-8-safe 4 KiB bounded NonHuman output!).
pub fn query_public_oss_api(provider_id: &str, query: &str) -> Result<String, String> {
    let url = build_allowlisted_url(provider_id, query)?;
    let ua = concat!("beast-civilization/", env!("CARGO_PKG_VERSION"));
    let out = Command::new("curl")
        .args(["-fsSL", "--max-time", "10", "-A", ua, &url])
        .output()
        .map_err(|e| format!("failed to execute curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("HTTP error querying {provider_id}: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.len() > 4096 {
        let mut cut = 4096;
        while cut > 0 && !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
        text.push_str("\n...[NonHuman 4KiB output cap]");
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_blocks_arbitrary_hosts_and_injection_mutants() {
        assert!(build_allowlisted_url("arxiv", "cache-to-cache kv transfer").is_ok());
        assert!(build_allowlisted_url("evil_exfil_host", "secrets").is_err());
        assert!(build_allowlisted_url("arxiv", "foo; rm -rf /").is_err());
    }
}
