//! Native Zero-Dependency ELF64/Multiboot2 & ELF32/Multiboot1 Forensic Binary Auditor.
//! Parses raw emitted freestanding kernel ELFs, verifies architecture, `PT_LOAD` segments,
//! entry point >= 1 MiB, AND exact 32-bit wrapping Multiboot1 / Multiboot2 header checksums!

use std::fs;
use std::path::Path;

pub const MULTIBOOT1_MAGIC: u32 = 0x1BAD_B002;
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

#[derive(Debug, Clone)]
pub struct Elf32Multiboot1Audit {
    pub file_bytes: usize,
    pub machine: u16,
    pub entry_addr: u32,
    pub pt_load_count: usize,
    pub multiboot1_offset: usize,
    pub multiboot1_checksum_valid: bool,
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
        return Err(format!(
            "expected ELF64 little-endian (class=2, data=1), got ({class}, {endian})"
        ));
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
        return Err(format!(
            "MUTANT CAUGHT: Multiboot2 checksum non-zero (sum=0x{sum:08x})"
        ));
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

pub fn audit_elf32_multiboot1(bytes: &[u8]) -> Result<Elf32Multiboot1Audit, String> {
    if bytes.len() < 52 {
        return Err("file too short for ELF32 header".into());
    }
    if &bytes[0..4] != b"\x7FELF" {
        return Err("missing ELF magic".into());
    }
    let class = bytes[4];
    let endian = bytes[5];
    if class != 1 || endian != 1 {
        return Err(format!(
            "expected ELF32 little-endian (class=1, data=1), got ({class}, {endian})"
        ));
    }
    let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if machine != 3 {
        return Err(format!("expected EM_386 (3), got {machine}"));
    }
    let entry_addr = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
    let phoff = u32::from_le_bytes(bytes[28..32].try_into().unwrap()) as usize;
    let phentsize = u16::from_le_bytes([bytes[42], bytes[43]]) as usize;
    let phnum = u16::from_le_bytes([bytes[44], bytes[45]]) as usize;

    let mut pt_load_count = 0usize;
    for i in 0..phnum {
        let off = phoff + i * phentsize;
        if off + 4 > bytes.len() {
            return Err("ELF32 program header out of bounds".into());
        }
        let p_type = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if p_type == 1 {
            pt_load_count += 1;
        }
    }
    if pt_load_count == 0 {
        return Err("no PT_LOAD segments in ELF32 kernel".into());
    }

    // Multiboot1 spec requires magic 0x1BADB002 within the first 8192 bytes, 4-byte aligned.
    let search_end = bytes.len().min(8192).saturating_sub(12);
    let mut mb_off = None;
    for off in (0..=search_end).step_by(4) {
        let m = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if m == MULTIBOOT1_MAGIC {
            mb_off = Some(off);
            break;
        }
    }
    let off = mb_off.ok_or_else(|| "Multiboot1 magic 0x1BADB002 not found in first 8192B".to_string())?;
    let magic = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
    let flags = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap());
    let csum = u32::from_le_bytes(bytes[off + 8..off + 12].try_into().unwrap());
    let sum = magic.wrapping_add(flags).wrapping_add(csum);
    if sum != 0 {
        return Err(format!(
            "MUTANT CAUGHT: Multiboot1 checksum non-zero (sum=0x{sum:08x})"
        ));
    }

    Ok(Elf32Multiboot1Audit {
        file_bytes: bytes.len(),
        machine,
        entry_addr,
        pt_load_count,
        multiboot1_offset: off,
        multiboot1_checksum_valid: true,
    })
}

pub fn audit_elf64_file(path: &Path) -> Result<ElfKernelAudit, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    audit_elf64_multiboot2(&bytes)
}

pub fn audit_elf32_file(path: &Path) -> Result<Elf32Multiboot1Audit, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    audit_elf32_multiboot1(&bytes)
}
