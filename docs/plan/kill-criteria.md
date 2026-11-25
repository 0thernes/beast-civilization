# P0.07 — Pre-Registered Kill Criteria (`beast-civilization`)

Any subsystem violating a kill criterion is cut, simplified, or redesigned rather than masked with stubs or relaxed tests (`AI13`):

1. **KC-1 (Buildability):** Clean build of the active slice exceeds 5 minutes or requires manual patching on Windows x64.
2. **KC-2 (Polyglot FFI Latency):** Hot-path dispatch exceeds `10 µs` (`Rust ↔ Zig ↔ C11` target `< 50 ns`; `CRuby/YARV ↔ Zig` target `< 2 µs`).
3. **KC-3 (C2C Fidelity & Speed):** Local `SwaT` / `Augment-Cache` fails to beat text-to-text latency or loses `> 5%` accuracy on controlled multi-agent tasks.
4. **KC-4 (Insert-Network Evolution):** Evolved mixed-precision chimeras fail to beat matched-parameter random wiring after 100 generations on real benchmarks.
5. **KC-5 (Anti-LARP Integrity):** Any planted mutant (leaky ledger, fake C2C text fallback disguised as KV transfer, spectral overflow, Ring-0 bypass, corrupted binary, sycophantic judge) escapes detection.
