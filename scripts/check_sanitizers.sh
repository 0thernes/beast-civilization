#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Copyright (c) 2026 Alexander Stephen Donahue
# Compile and execute C11 sanitizer (-fsanitize=address,undefined) and optional
# Clang libFuzzer (-fsanitize=fuzzer,address,undefined) smoke harness (Build Standard B15/B16).
set -euo pipefail

CC="${CC:-cc}"
OUT="${TMPDIR:-/tmp}/beast_asan_smoke"

echo "[SAN] Compiling C11 ASan+UBSan smoke harness (-Wall -Wextra -Werror -fsanitize=address,undefined)..."
cat > "${OUT}.c" <<'EOF'
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

static uint8_t pack5(const int8_t t[5]) {
    uint32_t acc = 0, p = 1;
    for (int i = 0; i < 5; i++) {
        acc += (uint32_t)(t[i] + 1) * p;
        p *= 3u;
    }
    return (uint8_t)acc;
}

static void unpack5(uint8_t code, int8_t out[5]) {
    uint32_t x = code;
    for (int k = 0; k < 5; k++) {
        out[k] = (int8_t)((x % 3u) - 1u);
        x /= 3u;
    }
}

#ifdef BEAST_LIBFUZZER
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    if (size == 0) return 0;
    /* Fuzz b1.58 pack/unpack invariants and GF(2^8) XOR involution */
    uint8_t code = (uint8_t)(data[0] % 243u);
    int8_t trits[5];
    unpack5(code, trits);
    if (pack5(trits) != code) __builtin_trap();
    for (size_t i = 0; i + 1 < size; i += 2) {
        uint8_t a = data[i], b = data[i + 1];
        if ((uint8_t)((a ^ b) ^ b) != a) __builtin_trap();
    }
    return 0;
}
#else
int main(void) {
    int checked = 0;
    for (int a = -1; a <= 1; a++)
      for (int b = -1; b <= 1; b++)
        for (int c = -1; c <= 1; c++)
          for (int d = -1; d <= 1; d++)
            for (int e = -1; e <= 1; e++) {
                int8_t t[5] = {(int8_t)a, (int8_t)b, (int8_t)c, (int8_t)d, (int8_t)e};
                uint8_t code = pack5(t);
                if (code > 242) return 2;
                int8_t u[5];
                unpack5(code, u);
                for (int k = 0; k < 5; k++) {
                    if (u[k] != t[k]) return 3;
                }
                checked++;
            }
    if (checked != 243) return 4;

    uint8_t *a = (uint8_t *)malloc(256);
    uint8_t *b = (uint8_t *)malloc(256);
    uint8_t *p = (uint8_t *)malloc(256);
    if (!a || !b || !p) return 5;
    for (size_t i = 0; i < 256; i++) {
        a[i] = (uint8_t)i;
        b[i] = (uint8_t)(255u - i);
        p[i] = (uint8_t)(a[i] ^ b[i]);
    }
    for (size_t i = 0; i < 256; i++) {
        if ((uint8_t)(p[i] ^ b[i]) != a[i]) return 6;
    }
    free(a); free(b); free(p);
    printf("[PASS] C11 ASan+UBSan smoke: 243/243 b1.58 bijection + 256B GF(2^8) parity clean\n");
    return 0;
}
#endif
EOF

"$CC" -std=c11 -O1 -g -Wall -Wextra -Werror -fsanitize=address,undefined -fno-omit-frame-pointer "${OUT}.c" -o "$OUT"
"$OUT"

if command -v clang >/dev/null 2>&1; then
    echo "[FUZZ] Running Clang libfuzzer smoke pass (-fsanitize=fuzzer,address,undefined, 2000 runs)..."
    clang -std=c11 -O1 -g -DBEAST_LIBFUZZER -fsanitize=fuzzer,address,undefined -fno-omit-frame-pointer "${OUT}.c" -o "${OUT}_fuzz"
    "${OUT}_fuzz" -max_total_time=2 -runs=2000
    rm -f "${OUT}_fuzz"
fi

rm -f "${OUT}.c" "$OUT"
