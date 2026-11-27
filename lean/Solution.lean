module

public import BeastProofs

/-!
# BEAST CIVILIZATION v1.0.0 Formal Solution Entrypoint (`Solution.lean`)

Exports all 81 machine-checked theorems from `BeastProofs` (`B01`–`B65`, `NV01`–`NV08`,
`AT01`–`AT08` across 9 proof modules + pure `Spec.lean`, with `autoImplicit = false`,
`relaxedAutoImplicit = false`, zero `sorry`, zero `native_decide`) for isolated judging
against `Challenge.lean` via `lake comparator --config comparator.json --paranoid`.

Copyright (c) 2026 Alexander Stephen Donahue <0_0@0thernes.art>.
SPDX-License-Identifier: Apache-2.0
-/
