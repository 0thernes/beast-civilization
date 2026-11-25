# Security Policy (`A16`, `B36`)

## Threat Model Summary
`beast-civilization` executes self-rewriting code (`T0–T3`), live YARV bytecode mutations (`T1a`), arbitrary-precision native/GPU kernels, and links bare-metal RWX JIT engines (`AUTOPHAGE`).

## Ring-0 Security Boundaries
1. **Zero Keys in Agent Zone:** API keys, signing keys, and spend budgets live exclusively inside the separate `beast-root` Ring-0 process (modeled after `nagisanzenin/INIT-Agent` and `0thernes/noetic-organism-omega`).
2. **Cryptographic Lineage:** Every self-modification produces a SHA-256 / Ed25519 signed receipt verified against Gates `G0–G4` before hot-swap.
3. **Spectral & Resource Caps:** Mutated hybrid neural networks enforce `‖W‖₂ ≤ 0.99`, zero-NaN verification, and bounded FLOP/memory arenas.
4. **No Committed Secrets (`A24`, `AI10`):** Secrets live solely in local environment variables or OS credential stores (see `.env.example`).

## Reporting
Report vulnerabilities directly to `0_0@0thernes.art`.
