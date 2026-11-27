<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# ADR-0002: Non-Destructive MIT Augment-Cache Dominance Over Destructive Overwrite

- **Status:** Accepted (`v1.0.0`, 2026-11-26)
- **Context:** When transferring latent KV-caches between heterogeneous model families (`SwaT` / `C2C`), overwriting the receiver's local key-value cache destroys local working context, whereas concatenating mapped peer KV states preserves local keys while enriching attention support.
- **Decision:** Enforce `CarrierMode::AugmentKv` (`[K_self || K_peer]`, length $L_s + L_p$) as the mandatory KV-cache transfer mode across Rust (`beast-c2c`), Zig (`beast_c2c_augment_attn_f32`), and Blackwell PTX (`beast_c2c_augment_scores`), while reserving replacement exclusively for hidden activation streams (`CarrierMode::ReplaceActivation`) and prohibiting cloud text fallbacks from spoofing `is_local_c2c = 1` (`Bc2cPacketHeader::validate`).
- **Verification:** Machine-checked in Lean (`B06`, `B07`, `B08`, `B21`, `NV02`, `AT02`), verified bit-exact Rust vs. Zig (`G03`), and statistically validated across $N=24$ paired seeds (`H1`, $p_{\mathrm{Holm}} \le 0.05$) in `results/paired_stats_n24.txt`.
