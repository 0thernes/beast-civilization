# Future Ideas Backlog (`AI02` Scope Lock)

Ideas parked here so active slices never suffer silent scope creep:
- Live QEMU automated boot-test loop verifying serial output of `zig/freestanding_kernel/beast_microkernel.elf`.
- Full Rust-to-`tsotchke/eshkol` LLVM JIT bridge once Eshkol Windows Clang 23 build script is validated.
- Full `tsotchke/moonlab` + `tsotchke/quantum_geometric_tensor` + `tsotchke/quantum_rng` native Win64 DLL builds linked alongside our in-tree C/Rust/Zig kernels.
- Live GGUF/safetensors weights loader for `Nagi-421M`, `TemRust-1.5B`, Whisper/Moonshine ASR, Kokoro TTS, and Florence-2 OCR over `beast-c2c` Blackwell VRAM arenas.
- Hardware `PLANARIA` `.hbin` capsule exporter wrapping compiled Zig/YARV colony workers into self-healing bisectable 96 KiB binaries.
