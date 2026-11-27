//! Zero-Dependency OpenAI / Ollama / llama.cpp / `beast serve` HTTP/1.1 LLM Client
//! with Ring-0 SSRF Loopback Allowlist & CRLF Injection Protection (`crates/beast-agent/src/llm_client.rs`).

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ParsedHttpUrl {
    pub host: String,
    pub port: u16,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct ChatCompletionReply {
    pub endpoint: String,
    pub model: String,
    pub status_code: u16,
    pub assistant_content: String,
    pub raw_bytes: usize,
}

pub fn is_loopback_host(host: &str) -> bool {
    let low = host.to_ascii_lowercase();
    matches!(low.as_str(), "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

/// Parse & validate an HTTP/1.1 endpoint URL under Ring-0 SSRF and CRLF guards.
pub fn validate_llm_endpoint(url: &str, allow_remote: bool) -> Result<ParsedHttpUrl, String> {
    if url.bytes().any(|b| b < 0x20 || b == 0x7F || b == b' ') {
        return Err("MUTANT CAUGHT: CRLF/control-byte/whitespace injection in LLM endpoint URL".into());
    }
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("MUTANT CAUGHT: only http:// endpoints supported directly by bare-metal client (got {url})"))?;
    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], rest[idx..].to_string()),
        None => (rest, "/v1/chat/completions".to_string()),
    };
    if authority.is_empty() || authority.contains('@') || authority.contains(' ') {
        return Err("MUTANT CAUGHT: malformed authority in LLM endpoint URL".into());
    }
    let (host, port) = if let Some(stripped) = authority.strip_prefix('[') {
        let end = stripped.find(']').ok_or("invalid IPv6 literal")?;
        let h = format!("[{}]", &stripped[..end]);
        let after = &stripped[end + 1..];
        let p = if let Some(ps) = after.strip_prefix(':') {
            ps.parse::<u16>().map_err(|_| "invalid port")?
        } else {
            80
        };
        (h, p)
    } else if let Some((h, ps)) = authority.rsplit_once(':') {
        (h.to_string(), ps.parse::<u16>().map_err(|_| "invalid port")?)
    } else {
        (authority.to_string(), 80)
    };
    if host.is_empty() || host == "0.0.0.0" {
        return Err("MUTANT CAUGHT: empty or unspecified host in LLM endpoint URL".into());
    }

    if !allow_remote && !is_loopback_host(&host) {
        return Err(format!(
            "MUTANT CAUGHT: Ring-0 SSRF guard blocked non-loopback LLM host '{host}' (pass --allow-remote explicitly)"
        ));
    }
    Ok(ParsedHttpUrl { host, port, path })
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {}
            c => out.push(c),
        }
    }
    out
}

fn extract_assistant_content(body: &str) -> String {
    let needle = "\"content\"";
    let Some(pos) = body.find(needle) else {
        return body.trim().to_string();
    };
    let mut idx = pos + needle.len();
    let bytes = body.as_bytes();
    while idx < bytes.len() && matches!(bytes[idx], b' ' | b':' | b'\t') {
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b'"' {
        return body.trim().to_string();
    }
    idx += 1;
    let mut raw_out: Vec<u8> = Vec::new();
    let mut esc = false;
    while idx < bytes.len() {
        let b = bytes[idx];
        idx += 1;
        if esc {
            match b {
                b'n' => raw_out.push(b'\n'),
                b'r' => raw_out.push(b'\r'),
                b't' => raw_out.push(b'\t'),
                _ => raw_out.push(b),
            }
            esc = false;
        } else if b == b'\\' {
            esc = true;
        } else if b == b'"' {
            break;
        } else {
            raw_out.push(b);
        }
    }
    String::from_utf8_lossy(&raw_out).into_owned()
}

/// Execute a real HTTP/1.1 `POST /v1/chat/completions` against `beast serve`, Ollama, llama-server, or vLLM.
pub fn post_chat_completion(
    endpoint_url: &str,
    model: &str,
    prompt: &str,
    allow_remote: bool,
) -> Result<ChatCompletionReply, String> {
    let parsed = validate_llm_endpoint(endpoint_url, allow_remote)?;
    let addr = format!("{}:{}", parsed.host, parsed.port);
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| format!("TCP connect to LLM endpoint {addr} failed: {e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let body = format!(
        r#"{{"model":"{}","temperature":0,"messages":[{{"role":"system","content":"You are the BEAST Civilization bare-metal planner."}},{{"role":"user","content":"{}"}}]}}"#,
        json_escape(model),
        json_escape(prompt)
    );

    let auth_hdr = match env::var("BEAST_API_KEY") {
        Ok(tok) if !tok.is_empty() && !tok.contains('\r') && !tok.contains('\n') => {
            format!("Authorization: Bearer {tok}\r\n")
        }
        _ => String::new(),
    };

    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        parsed.path,
        parsed.host,
        parsed.port,
        auth_hdr,
        body.len(),
        body
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("HTTP write failed: {e}"))?;
    stream.flush().map_err(|e| format!("HTTP flush failed: {e}"))?;

    let mut resp_bytes = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if resp_bytes.len() + n > MAX_RESPONSE_BYTES {
                    return Err("LLM HTTP response exceeded 1 MiB safety cap".into());
                }
                resp_bytes.extend_from_slice(&chunk[..n]);
            }
            Err(e) => return Err(format!("HTTP read failed: {e}")),
        }
    }

    let raw = String::from_utf8_lossy(&resp_bytes);
    let (hdr, resp_body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "malformed HTTP response (missing header separator)".to_string())?;
    let status_code = hdr
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if status_code != 200 {
        return Err(format!("LLM endpoint returned HTTP {status_code}: {resp_body}"));
    }

    Ok(ChatCompletionReply {
        endpoint: endpoint_url.to_string(),
        model: model.to_string(),
        status_code,
        assistant_content: extract_assistant_content(resp_body),
        raw_bytes: resp_bytes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssrf_and_crlf_mutants_blocked() {
        assert!(validate_llm_endpoint("http://127.0.0.1:11435/v1/chat/completions", false).is_ok());
        assert!(validate_llm_endpoint("http://169.254.169.254/latest/meta-data", false).is_err());
        assert!(validate_llm_endpoint("http://127.0.0.1:11435/v1\r\nX-Evil: 1", false).is_err());
        assert!(validate_llm_endpoint("file:///etc/passwd", true).is_err());
        assert!(validate_llm_endpoint("http://api.example.com:8080/v1/chat/completions", true).is_ok());
    }
}
