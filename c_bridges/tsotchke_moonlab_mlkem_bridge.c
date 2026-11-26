/*
 * Live C-ABI Bridge to upstream `tsotchke/moonlab` FIPS-203 ML-KEM-512/768/1024 Post-Quantum Cryptography.
 * Compiled directly via `zig cc` into `bin/tsotchke_moonlab_mlkem_bridge.dll`.
 */

#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include "crypto/mlkem/mlkem.h"
#include "crypto/sha3/sha3.h"

#if defined(_WIN32)
  #define BEAST_MLKEM_EXPORT __declspec(dllexport)
#else
  #define BEAST_MLKEM_EXPORT __attribute__((visibility("default")))
#endif

/* Supply deterministic SHAKE256-backed entropy symbol required by mlkem.c's *_qrng wrappers
 * when compiling mlkem.c standalone without full OS QRNG driver. Core deterministic
 * keygen/encaps/decaps take explicit caller seeds directly. */
int moonlab_qrng_bytes(uint8_t *out, size_t len) {
    static uint64_t ctr = 0xB3A5700000000001ULL;
    uint8_t seed[8];
    ctr++;
    for (int i = 0; i < 8; i++) seed[i] = (uint8_t)((ctr >> (i * 8)) & 0xFF);
    shake256(seed, sizeof(seed), out, len);
    return 0;
}

/* Executes full FIPS-203 ML-KEM-768 KeyGen -> Sec 7.2/7.3 validation -> Encaps -> Decaps ->
 * IND-CCA2 Fujisaki-Okamoto tampered-ciphertext implicit-rejection test!
 * Writes the 32-byte shared secret to `out_shared_secret` on success. */
BEAST_MLKEM_EXPORT int beast_tsotchke_mlkem768_verify(
    uint64_t seed,
    uint8_t out_shared_secret[32],
    int *out_fo_tamper_rejected
) {
    uint8_t seed_buf[8];
    for (int i = 0; i < 8; i++) seed_buf[i] = (uint8_t)((seed >> (i * 8)) & 0xFF);

    uint8_t d[32], z[32], m[32];
    uint8_t expanded[96];
    shake256(seed_buf, sizeof(seed_buf), expanded, sizeof(expanded));
    memcpy(d, expanded, 32);
    memcpy(z, expanded + 32, 32);
    memcpy(m, expanded + 64, 32);

    uint8_t ek[MLKEM768_PUBLICKEYBYTES];
    uint8_t dk[MLKEM768_SECRETKEYBYTES];
    uint8_t ct[MLKEM768_CIPHERTEXTBYTES];
    uint8_t k_alice[32], k_bob[32], k_tamper[32];

    moonlab_mlkem768_keygen(ek, dk, d, z);
    if (moonlab_mlkem768_check_ek(ek) != 1) return -1;
    if (moonlab_mlkem768_check_dk(dk) != 1) return -2;

    moonlab_mlkem768_encaps(ct, k_alice, ek, m);
    moonlab_mlkem768_decaps(k_bob, ct, dk);
    if (memcmp(k_alice, k_bob, 32) != 0) return -3;

    /* Planted IND-CCA2 Tamper Mutant: flip 1 bit in ciphertext -> FO transform MUST produce K_tamper != K_alice */
    ct[0] ^= 0x01;
    moonlab_mlkem768_decaps(k_tamper, ct, dk);
    int rejected = (memcmp(k_alice, k_tamper, 32) != 0) ? 1 : 0;
    if (out_fo_tamper_rejected) *out_fo_tamper_rejected = rejected;
    if (!rejected) return -4;

    if (out_shared_secret) memcpy(out_shared_secret, k_alice, 32);
    return 0;
}
