//! `beast-model`: Real zero-dependency GGUF v3 binary codec, cross-architecture
//! Transformer -> Mamba/sLTC/b1.58 Chimera C2C engine, 4-Layer Autoregressive GQA/RoPE/SwiGLU
//! LLM C2C engine, Eshkol-style Forward/Reverse/Symbolic AD, and real Audio/Vision/OCR DSP kernels.

pub mod eshkol_ad;
pub mod gguf;
pub mod llm_engine;
pub mod llm_server;
pub mod omni_engine;
pub mod omnimodal;
