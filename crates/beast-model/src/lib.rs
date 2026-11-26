//! `beast-model`: Real zero-dependency GGUF v3 binary codec, cross-architecture
//! Transformer -> Mamba/sLTC/b1.58 Chimera C2C engine, and real bare-metal
//! Audio (TTS `.wav` + FFT/Mel ASR) & Vision/OCR (`.pgm` + Sobel + `b1.58` glyph decode) kernels.

pub mod gguf;
pub mod omni_engine;
pub mod omnimodal;
