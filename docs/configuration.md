<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Environment & Configuration Reference (`A33`)

All environment variables are optional; the full verification suite runs **100% offline** with default settings. See [`.env.example`](../.env.example) for a copyable template.

| Variable | Default | Scope | Purpose |
|---|---|---|---|
| `BEAST_API_KEY` | *(unset)* | `beast agent-run --endpoint` | Optional Bearer token sent to a user-specified OpenAI-compatible endpoint. **Scrubbed automatically** before spawning any Ring-0 sandboxed child process (`G15`, `G23`). |
| `BEAST_ZONE` | `AGENT_SANDBOX_NO_SECRETS` | Sandboxed child processes | Diagnostic marker injected by `Ring0Root::build_sandboxed_command`. |
| `BEAST_P9_TELEMETRY` | `0` / `1` | Forged A-Life executable | Toggles read-only telemetry mode to verify P9 observer independence (`0`-bit state delta). |
| `TECTONIC_BIN` | `tectonic` / `~/.local/bin/tectonic` | `scripts/build_paper.py` | Optional explicit path to the Tectonic TeX engine. |
| `BEAST_BIN` | `bin/beast.exe` / `target/release/beast` | `scripts/verify_stats.py` | Optional override for the compiled `beast` CLI binary. |

## Security Invariants
- Never commit `.env` files (ignored via `.gitignore` and audited by `gitleaks`).
- `beast agent-run --endpoint` blocks all non-loopback hosts unless `--allow-remote` is passed explicitly (`Ring-0 SSRF guard`, `B62`).
- `beast serve` binds to `127.0.0.1` by default; expose externally only behind an authenticated reverse proxy.
