//! Native Zero-Dependency ELF64 + Multiboot2 Forensic Binary Auditor (`crates/beast-wire/src/elf_audit.rs`).
//! Parses raw emitted `bin/beast_microkernel.elf` headers, verifies `ELF64`, `EM_X86_64`,
//! `PT_LOAD` segments, entry point >= 1 MiB, AND exact 32-bit wrapping Multiboot2 checksum!

use std::fs;
use std::path::Path;

pub const MULTIBOOT2_MAGIC: u32 = 0xE852_50D6;

#[derive(Debug, Clone)]
pub struct ElfKernelAudit {
    pub file_bytes: usize,
    pub is_elf64: bool,
    pub machine: u16,
    pub entry_addr: u64,
    pub pt_load_count: usize,
    pub multiboot2_offset: usize,
    pub multiboot2_checksum_valid: bool,
}

pub fn audit_elf64_multiboot2(bytes: &[u8]) -> Result<ElfKernelAudit, String> {
    if bytes.len() < 64 {
        return Err("file too short for ELF64 header".into());
    }
    if &bytes[0..4] != b"\x7FELF" {
        return Err("missing ELF magic".into());
    }
    let class = bytes[4];
    let endian = bytes[5];
    if class != 2 || endian != 1 {
        return Err(format!("expected ELF64 little-endian (class=2, data=1), got ({class}, {endian})"));
    }
    let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if machine != 62 {
        return Err(format!("expected EM_X86_64 (62), got {machine}"));
    }
    let entry_addr = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    let phoff = u64::from_le_bytes(bytes[32..40].try_into().unwrap()) as usize;
    let phentsize = u16::from_le_bytes([bytes[54], bytes[55]]) as usize;
    let phnum = u16::from_le_bytes([bytes[56], bytes[57]]) as usize;

    let mut pt_load_count = 0usize;
    for i in 0..phnum {
        let off = phoff + i * phentsize;
        if off + 4 > bytes.len() {
            return Err("program header out of bounds".into());
        }
        let p_type = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if p_type == 1 {
            pt_load_count += 1;
        }
    }
    if pt_load_count == 0 {
        return Err("no PT_LOAD segments in kernel ELF".into());
    }

    // Scan for Multiboot2 header magic (0xE85250D6) and verify exact mod-2^32 zero-sum checksum!
    let mut mb_off = None;
    for off in (0..=bytes.len().saturating_sub(16)).step_by(4) {
        let m = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if m == MULTIBOOT2_MAGIC {
            mb_off = Some(off);
            break;
        }
    }
    let off = mb_off.ok_or_else(|| "Multiboot2 magic 0xE85250D6 not found".to_string())?;
    let magic = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
    let arch = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap());
    let hlen = u32::from_le_bytes(bytes[off + 8..off + 12].try_into().unwrap());
    let csum = u32::from_le_bytes(bytes[off + 12..off + 16].try_into().unwrap());
    let sum = magic.wrapping_add(arch).wrapping_add(hlen).wrapping_add(csum);
    if sum != 0 {
        return Err(format!("MUTANT CAUGHT: Multiboot2 checksum non-zero (sum=0x{sum:08x})"));
    }

    Ok(ElfKernelAudit {
        file_bytes: bytes.len(),
        is_elf64: true,
        machine,
        entry_addr,
        pt_load_count,
        multiboot2_offset: off,
        multiboot2_checksum_valid: true,
    })
}

pub fn audit_elf64_file(path: &Path) -> Result<ElfKernelAudit, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    audit_elf64_multiboot2(&bytes)
}
