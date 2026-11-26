//! `beast-model`: Real zero-dependency GGUF v3 binary codec, cross-architecture
//! Transformer -> Mamba/sLTC/b1.58 Chimera C2C engine, 4-Layer Autoregressive GQA/RoPE/SwiGLU
//! LLM C2C engine, and real bare-metal Audio (TTS `.wav` + DFT ASR) & Vision/OCR (`.pgm` + Sobel + `b1.58`) kernels.

pub mod gguf;
pub mod llm_engine;
pub mod omni_engine;
pub mod omnimodal;
