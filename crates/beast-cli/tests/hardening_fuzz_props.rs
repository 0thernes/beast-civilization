// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Alexander Stephen Donahue
//! Comprehensive Integration, Regression, Property & 1,000-Case Deterministic Fuzz Suite
//! (`Build Standard A09 / B06 / B07 / B10 / B15`).
//!
//! Exercises every untrusted-input parser and Ring-0 security boundary against adversarial
//! payloads, integer-overflow headers, recursive nesting bombs, UTF-8 edge cases, and
//! case-insensitive / `./`-prefixed path-traversal attempts.

use beast_agent::llm_client::validate_llm_endpoint;
use beast_agent::mcp_server::handle_mcp_jsonrpc_line;
use beast_c2c::graft_io::load_bcg_bytes;
use beast_model::gguf::{dequantize_q4_0, f16_to_f32, f32_to_f16, quantize_q4_0, GgufFile};
use beast_model::llm_server::{extract_last_json_content, spawn_ephemeral_loopback_server};
use beast_root::Ring0Root;
use beast_wire::elf_audit::{audit_elf32_multiboot1, audit_elf64_multiboot2};
use beast_wire::hbin_capsule::PlanariaCapsule;
use beast_wire::philox::philox_u64;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

/// Regression test for Cycle-1 HOLE-01: case-insensitive, `./`-prefixed, double-slash,
/// and control-byte Ring-0 ACL bypass vectors must all be rejected cleanly.
#[test]
fn regression_ring0_acl_case_prefix_and_control_bypass() {
    let mut r0 = Ring0Root::new(b"charter-regression-v1", 100_000, b"v0".to_vec());
    let attacks = [
        "crates/beast-root/src/lib.rs",
        "./crates/beast-root/src/lib.rs",
        "././crates//beast-root\\src\\lib.rs",
        "CRATES/BEAST-ROOT/src/lib.rs",
        "Lean/BeastProofs.lean",
        "./lean/Challenge.lean",
        "AGENTS.md",
        "./agents.md",
        ".env",
        "../outside.txt",
        "C:\\Windows\\System32\\cmd.exe",
        "safe\0crates/beast-root",
        "",
        ".",
    ];
    for atk in attacks {
        assert!(
            r0.check_write_acl(atk).is_err(),
            "Ring-0 ACL FAILED to block bypass vector: {atk:?}"
        );
    }
    assert_eq!(r0.acl_blocked_attempts as usize, attacks.len());
    assert!(r0.check_write_acl("bin/forged/valid_model.gguf").is_ok());
}

/// Regression test for Cycle-1 HOLE-02: GGUF v3 OOM header counts, integer overflow,
/// and nested `GGUF_TYPE_ARRAY` recursion bombs must return `Err` without panicking.
#[test]
fn regression_gguf_oom_overflow_and_recursion_bomb() {
    // 1. OOM n_tensors = 0xFFFFFFFF in a 24-byte file
    let mut oom_tensors = Vec::new();
    oom_tensors.extend_from_slice(b"GGUF");
    oom_tensors.extend_from_slice(&3u32.to_le_bytes());
    oom_tensors.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    oom_tensors.extend_from_slice(&0u64.to_le_bytes());
    assert!(GgufFile::decode(&oom_tensors).is_err());

    // 2. Recursive GGUF_TYPE_ARRAY bomb (depth = 20 > cap 8)
    let mut bomb = Vec::new();
    bomb.extend_from_slice(b"GGUF");
    bomb.extend_from_slice(&3u32.to_le_bytes());
    bomb.extend_from_slice(&0u64.to_le_bytes()); // n_tensors = 0
    bomb.extend_from_slice(&1u64.to_le_bytes()); // n_kv = 1
    bomb.extend_from_slice(&1u64.to_le_bytes()); // key len = 1
    bomb.push(b'k');
    bomb.extend_from_slice(&9u32.to_le_bytes()); // GGUF_TYPE_ARRAY
    for _ in 0..20 {
        bomb.extend_from_slice(&9u32.to_le_bytes()); // elem_ty = GGUF_TYPE_ARRAY
        bomb.extend_from_slice(&1u64.to_le_bytes()); // count = 1
    }
    assert!(GgufFile::decode(&bomb).is_err());
}

/// Regression test for Cycle-1 HOLE-03: ELF32/ELF64 crafted `phoff = usize::MAX - 2`
/// and `phentsize = 0` must never panic or spin.
#[test]
fn regression_elf_integer_overflow_and_zero_phentsize() {
    let mut elf64 = vec![0u8; 64];
    elf64[0..4].copy_from_slice(b"\x7FELF");
    elf64[4] = 2; // ELF64
    elf64[5] = 1; // LE
    elf64[18..20].copy_from_slice(&62u16.to_le_bytes());
    elf64[32..40].copy_from_slice(&(u64::MAX - 2).to_le_bytes()); // phoff overflow
    elf64[54..56].copy_from_slice(&56u16.to_le_bytes());
    elf64[56..58].copy_from_slice(&2u16.to_le_bytes());
    assert!(audit_elf64_multiboot2(&elf64).is_err());

    // phentsize = 0 with phnum = 65535
    elf64[32..40].copy_from_slice(&0u64.to_le_bytes());
    elf64[54..56].copy_from_slice(&0u16.to_le_bytes());
    elf64[56..58].copy_from_slice(&65535u16.to_le_bytes());
    assert!(audit_elf64_multiboot2(&elf64).is_err());

    let mut elf32 = vec![0u8; 52];
    elf32[0..4].copy_from_slice(b"\x7FELF");
    elf32[4] = 1;
    elf32[5] = 1;
    elf32[18..20].copy_from_slice(&3u16.to_le_bytes());
    elf32[28..32].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
    elf32[42..44].copy_from_slice(&32u16.to_le_bytes());
    elf32[44..46].copy_from_slice(&4u16.to_le_bytes());
    assert!(audit_elf32_multiboot1(&elf32).is_err());
}

/// Regression test for Cycle-1 HOLE-04: `.bcg` crafted `seq_len * dim` overflow must fail cleanly.
#[test]
fn regression_bcg_overflow_and_oom() {
    let mut buf = vec![0u8; 48];
    buf[0..4].copy_from_slice(b"BCGB");
    buf[4..8].copy_from_slice(&1u32.to_le_bytes());
    buf[8..12].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    buf[12..16].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    assert!(load_bcg_bytes(&buf, "fuzz.bcg").is_err());
}

/// Regression test for Cycle-1 HOLE-05 & BUG-04: HTTP `Content-Length` overflow (`usize::MAX`),
/// `405 Method Not Allowed`, `GET /healthz`, and multi-byte UTF-8 JSON extraction.
#[test]
fn regression_http_server_content_length_overflow_405_healthz_and_utf8() {
    let extracted = extract_last_json_content(r#"{"content":"héllo Ω αβγ"}"#);
    assert_eq!(extracted, "héllo Ω αβγ");

    let (url, handle) = spawn_ephemeral_loopback_server(None, 3).expect("spawn server");
    let host_port = url.trim_start_matches("http://").split('/').next().expect("host_port").to_string();

    // 1. GET /healthz
    let mut s1 = TcpStream::connect(&host_port).expect("connect healthz");
    s1.write_all(format!("GET /healthz HTTP/1.1\r\nHost: {host_port}\r\n\r\n").as_bytes()).unwrap();
    let mut r1 = String::new();
    s1.read_to_string(&mut r1).unwrap();
    assert!(r1.contains("200 OK") && r1.contains("\"status\":\"ok\""));

    // 2. Malicious Content-Length: 18446744073709551615 -> 413 Payload Too Large (no panic!)
    let mut s2 = TcpStream::connect(&host_port).expect("connect overflow cl");
    s2.write_all(
        format!("POST /v1/chat/completions HTTP/1.1\r\nHost: {host_port}\r\nContent-Length: 18446744073709551615\r\n\r\n").as_bytes(),
    ).unwrap();
    let mut r2 = String::new();
    s2.read_to_string(&mut r2).unwrap();
    assert!(r2.contains("413 Payload Too Large"), "got: {r2}");

    // 3. DELETE /health -> 405 Method Not Allowed
    let mut s3 = TcpStream::connect(&host_port).expect("connect 405");
    s3.write_all(format!("DELETE /health HTTP/1.1\r\nHost: {host_port}\r\n\r\n").as_bytes()).unwrap();
    let mut r3 = String::new();
    s3.read_to_string(&mut r3).unwrap();
    assert!(r3.contains("405 Method Not Allowed"), "got: {r3}");

    assert_eq!(handle.join().expect("server thread clean"), 3);
}

/// Regression test for Cycle-1 HOLE-08 & CONCERN-02: MCP JSON-RPC ID injection + SSRF case/0.0.0.0 guards.
#[test]
fn regression_mcp_id_injection_and_ssrf_guards() {
    let root = Path::new(".");
    let resp = handle_mcp_jsonrpc_line(
        root,
        r#"{"jsonrpc":"2.0","id":"evil\"injected","method":"initialize"}"#,
    )
    .expect("initialize with sanitized id");
    assert!(resp.contains(r#""id":null"#), "unclean id leaked: {resp}");
    assert!(!resp.contains("0.6.0"), "stale version string in MCP response: {resp}");

    assert!(validate_llm_endpoint("http://LOCALHOST:11435/v1/chat/completions", false).is_ok());
    assert!(validate_llm_endpoint("http://0.0.0.0:11435/v1/chat/completions", false).is_err());
    assert!(validate_llm_endpoint("http://127.0.0.1:11435/v1 /evil", false).is_err());
}

/// 1,000-case deterministic fuzz campaign (`Build Standard B15` — libFuzzer-compatible corpus generator)
/// hammering GGUF, ELF64, ELF32, `.bcg`, Q4_0, and MCP JSON-RPC parsers: ZERO panics allowed!
#[test]
fn fuzz_1000_cases_all_binary_and_protocol_parsers_never_panic() {
    let root = Path::new(".");
    for case_idx in 0u64..1000u64 {
        let len = (philox_u64(0xF022, case_idx, 1) % 256) as usize;
        let mut buf = Vec::with_capacity(len);
        for b in 0..len {
            buf.push((philox_u64(0xF022, case_idx * 300 + b as u64, 2) & 0xFF) as u8);
        }
        // Inject magic headers on 25% of cases so fuzzing penetrates deep past magic checks!
        if len >= 4 {
            match case_idx % 4 {
                0 => buf[0..4].copy_from_slice(b"GGUF"),
                1 => buf[0..4].copy_from_slice(b"\x7FELF"),
                2 => buf[0..4].copy_from_slice(b"BCGB"),
                _ => {}
            }
        }
        let _ = GgufFile::decode(&buf);
        let _ = audit_elf64_multiboot2(&buf);
        let _ = audit_elf32_multiboot1(&buf);
        let _ = load_bcg_bytes(&buf, "fuzz.bcg");
        let _ = dequantize_q4_0(&buf);
        let s = String::from_utf8_lossy(&buf);
        let _ = handle_mcp_jsonrpc_line(root, &s);
        let _ = validate_llm_endpoint(&s, false);
        let _ = extract_last_json_content(&s);
    }
}

/// Property tests (`Build Standard B10`): Planaria Cauchy MDS heals ALL 70 = C(8,4) 4-shard subsets
/// bit-exact, and `f32_to_f16`/`f16_to_f32` + `quantize_q4_0` satisfy finite round-trip bounds!
#[test]
fn property_planaria_all_70_subsets_and_f16_q40_bounds() {
    let payload = b"PROPERTY-CHECK-ALL-70-CAUCHY-MDS-4-OF-8-SUBSETS-BIT-EXACT!!";
    let cap = PlanariaCapsule::encode(payload);
    let mut subsets_checked = 0usize;
    for a in 0..8 {
        for b in (a + 1)..8 {
            for c in (b + 1)..8 {
                for d in (c + 1)..8 {
                    let chosen = [
                        cap.shards[a].clone(),
                        cap.shards[b].clone(),
                        cap.shards[c].clone(),
                        cap.shards[d].clone(),
                    ];
                    let (healed, q) = cap.heal(&chosen).expect("every 4-of-8 Cauchy subset must heal");
                    assert_eq!(q, 0);
                    assert_eq!(healed.as_slice(), payload);
                    subsets_checked += 1;
                }
            }
        }
    }
    assert_eq!(subsets_checked, 70);

    // IEEE f16 exact roundtrip on power-of-two representable values & Q4_0 bounded error across 64 seeds
    for &v in &[0.0f32, 0.5, -0.5, 1.0, -1.0, 2.0, -4.0, 16.0, 1024.0] {
        assert_eq!(f16_to_f32(f32_to_f16(v)), v);
    }
    for seed in 0u64..64u64 {
        let xs: Vec<f32> = (0..64)
            .map(|i| (((philox_u64(seed, i, 0x9400) % 1000) as f32) / 500.0) - 1.0)
            .collect();
        let packed = quantize_q4_0(&xs).expect("q4_0 pack");
        let deq = dequantize_q4_0(&packed).expect("q4_0 unpack");
        let max_err = xs.iter().zip(deq.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
        assert!(max_err < 0.16, "Q4_0 error exceeded bound on seed {seed}: {max_err}");
    }
}
