module

public import BeastProofs.LiveReceiptData

/-!
# BeastProofs.Spec — Pure Shared Definitions Only (Zero Proofs, Zero `sorry`)

Canonical quantization, Cache-to-Cache, Ring-0 isolation, Colony governance,
cryptographic/physics, and runtime-receipt definitions shared identically by BOTH:
  - `Challenge.lean` (isolated statement-only specification — imports ONLY this module)
  - `BeastProofs/*.lean` & `Solution.lean` (sorry-free proofs checked across 6 kernels)

Pure Lean 4.35.0 `module` (zero Mathlib, zero `sorry`, zero custom axioms).
Copyright (c) 2026 Alexander Stephen Donahue <0_0@0thernes.art>.
SPDX-License-Identifier: Apache-2.0
-/

namespace BeastProofs

/-- Pack 5 ternary digits in `{0, 1, 2}` (representing `{-1, 0, +1}`) into base-3 integer `< 243`. -/
@[expose] public def pack5Trits (d0 d1 d2 d3 d4 : Fin 3) : Nat :=
  d0.val + 3 * (d1.val + 3 * (d2.val + 3 * (d3.val + 3 * d4.val)))

/-- Inverse base-3 unpacking of a packed 5-trit byte. -/
@[expose] public def unpack5Trits (c : Nat) : Nat × Nat × Nat × Nat × Nat :=
  let d0 := c % 3
  let c1 := c / 3
  let d1 := c1 % 3
  let c2 := c1 / 3
  let d2 := c2 % 3
  let c3 := c2 / 3
  let d3 := c3 % 3
  let d4 := (c3 / 3) % 3
  (d0, d1, d2, d3, d4)

/-- C2C carrier modes matching `crates/beast-c2c/src/lib.rs`. -/
public inductive CarrierMode where
  | augmentKv | replaceActivation | cacheGraft | textCloudFallback
  deriving DecidableEq

/-- Anti-LARP honesty predicate: cloud text fallback may never claim `isLocalC2c = true`. -/
@[expose] public def packetHonest (mode : CarrierMode) (isLocalC2c : Bool) : Bool :=
  match mode with
  | CarrierMode.textCloudFallback => !isLocalC2c
  | _ => true

/-- Abstract Ring-0 guardian state (`crates/beast-root/src/lib.rs`). -/
public structure RootState where
  protectedRootHash : Nat
  liveBodyHash : Nat
  spendCap : Nat
  spent : Nat
  deriving DecidableEq

/-- Gated self-rewrite transition preserving Ring-0 immutability. -/
@[expose] public def applyRewrite (s : RootState) (touchesRing0 : Bool) (gatesPassed : Bool) (newBodyHash : Nat) : RootState :=
  if touchesRing0 || !gatesPassed then s else { s with liveBodyHash := newBodyHash }

/-- NonHuman prefix cost in basis points (`P * (125 + 5 * N)`). -/
@[expose] public def nonhumanPrefixCostBp (P N : Nat) : Nat := P * (125 + 5 * N)

/-- Fibonacci fusion Hilbert-space dimension sequence (`tsotchke/spin_based_neural_network`). -/
@[expose] public def fibo : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fibo (n + 1) + fibo n

/-- Integer dual numbers for exact forward-mode automatic differentiation (`eshkol_ad.rs`). -/
public structure DualInt where
  val : Int
  der : Int
  deriving DecidableEq

/-- Dual number multiplication satisfying the exact Leibniz product rule. -/
@[expose] public def dualMul (x y : DualInt) : DualInt :=
  { val := x.val * y.val, der := x.der * y.val + x.val * y.der }

/-- Dual number addition. -/
@[expose] public def dualAdd (x y : DualInt) : DualInt :=
  { val := x.val + y.val, der := x.der + y.der }

/-- Ring-0 HTTP/1.1 LLM endpoint loopback SSRF guard (`crates/beast-agent/src/llm_client.rs`). -/
@[expose] public def allowHttpHost (isLoopback allowRemote : Bool) : Bool :=
  isLoopback || allowRemote

end BeastProofs
