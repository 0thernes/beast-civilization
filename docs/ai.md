<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# AI Components, Model Pinning, Budgets & Evaluation Policy (`A40`, `B34`)

## 1. Local & Remote AI Component Pinning

`BEAST CIVILIZATION v1.0.0` ships with **zero mandatory cloud LLM dependencies** and **zero unpinned model weights**:

| Component | Implementation | Pinned Artifact / Architecture | Default Network Access |
|---|---|---|---|
| **Local 4-Layer C2C Chimera LLM** | `crates/beast-model/src/llm_engine.rs` | Deterministic 4-layer GQA (`32d`) $\to$ hybrid `sLTC`/`b1.58` Chimera (`24d`), serialized to `bin/beast_4layer_chimera.gguf` (`GGUF v3`, `Q4_0`/`F16`/`F32`) | None (100% local) |
| **Local OpenAI HTTP/1.1 Server** | `crates/beast-model/src/llm_server.rs` (`beast serve`) | Loopback-only (`127.0.0.1` / `localhost` / `[::1]`) `/v1/chat/completions`, `/v1/models`, `/healthz`; body cap `1 MiB` | Loopback only |
| **Optional Remote OpenAI Client** | `crates/beast-agent/src/llm_client.rs` | Explicitly opt-in via `--endpoint` + `BEAST_ALLOW_REMOTE_LLM=1`; Ring-0 SSRF guard blocks link-local (`169.254.169.254`), `0.0.0.0`, whitespace/CRLF injection, and non-loopback hosts by default (`B62`, `AT08`) | Blocked by default |
| **Omnimodal DSP Engines (OCR/ASR/TTS)** | `crates/beast-model/src/omnimodal.rs` | Zero-weight deterministic 2D Sobel+`b1.58` OCR, Goertzel DFT ASR, and 16 kHz PCM formant TTS (`G29`) | None (100% local) |

> **Honest Boundary (`F038`, `F068`, `F069`):** External multi-gigabyte checkpoints (HuggingFace, Whisper, Kokoro, TFLite) are intentionally not bundled; they remain honestly marked `SPEC` in `docs/matrices/300_agent_matrix.yaml`.

## 2. Hard Financial & Compute Budgets (`B10`, `B28`)

Every autonomous colony execution passes through the Ring-0 budget governor (`crates/beast-root/src/lib.rs`, `crates/beast-colony/src/economics.rs`):

- **Hard Micro-USD Spend Cap:** `RootState::spend_budget_micro_usd` enforces `spent + cost <= max_budget` (`B10`, `NV03`) with monotonic accounting; attempting to exceed the budget terminates the cycle cleanly with zero partial mutation.
- **Process Memory & Lifetime Budget:** Child compilers, QEMU microkernels, and forged binaries execute inside Win32 Kernel Job Objects (`128 MiB` hard memory cap, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, `G23`, `B43`) with secret environment variable scrubbing (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GITHUB_TOKEN`, etc., `B28`).
- **Token / Call Budget Reduction:** NonHuman prefix-caching economics (`crates/beast-colony/src/economics.rs`, `B11`) and MIT `SwaT` Cache-to-Cache projection (`G03`–`G04`) reduce redundant prompt prefill calls by `-25.5%` to `-28.8%` on deterministic workflows.

## 3. Automated Hallucination, Drift & Regression Evals (`B34`)

Run the deterministic local AI evaluation suite via:

```bash
python evals/eval_suite.py
```

The suite tests four failure modes without network calls:
1. **C2C Context Preservation Eval:** Ensures `Augment-Cache` preserves local self-context strictly above destructive `Overwrite-Cache` (`H1`).
2. **Ring-0 Safety & SSRF Eval:** Verifies `169.254.169.254`, uppercase `LOCALHOST` normalization edge cases, CRLF injection, and protected-path mutation attempts (`../`, `./`, `\`, case-variant) are rejected (`G02`, `G32`, `hardening_fuzz_props.rs`).
3. **GGUF & Prompt Protocol Fuzz Eval:** Exercises 1,000-case malformed binary/protocol inputs to guarantee zero panics or OOM crashes (`hardening_fuzz_props.rs`).
4. **Formal Proof & Citation Grounding Eval:** Verifies `81/81` Lean 4.35.0 theorems (`VerifyAxioms.lean`), `0` retracted/fabricated DOIs (`scripts/check_citations.py`), and `0/1,900` honest Mega-Atlas `v2.8.0` non-retirement accounting.
