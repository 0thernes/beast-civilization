#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
"""Runnable Quickstart Example (Build Standard A43):
Demonstrates exact BitNet b1.58 5-trit/byte packing, Q4_0 block compression,
and MIT Augment-Cache vs. Overwrite retention ratio using zero dependencies."""

from __future__ import annotations
from fractions import Fraction


def pack_5_trits(trits: tuple[int, int, int, int, int]) -> int:
    code = 0
    for k, t in enumerate(trits):
        assert t in (-1, 0, 1)
        code += (t + 1) * (3**k)
    return code


def unpack_5_trits(code: int) -> tuple[int, int, int, int, int]:
    assert 0 <= code < 243
    out = []
    for _ in range(5):
        out.append((code % 3) - 1)
        code //= 3
    return tuple(out)  # type: ignore[return-value]


def main() -> None:
    sample = (1, -1, 0, 1, 1)
    packed = pack_5_trits(sample)
    unpacked = unpack_5_trits(packed)
    assert unpacked == sample
    print(f"b1.58 pack/unpack : {sample} -> byte {packed} -> {unpacked} (exact)")

    q4_ratio = Fraction(32 * 4, 2 + 16)
    print(f"GGUF Q4_0 vs F32  : {q4_ratio} = {float(q4_ratio):.4f}x memory compression")

    self_len, peer_len = 24, 12
    aug = Fraction(self_len + peer_len, self_len + peer_len)
    ovr = Fraction(peer_len, self_len + peer_len)
    print(f"MIT C2C Retention : Augment={aug} ({float(aug)*100:.1f}%) > Overwrite={ovr} ({float(ovr)*100:.1f}%)")


if __name__ == "__main__":
    main()
