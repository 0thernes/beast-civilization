//! `beast-agent`: Autonomous Matrix Builder & Bare-Metal Colony Agent Engine (`v0.5.0`).
//! Combines:
//!   - `repo_map`: Merkle-hashed incremental AST/symbol extractor across `.rs`, `.zig`, `.c`, `.h`, `.rb`, `.lean`, `.ptx`
//!   - `forge`: Autonomous synthesis + compilation + QEMU Ring-0 live boot + Planaria self-healing of neural microkernels & Chimera GGUF models
//!   - `alife_forge`: Autonomous bare-metal Zig A-Life creature simulator synthesis, native compilation, P9 determinism & Planaria healing
//!   - `loop_engine`: Unified `INIT` x `NonHuman` x `Engram` x `TEMM1E` autonomous bare-metal agent loop with signed Rosette-Wire receipts

pub mod alife_forge;
pub mod forge;
pub mod loop_engine;
pub mod repo_map;
