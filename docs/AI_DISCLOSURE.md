<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# AI-Assistance, Authorship & Independent Verification Disclosure (`A40` / Playbook v1.3)

- **Sole Author & Responsible Maintainer:** Alexander Stephen Donahue (Independent Researcher, Spokane, WA, USA · `0_0@0thernes.art` · ORCID [`0009-0006-8168-9648`](https://orcid.org/0009-0006-8168-9648))
- **Role of AI Assistants During Development:** AI coding and proof-engineering assistants (Gemini / Antigravity) assisted under direct human architectural direction with software engineering (Rust, Zig, C11 bridges, CRuby/Prism, CUDA PTX, Python tooling), Lean 4.35.0 Palomar refactoring, multi-cycle red-team auditing, and AMS-LaTeX typesetting.
- **No Unverified AI Authority:** No scientific, mathematical, or empirical claim in this repository rests on LLM output trust:
  1. Every formal theorem (`81/81`) is checked mechanically by **6 independent Lean 4.35.0 proof kernels** (`lean4`, `leanchecker`, `leanchecker-paranoid`, `lake check --paranoid`, `lake comparator --paranoid`) under sandboxed `bubblewrap` in CI, with live negative tamper rejection.
  2. Every runtime claim is checked by deterministic executable gates (`33/33`), planted negative mutants (`40/40`), a `1,000`-case binary/protocol fuzz suite, a clean-room stdlib-only Python exact-arithmetic oracle, and pre-registered $N=24$ Holm-Bonferroni statistics.
  3. Every bibliographic DOI is checked live against the Crossref API (`results/citation-audit.json`, `0` retractions).
- **Runtime AI Components:** `beast-civilization` ships **zero third-party cloud LLM SDKs or opaque external weights**; `beast serve` executes deterministic bare-metal local weights, and `beast agent-run --endpoint` only connects to user-configured endpoints under strict Ring-0 loopback SSRF enforcement (`B62`) and micro-USD budget metering (`B10`).
