//! Standardized MCP (Model Context Protocol) JSON-RPC 2.0 Stdio Server (`W088`, `F083`)
//! + Native Zig/LLVM x86-64 Disassembly & Assembly Inspector (`F087`).

use crate::repo_map::build_merkle_repo_map;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn json_escape_mcp(s: &str) -> String {
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

fn parse_jsonrpc_id(trimmed: &str) -> String {
    let Some(pos) = trimmed.find("\"id\":") else {
        return "1".to_string();
    };
    let rest = trimmed[pos + 5..].trim_start();
    if let Some(after_q) = rest.strip_prefix('"') {
        let mut s = String::new();
        for ch in after_q.chars().take(64) {
            if ch == '"' {
                return format!("\"{}\"", json_escape_mcp(&s));
            }
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
                s.push(ch);
            } else {
                break;
            }
        }
        return "null".to_string();
    }
    let digits: String = rest
        .chars()
        .take(20)
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        "null".to_string()
    } else {
        digits
    }
}

/// Minimal zero-dependency JSON-RPC 2.0 MCP request dispatcher.
pub fn handle_mcp_jsonrpc_line(repo_root: &Path, line: &str) -> Result<String, String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') || !trimmed.contains("\"jsonrpc\"") {
        return Err("MUTANT CAUGHT: malformed JSON-RPC 2.0 frame".into());
    }

    let id_val = parse_jsonrpc_id(trimmed);

    if trimmed.contains("\"method\":\"initialize\"") || trimmed.contains("\"method\": \"initialize\"") {
        return Ok(format!(
            r#"{{"jsonrpc":"2.0","id":{id_val},"result":{{"protocolVersion":"2024-11-05","capabilities":{{"tools":{{}},"resources":{{}}}},"serverInfo":{{"name":"beast-civilization-mcp","version":"{}"}}}}}}"#,
            env!("CARGO_PKG_VERSION")
        ));
    }

    if trimmed.contains("\"method\":\"tools/list\"") || trimmed.contains("\"method\": \"tools/list\"") {
        return Ok(format!(
            r#"{{"jsonrpc":"2.0","id":{id_val},"result":{{"tools":[{{"name":"beast_repo_map","description":"Merkle multi-language AST Repo-Map"}},{{"name":"beast_inspect_asm","description":"Compile & inspect x86-64 LLVM assembly"}},{{"name":"beast_forge_kernel","description":"Synthesize & QEMU-verify freestanding OS microkernel"}},{{"name":"beast_forge_alife","description":"Synthesize & verify bare-metal A-Life binary"}}]}}}}"#
        ));
    }

    if trimmed.contains("\"method\":\"resources/list\"") || trimmed.contains("\"method\": \"resources/list\"") {
        return Ok(format!(
            r#"{{"jsonrpc":"2.0","id":{id_val},"result":{{"resources":[{{"uri":"beast://repo/merkle-map","name":"Merkle AST Repo-Map"}},{{"uri":"beast://verify/lean4-audit","name":"Pure Lean 4 Axiom Audit"}}]}}}}"#
        ));
    }

    if (trimmed.contains("\"method\":\"tools/call\"") || trimmed.contains("\"method\": \"tools/call\""))
        && trimmed.contains("beast_repo_map")
    {
        let rmap = build_merkle_repo_map(repo_root, 4096)?;
        return Ok(format!(
            r#"{{"jsonrpc":"2.0","id":{id_val},"result":{{"content":[{{"type":"text","text":"files={} symbols={} merkle={}"}}]}}}}"#,
            rmap.files_indexed,
            rmap.total_symbols,
            &rmap.merkle_root_hex[..16]
        ));
    }

    Err(format!("MUTANT CAUGHT: unknown MCP JSON-RPC method in {trimmed}"))
}

/// Runs live stdio JSON-RPC 2.0 MCP server loop (`beast mcp-serve`).
pub fn serve_mcp_stdio(repo_root: &Path) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let resp = match handle_mcp_jsonrpc_line(repo_root, &line) {
            Ok(ok_json) => ok_json,
            Err(e) => format!(
                r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":-32600,"message":"{}"}}}}"#,
                json_escape_mcp(&e)
            ),
        };
        let _ = writeln!(stdout, "{resp}");
        let _ = stdout.flush();
    }
}

#[derive(Debug, Clone)]
pub struct AsmInspectReport {
    pub asm_bytes: usize,
    pub instruction_lines: usize,
    pub exported_symbols: usize,
    pub has_ret_instruction: bool,
    pub has_simd_or_alu: bool,
}

/// Compiles `zig/beast-metal/src/root.zig` with `-OReleaseFast -femit-asm=...` (`F087` Godbolt/LLVM inspection)
/// and forensically parses real emitted x86-64 machine assembly!
pub fn inspect_zig_x86_64_asm(repo_root: &Path) -> Result<AsmInspectReport, String> {
    let zig = {
        let p = PathBuf::from(r"C:\tools\zig\zig.exe");
        if p.is_file() { p } else { PathBuf::from("zig") }
    };
    let out_dir = repo_root.join("bin").join("forged");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let asm_path = out_dir.join("beast_metal_disasm.s");
    let obj_path = out_dir.join("beast_metal_disasm.obj");

    let out = Command::new(&zig)
        .arg("build-obj")
        .arg(repo_root.join("zig").join("beast-metal").join("src").join("root.zig"))
        .arg("-OReleaseFast")
        .arg(format!("-femit-asm={}", asm_path.display()))
        .arg(format!("-femit-bin={}", obj_path.display()))
        .output()
        .map_err(|e| format!("zig -femit-asm failed: {e}"))?;
    if !out.status.success() {
        return Err(format!("zig asm emit failed: {}", String::from_utf8_lossy(&out.stderr)));
    }

    let asm_text = fs::read_to_string(&asm_path).map_err(|e| e.to_string())?;
    let mut instruction_lines = 0usize;
    let mut exported_symbols = 0usize;
    let mut has_ret = false;
    let mut has_simd_or_alu = false;

    for raw in asm_text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if line.starts_with(".globl") || line.starts_with(".global") || line.starts_with(".def") {
            exported_symbols += 1;
            continue;
        }
        if line.starts_with('.') || line.ends_with(':') {
            continue;
        }
        instruction_lines += 1;
        if line.starts_with("ret") {
            has_ret = true;
        }
        if line.starts_with("vp") || line.starts_with("vfm") || line.starts_with("imul") || line.starts_with("add") || line.starts_with("xor") {
            has_simd_or_alu = true;
        }
    }

    if instruction_lines < 100 || exported_symbols < 5 || !has_ret || !has_simd_or_alu {
        return Err(format!(
            "x86-64 assembly inspection sanity failed: instrs={instruction_lines} syms={exported_symbols} ret={has_ret}"
        ));
    }

    Ok(AsmInspectReport {
        asm_bytes: asm_text.len(),
        instruction_lines,
        exported_symbols,
        has_ret_instruction: has_ret,
        has_simd_or_alu,
    })
}
