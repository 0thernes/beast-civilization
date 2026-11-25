/*
 * Live C-ABI Bridge to upstream `tsotchke/quantum_rng` (compiled via `zig cc` on Win64/Linux).
 * Executes Tsotchke's genuine state-vector `create_bell_state_phi_plus` + `bell_test_chsh`
 * using a deterministic SplitMix64/Philox-seeded callback for reproducible Track-A verification.
 */

#include <stdint.h>
#include <stddef.h>
#include "bell_test.h"

#if defined(_WIN32)
  #define BEAST_TSO_EXPORT __declspec(dllexport)
#else
  #define BEAST_TSO_EXPORT __attribute__((visibility("default")))
#endif

typedef struct {
    uint64_t state;
} BeastDetRng;

static int beast_det_entropy_cb(void *user_data, uint8_t *buf, size_t size) {
    BeastDetRng *rng = (BeastDetRng *)user_data;
    size_t i = 0;
    while (i < size) {
        rng->state += 0x9e3779b97f4a7c15ULL;
        uint64_t z = rng->state;
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        z = z ^ (z >> 31);
        for (int b = 0; b < 8 && i < size; b++, i++) {
            buf[i] = (uint8_t)((z >> (b * 8)) & 0xFFu);
        }
    }
    return 0;
}

BEAST_TSO_EXPORT double beast_tsotchke_run_bell_chsh(uint64_t seed, size_t shots, int *out_violates) {
    quantum_state_t st;
    if (quantum_state_init(&st, 2) != QS_SUCCESS) {
        if (out_violates) *out_violates = 0;
        return -1.0;
    }
    BeastDetRng rng = { seed ^ 0x54534F5443484B45ULL };
    quantum_entropy_ctx_t ectx;
    quantum_entropy_init(&ectx, beast_det_entropy_cb, &rng);

    bell_test_result_t res = bell_test_chsh(&st, 0, 1, shots, NULL, &ectx);
    quantum_state_free(&st);
    if (out_violates) *out_violates = res.violates_classical;
    return res.chsh_value;
}
