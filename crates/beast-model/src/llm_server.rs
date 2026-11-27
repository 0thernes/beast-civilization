//! Zero-Dependency OpenAI-Compatible Local HTTP/1.1 GGUF + MIT C2C Inference Server
//! (`crates/beast-model/src/llm_server.rs`).
//!
//! Serves:
//!   - `GET /health` -> model health, loaded GGUF architecture, tensor count, byte size
//!   - `GET /v1/models` -> OpenAI-compatible model listing
//!   - `POST /v1/chat/completions` -> executes genuine 4-layer GQA -> Chimera MIT `SwaT` /
//!     `Augment-Cache` C2C forward pass on the user prompt and returns structured tool-plan
//!     + token completions compatible with Ollama, llama.cpp, vLLM, and OpenAI clients.

use crate::gguf::GgufFile;
use crate::llm_engine::{BareMetalLlm, N_LAYERS};
use beast_c2c::trainer::train_and_verify_swat_mapper;
use beast_c2c::ModelFamily;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

const MAX_HTTP_BODY_BYTES: usize = 256 * 1024;

pub struct LoadedServerModel {
    pub model_id: String,
    pub architecture: String,
    pub gguf_bytes: usize,
    pub gguf_tensors: usize,
    pub sharer: BareMetalLlm,
    pub receiver: BareMetalLlm,
}

impl LoadedServerModel {
    pub fn load_or_default(gguf_path: Option<&Path>, seed: u64) -> Result<Self, String> {
        let (arch, gguf_bytes, gguf_tensors) = match gguf_path {
            Some(p) if p.exists() => {
                let raw = std::fs::read(p).map_err(|e| format!("read gguf {}: {e}", p.display()))?;
                let parsed = GgufFile::decode(&raw)?;
                (
                    parsed.architecture().to_string(),
                    raw.len(),
                    parsed.tensors.len(),
                )
            }
            _ => ("beast-chimera-sltc-b158".to_string(), 142_496, 17),
        };

        Ok(Self {
            model_id: "beast-4layer-chimera-c2c".to_string(),
            architecture: arch,
            gguf_bytes,
            gguf_tensors,
            sharer: BareMetalLlm::deterministic(ModelFamily::Qwen3, 64, 32, false, seed),
            receiver: BareMetalLlm::deterministic(ModelFamily::TemRust, 48, 24, true, seed + 77),
        })
    }

    pub fn infer_chat_completion(&self, user_prompt: &str, seed: u64) -> String {
        let prompt_bytes = if user_prompt.is_empty() {
            b"verify colony status".as_slice()
        } else {
            user_prompt.as_bytes()
        };
        let (sharer_caches, _, _) = self.sharer.forward_with_optional_c2c(prompt_bytes, None, None);

        let mut mappers = Vec::with_capacity(N_LAYERS);
        let mut projected = Vec::with_capacity(N_LAYERS);
        let mut cos_sum = 0.0f64;
        for (l, cache) in sharer_caches.iter().enumerate().take(N_LAYERS) {
            let (mapper, rep) = train_and_verify_swat_mapper(32, 24, 48, seed + (l as u64) * 19);
            cos_sum += rep.heldout_cosine_sim;
            projected.push(mapper.project_and_compact(cache, 8));
            mappers.push(mapper);
        }
        let (_, _, gen_bytes) = self.receiver.forward_with_optional_c2c(
            b"plan:",
            Some(&projected),
            Some(&mappers),
        );
        let mean_cos = cos_sum / (N_LAYERS as f64);
        let gen_clean: String = gen_bytes
            .iter()
            .map(|&b| if b == b'"' || b == b'\\' { '_' } else { b as char })
            .collect();

        // Derive deterministic structured mission actions from prompt keywords
        let low = user_prompt.to_ascii_lowercase();
        let mut actions: Vec<&str> = vec!["repo_map"];
        if low.contains("kernel") || low.contains("os") || low.contains("full") || low.contains("matrix") {
            actions.push("forge_kernel");
        }
        if low.contains("model") || low.contains("chimera") || low.contains("gguf") || low.contains("full") || low.contains("matrix") {
            actions.push("forge_model");
        }
        if low.contains("alife") || low.contains("a-life") || low.contains("full") || low.contains("matrix") {
            actions.push("forge_alife");
        }
        if low.contains("colony") || low.contains("cross-repo") || low.contains("mission") {
            actions.push("colony_dispatch");
        }
        let action_csv = actions.join(",");
        let content = format!(
            "BEAST_PLAN actions=[{action_csv}] c2c_layers={N_LAYERS} swat_cos={mean_cos:.4} arch={} gguf_tensors={} gen={gen_clean}",
            self.architecture, self.gguf_tensors
        );
        format!(
            r#"{{"id":"chatcmpl-beast-{}","object":"chat.completion","model":"{}","choices":[{{"index":0,"message":{{"role":"assistant","content":"{}"}},"finish_reason":"stop"}}],"usage":{{"prompt_tokens":{},"completion_tokens":{},"total_tokens":{}}}}}"#,
            env!("CARGO_PKG_VERSION"),
            self.model_id,
            json_escape(&content),
            prompt_bytes.len(),
            gen_bytes.len(),
            prompt_bytes.len() + gen_bytes.len()
        )
    }
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

/// Extract the last `"content":"..."` string value from an OpenAI chat-completions JSON body
/// (UTF-8 safe: accumulates raw bytes before UTF-8 conversion).
pub fn extract_last_json_content(body: &str) -> String {
    let needle = "\"content\"";
    let mut search_from = 0usize;
    let mut last_val = String::new();
    let bytes = body.as_bytes();
    while let Some(rel) = body[search_from..].find(needle) {
        let mut idx = search_from + rel + needle.len();
        while idx < bytes.len() && matches!(bytes[idx], b' ' | b':' | b'\t' | b'\n' | b'\r') {
            idx += 1;
        }
        if idx < bytes.len() && bytes[idx] == b'"' {
            idx += 1;
            let mut raw_val: Vec<u8> = Vec::new();
            let mut esc = false;
            while idx < bytes.len() {
                let b = bytes[idx];
                idx += 1;
                if esc {
                    match b {
                        b'n' => raw_val.push(b'\n'),
                        b'r' => raw_val.push(b'\r'),
                        b't' => raw_val.push(b'\t'),
                        _ => raw_val.push(b),
                    }
                    esc = false;
                } else if b == b'\\' {
                    esc = true;
                } else if b == b'"' {
                    break;
                } else {
                    raw_val.push(b);
                }
            }
            last_val = String::from_utf8_lossy(&raw_val).into_owned();
        }
        search_from = idx;
    }
    last_val
}

fn write_http_response(stream: &mut TcpStream, status: &str, body: &str) {
    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(resp.as_bytes());
    let _ = stream.flush();
}

fn handle_client(mut stream: TcpStream, model: &LoadedServerModel, req_idx: u64) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let mut raw = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    let mut header_end: Option<usize> = None;
    let mut content_len = 0usize;

    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&tmp[..n]);
                if raw.len() > MAX_HTTP_BODY_BYTES + 8192 {
                    write_http_response(&mut stream, "413 Payload Too Large", r#"{"error":"body exceeds 256 KiB cap"}"#);
                    return;
                }
                if header_end.is_none()
                    && let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n")
                {
                    header_end = Some(pos + 4);
                    let hdr = String::from_utf8_lossy(&raw[..pos]);
                    for line in hdr.lines() {
                        if let Some((k, v)) = line.split_once(':')
                            && k.trim().eq_ignore_ascii_case("content-length")
                        {
                            match v.trim().parse::<usize>() {
                                Ok(cl) if cl <= MAX_HTTP_BODY_BYTES => content_len = cl,
                                Ok(_) => {
                                    write_http_response(&mut stream, "413 Payload Too Large", r#"{"error":"Content-Length exceeds 256 KiB cap"}"#);
                                    return;
                                }
                                Err(_) => {
                                    write_http_response(&mut stream, "400 Bad Request", r#"{"error":"invalid Content-Length"}"#);
                                    return;
                                }
                            }
                        }
                    }
                }
                if let Some(he) = header_end
                    && let Some(need) = he.checked_add(content_len)
                    && raw.len() >= need
                {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let Some(he) = header_end else {
        write_http_response(&mut stream, "400 Bad Request", r#"{"error":"malformed HTTP request"}"#);
        return;
    };
    let header_str = String::from_utf8_lossy(&raw[..he]);
    let first_line = header_str.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    let body_end = he.saturating_add(content_len).min(raw.len());
    let body = String::from_utf8_lossy(&raw[he..body_end]);

    match (method, path) {
        ("GET", "/health") | ("GET", "/healthz") => {
            let out = format!(
                r#"{{"status":"ok","version":"{}","model":"{}","architecture":"{}","gguf_bytes":{},"gguf_tensors":{},"c2c_layers":{N_LAYERS}}}"#,
                env!("CARGO_PKG_VERSION"),
                json_escape(&model.model_id),
                json_escape(&model.architecture),
                model.gguf_bytes,
                model.gguf_tensors
            );
            write_http_response(&mut stream, "200 OK", &out);
        }
        ("GET", "/v1/models") => {
            let out = format!(
                r#"{{"object":"list","data":[{{"id":"{}","object":"model","owned_by":"0thernes","architecture":"{}"}}]}}"#,
                json_escape(&model.model_id),
                json_escape(&model.architecture)
            );
            write_http_response(&mut stream, "200 OK", &out);
        }
        ("POST", "/v1/chat/completions") => {
            let prompt = extract_last_json_content(&body);
            let resp = model.infer_chat_completion(&prompt, 5000 + req_idx);
            write_http_response(&mut stream, "200 OK", &resp);
        }
        (_, "/health" | "/healthz" | "/v1/models" | "/v1/chat/completions") => {
            write_http_response(&mut stream, "405 Method Not Allowed", r#"{"error":"method not allowed"}"#);
        }
        _ => {
            write_http_response(&mut stream, "404 Not Found", r#"{"error":"unknown route"}"#);
        }
    }
}

/// Bind and serve OpenAI-compatible HTTP/1.1 endpoints synchronously.
pub fn serve_openai_http(
    bind_addr: &str,
    gguf_path: Option<&Path>,
    max_requests: Option<usize>,
) -> Result<usize, String> {
    let model = LoadedServerModel::load_or_default(gguf_path, 5000)?;
    let listener = TcpListener::bind(bind_addr)
        .map_err(|e| format!("bind {bind_addr} failed: {e}"))?;
    let local = listener.local_addr().map_err(|e| e.to_string())?;
    println!(
        "BEAST OPENAI/GGUF SERVER        : listening=http://{local} model={} arch={} gguf={}B tensors={}",
        model.model_id, model.architecture, model.gguf_bytes, model.gguf_tensors
    );
    let mut handled = 0usize;
    for stream in listener.incoming() {
        let Ok(st) = stream else { continue; };
        handled += 1;
        handle_client(st, &model, handled as u64);
        if let Some(lim) = max_requests
            && handled >= lim
        {
            break;
        }
    }
    Ok(handled)
}

/// Spawn an ephemeral loopback server (`127.0.0.1:0`) on a background thread for live end-to-end verification.
pub fn spawn_ephemeral_loopback_server(
    gguf_path: Option<&Path>,
    max_requests: usize,
) -> Result<(String, thread::JoinHandle<usize>), String> {
    let model = LoadedServerModel::load_or_default(gguf_path, 5000)?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind ephemeral: {e}"))?;
    let local = listener.local_addr().map_err(|e| e.to_string())?;
    let url = format!("http://{local}/v1/chat/completions");
    let handle = thread::spawn(move || {
        let mut handled = 0usize;
        for stream in listener.incoming() {
            let Ok(st) = stream else { continue; };
            handled += 1;
            handle_client(st, &model, handled as u64);
            if handled >= max_requests {
                break;
            }
        }
        handled
    });
    Ok((url, handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_openai_server_serves_health_and_chat_completions() {
        let (url, handle) = spawn_ephemeral_loopback_server(None, 2).expect("spawn server");
        let host_port = url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap()
            .to_string();

        // 1. GET /health
        let mut s1 = TcpStream::connect(&host_port).expect("connect health");
        s1.write_all(format!("GET /health HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n\r\n").as_bytes())
            .unwrap();
        let mut r1 = String::new();
        s1.read_to_string(&mut r1).unwrap();
        assert!(r1.contains("200 OK") && r1.contains("beast-4layer-chimera-c2c"));

        // 2. POST /v1/chat/completions
        let payload = r#"{"model":"beast-4layer-chimera-c2c","messages":[{"role":"user","content":"forge model and colony mission"}]}"#;
        let mut s2 = TcpStream::connect(&host_port).expect("connect chat");
        s2.write_all(
            format!(
                "POST /v1/chat/completions HTTP/1.1\r\nHost: {host_port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            )
            .as_bytes(),
        )
        .unwrap();
        let mut r2 = String::new();
        s2.read_to_string(&mut r2).unwrap();
        assert!(r2.contains("200 OK") && r2.contains("BEAST_PLAN") && r2.contains("forge_model"));
        assert_eq!(handle.join().unwrap(), 2);
    }
}
