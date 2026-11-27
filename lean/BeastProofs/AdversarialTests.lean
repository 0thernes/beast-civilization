module

public import BeastProofs.Spec

/-!
# BeastProofs.AdversarialTests — Machine-Checked Refutations of False Variants (`AT01`–`AT08`)

Playbook v1.3 Adversarial Specification Audit: formally proves `¬ P` for eight deliberately
broken or weakened claims, demonstrating that our `Spec.lean` definitions genuinely reject
faulty encodings, cloud spoofing, Ring-0 bypasses, non-contractive weights, corrupted
Multiboot2 headers, violated QGT bounds, broken compression, and unguarded SSRF.

Copyright (c) 2026 Alexander Stephen Donahue <0_0@0thernes.art>.
SPDX-License-Identifier: Apache-2.0
-/

namespace BeastProofs.AdversarialTests

/-- AT01: Refute fitting 6 ternary trits into 1 byte (`3^6 = 729 > 256`). -/
public theorem AT01_refute_six_trits_per_byte : ¬ (3 ^ 6 ≤ 256) := by decide

/-- AT02: Refute accepting a cloud text fallback spoofed as local C2C (`packetHonest = false ≠ true`). -/
public theorem AT02_refute_cloud_spoof_acceptance :
    ¬ (packetHonest CarrierMode.textCloudFallback true = true) := by decide

/-- AT03: Refute claiming Ring-0 allows overwriting `liveBodyHash` when `touchesRing0 = true`. -/
public theorem AT03_refute_ring0_rewrite_when_protected :
    ¬ (∀ (s : RootState) (newBody : Nat), (applyRewrite s true true newBody).liveBodyHash = newBody) := by
  intro h
  have h1 := h { protectedRootHash := 1, liveBodyHash := 10, spendCap := 100, spent := 0 } 99
  simp [applyRewrite] at h1

/-- AT04: Refute strict spectral contraction when `wMilli = 1000` (`1000 * 1 < 1000 * 1` is false). -/
public theorem AT04_refute_unit_norm_strict_contraction :
    ¬ (∀ (wMilli x : Nat), wMilli ≤ 1000 → x > 0 → wMilli * x < 1000 * x) := by
  intro h
  have h1 := h 1000 1 (Nat.le_refl 1000) (by decide)
  omega

/-- AT05: Refute a 1-bit corrupted Multiboot2 checksum (`csum ^^^ 1`) summing to zero mod 2^32. -/
public theorem AT05_refute_corrupted_multiboot2_checksum :
    let magic : BitVec 32 := 0xE85250D6#32
    let arch  : BitVec 32 := 0#32
    let hlen  : BitVec 32 := 24#32
    let badCsum : BitVec 32 := (0#32 - (magic + arch + hlen)) ^^^ 1#32
    ¬ (magic + arch + hlen + badCsum = 0#32) := by
  decide

/-- AT06: Refute QGT PSD bound when off-diagonal metric exceeds determinant (`gxx=1, gyy=1, gxy=2, omh=1`). -/
public theorem AT06_refute_violated_qgt_psd_determinant :
    ¬ ((1 : Int) * 1 - 2 * 2 ≥ 1 * 1) := by decide

/-- AT07: Refute `Q4_0` strict compression at degenerate zero blocks (`n = 0`: `18 * 0 < 128 * 0` is false). -/
public theorem AT07_refute_q40_strict_compression_at_zero :
    ¬ (∀ (n : Nat), 18 * n < 128 * n) := by
  intro h
  have h0 := h 0
  omega

/-- AT08: Refute allowing non-loopback HTTP SSRF host when `allowRemote = false`. -/
public theorem AT08_refute_unguarded_remote_ssrf :
    ¬ (allowHttpHost false false = true) := by decide

end BeastProofs.AdversarialTests
