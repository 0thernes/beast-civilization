//! NonHuman cost-accounting, EffortMining classifier, Nagi 1-pass typed router,
//! 8-Architecture / 20-Domain Omnimodal matrix, and Public Open-Source API registry.

/// Exact NonHuman cost accounting (`nagisanzenin/nonhuman`):
/// `C = P*(1.25*u + 0.05*N) + sum_i delta_i*(1.25 + 0.05*(N - i)) + 5 * sum_i (o_i + h_i)`
/// where `i` is 1-indexed (`1..=N`).
pub fn nonhuman_run_cost(
    prefix_tokens: f64,
    cache_writes_u: f64,
    deltas: &[f64],
    outputs: &[f64],
    thinking: &[f64],
) -> f64 {
    let n = deltas.len().max(outputs.len()).max(thinking.len());
    let nf = n as f64;
    let prefix_cost = prefix_tokens * (1.25 * cache_writes_u + 0.05 * nf);
    let mut ctx_growth = 0.0;
    for (idx, &d) in deltas.iter().enumerate() {
        let i = (idx + 1) as f64;
        ctx_growth += d * (1.25 + 0.05 * (nf - i));
    }
    let mut gen_cost = 0.0;
    for idx in 0..n {
        let o = outputs.get(idx).copied().unwrap_or(0.0);
        let h = thinking.get(idx).copied().unwrap_or(0.0);
        gen_cost += 5.0 * (o + h);
    }
    prefix_cost + ctx_growth + gen_cost
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffortTier {
    ReflexLocalB158,
    LowLocal4B,
    MediumHybrid,
    HighFrontier,
    MaxFormalProof,
}

/// EffortMining (`nagisanzenin/effortmining`) subtask classifier.
pub fn classify_effort_tier(complexity_0_100: u8, requires_formal_proof: bool, mutates_kernel: bool) -> EffortTier {
    if requires_formal_proof {
        EffortTier::MaxFormalProof
    } else if mutates_kernel || complexity_0_100 >= 80 {
        EffortTier::HighFrontier
    } else if complexity_0_100 >= 50 {
        EffortTier::MediumHybrid
    } else if complexity_0_100 >= 20 {
        EffortTier::LowLocal4B
    } else {
        EffortTier::ReflexLocalB158
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchFamily {
    DenseOrMoeTransformer,
    StateSpaceMambaRwkv,
    TernaryBitLevelB158,
    LiquidOdePinnSltc,
    EquivariantGeometricQgtl,
    VisionOcrEncoderDecoder,
    AudioAsrTtsConformer,
    GeneticMachineCodeAutophage,
}

#[derive(Debug, Clone, Copy)]
pub struct DomainRoute {
    pub domain: &'static str,
    pub primary_arch: ArchFamily,
    pub precision_hint: &'static str,
    pub c2c_capable_local: bool,
}

/// The 8-Architecture / 20-Domain Omnimodal Nagi Routing Table.
pub const OMNIMODAL_ROUTES_20: [DomainRoute; 20] = [
    DomainRoute { domain: "systems_code_rust_zig", primary_arch: ArchFamily::DenseOrMoeTransformer, precision_hint: "NVFP4", c2c_capable_local: true },
    DomainRoute { domain: "os_kernel_freestanding", primary_arch: ArchFamily::DenseOrMoeTransformer, precision_hint: "FP8", c2c_capable_local: true },
    DomainRoute { domain: "asm_binary_reverse", primary_arch: ArchFamily::StateSpaceMambaRwkv, precision_hint: "b1.58", c2c_capable_local: true },
    DomainRoute { domain: "long_repo_log_stream", primary_arch: ArchFamily::StateSpaceMambaRwkv, precision_hint: "INT4", c2c_capable_local: true },
    DomainRoute { domain: "typed_1pass_routing", primary_arch: ArchFamily::TernaryBitLevelB158, precision_hint: "b1.58", c2c_capable_local: true },
    DomainRoute { domain: "ast_reflex_filter", primary_arch: ArchFamily::TernaryBitLevelB158, precision_hint: "u1", c2c_capable_local: true },
    DomainRoute { domain: "adaptive_margin_control", primary_arch: ArchFamily::LiquidOdePinnSltc, precision_hint: "FP64", c2c_capable_local: true },
    DomainRoute { domain: "pde_physics_simulation", primary_arch: ArchFamily::LiquidOdePinnSltc, precision_hint: "FP64", c2c_capable_local: true },
    DomainRoute { domain: "quantum_geometry_qgtl", primary_arch: ArchFamily::EquivariantGeometricQgtl, precision_hint: "C64", c2c_capable_local: true },
    DomainRoute { domain: "3d_se3_equivariant", primary_arch: ArchFamily::EquivariantGeometricQgtl, precision_hint: "FP32", c2c_capable_local: true },
    DomainRoute { domain: "cfg_callgraph_gnn", primary_arch: ArchFamily::EquivariantGeometricQgtl, precision_hint: "FP16", c2c_capable_local: true },
    DomainRoute { domain: "ocr_pdf_latex_doc", primary_arch: ArchFamily::VisionOcrEncoderDecoder, precision_hint: "NVFP4", c2c_capable_local: true },
    DomainRoute { domain: "schematic_waveform_vision", primary_arch: ArchFamily::VisionOcrEncoderDecoder, precision_hint: "FP8", c2c_capable_local: true },
    DomainRoute { domain: "video_temporal_frames", primary_arch: ArchFamily::VisionOcrEncoderDecoder, precision_hint: "NVFP4", c2c_capable_local: true },
    DomainRoute { domain: "asr_stt_transcription", primary_arch: ArchFamily::AudioAsrTtsConformer, precision_hint: "FP16", c2c_capable_local: true },
    DomainRoute { domain: "tts_neural_synthesis", primary_arch: ArchFamily::AudioAsrTtsConformer, precision_hint: "FP16", c2c_capable_local: true },
    DomainRoute { domain: "wakeword_dsp_acoustics", primary_arch: ArchFamily::AudioAsrTtsConformer, precision_hint: "INT8", c2c_capable_local: true },
    DomainRoute { domain: "formal_math_lean4", primary_arch: ArchFamily::DenseOrMoeTransformer, precision_hint: "FP8", c2c_capable_local: true },
    DomainRoute { domain: "alife_rwx_jit_evolution", primary_arch: ArchFamily::GeneticMachineCodeAutophage, precision_hint: "GF2_8", c2c_capable_local: true },
    DomainRoute { domain: "chimera_insert_nas", primary_arch: ArchFamily::GeneticMachineCodeAutophage, precision_hint: "MIXED", c2c_capable_local: true },
];

#[derive(Debug, Clone, Copy)]
pub struct PublicOssApi {
    pub id: &'static str,
    pub endpoint: &'static str,
    pub domain: &'static str,
    pub auth_required: bool,
}

/// Baked-in Public Open-Source Scientific & Bare-Metal Systems API Registry (zero vendor lock-in).
pub const PUBLIC_OSS_APIS: [PublicOssApi; 12] = [
    PublicOssApi { id: "arxiv", endpoint: "https://export.arxiv.org/api/query", domain: "papers_preprints", auth_required: false },
    PublicOssApi { id: "openalex", endpoint: "https://api.openalex.org/works", domain: "scholarly_graph", auth_required: false },
    PublicOssApi { id: "semantic_scholar", endpoint: "https://api.semanticscholar.org/graph/v1", domain: "citations", auth_required: false },
    PublicOssApi { id: "godbolt", endpoint: "https://godbolt.org/api/compilers", domain: "compiler_disassembly", auth_required: false },
    PublicOssApi { id: "osv", endpoint: "https://api.osv.dev/v1/query", domain: "cve_security", auth_required: false },
    PublicOssApi { id: "crates_io", endpoint: "https://crates.io/api/v1/crates", domain: "rust_packages", auth_required: false },
    PublicOssApi { id: "pypi", endpoint: "https://pypi.org/pypi", domain: "python_packages", auth_required: false },
    PublicOssApi { id: "huggingface", endpoint: "https://huggingface.co/api/models", domain: "open_weights", auth_required: false },
    PublicOssApi { id: "github_public", endpoint: "https://api.github.com", domain: "source_repos", auth_required: false },
    PublicOssApi { id: "wikidata", endpoint: "https://www.wikidata.org/w/api.php", domain: "structured_knowledge", auth_required: false },
    PublicOssApi { id: "europepmc", endpoint: "https://www.ebi.ac.uk/europepmc/webservices/rest", domain: "biomed_fulltext", auth_required: false },
    PublicOssApi { id: "pci_ids", endpoint: "https://pci-ids.ucw.cz/v2.2/pci.ids", domain: "bare_metal_hardware", auth_required: false },
];

pub fn route_domain(query_domain: &str) -> DomainRoute {
    for r in &OMNIMODAL_ROUTES_20 {
        if r.domain == query_domain {
            return *r;
        }
    }
    OMNIMODAL_ROUTES_20[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonhuman_fewer_calls_strictly_cheaper() {
        let c_unbatched = nonhuman_run_cost(10_000.0, 1.0, &[1000.0; 14], &[400.0; 14], &[200.0; 14]);
        let c_batched = nonhuman_run_cost(10_000.0, 1.0, &[1000.0; 10], &[400.0; 10], &[200.0; 10]);
        assert!(c_batched < c_unbatched);
        assert_eq!(OMNIMODAL_ROUTES_20.len(), 20);
        assert_eq!(PUBLIC_OSS_APIS.len(), 12);
    }
}
