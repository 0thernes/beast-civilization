module

public import BeastProofs.Spec
public import BeastProofs.Quantization
public import BeastProofs.CacheTelepathy
public import BeastProofs.Ring0Security
public import BeastProofs.ColonyGovernance
public import BeastProofs.CryptoPhysics
public import BeastProofs.SystemsRuntime
public import BeastProofs.LiveReceipt
public import BeastProofs.NonVacuity
public import BeastProofs.AdversarialTests

/-!
# BeastProofs (v1.0.0) — Root Re-Export of All 81 Pure Lean 4.35.0 Formal Theorems

Aggregates all nine sorry-free proof modules (65 core domain theorems `B01`–`B65`,
8 concrete non-vacuity witnesses `NV01`–`NV08`, and 8 adversarial false-variant
refutations `AT01`–`AT08`) over the shared pure specification `BeastProofs.Spec`.

Zero Mathlib, zero `sorry`, zero `native_decide`, zero `bv_decide`, `autoImplicit = false`.
Copyright (c) 2026 Alexander Stephen Donahue <0_0@0thernes.art>.
SPDX-License-Identifier: Apache-2.0
-/
