/*
 * Live C-ABI Bridge to upstream `tsotchke/libirrep` (SO(3)/SE(3) Spherical Harmonics + AVX2 Dispatch).
 * Compiled directly via `zig cc` into `bin/tsotchke_libirrep_bridge.dll`.
 */

#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <irrep/spherical_harmonics.h>

#if defined(_WIN32)
  #define BEAST_IRREP_EXPORT __declspec(dllexport)
#else
  #define BEAST_IRREP_EXPORT __attribute__((visibility("default")))
#endif

#ifndef M_PI
  #define M_PI 3.14159265358979323846
#endif

BEAST_IRREP_EXPORT int beast_tsotchke_libirrep_sph_verify(
    double rx, double ry, double rz,
    double out_sh[9],
    double *out_total_norm_sq
) {
    double norm = sqrt(rx * rx + ry * ry + rz * rz);
    if (!(norm > 1e-12)) return -1;
    double r_hat[3] = { rx / norm, ry / norm, rz / norm };

    /* Compute all l = 0, 1, 2 real orthonormal spherical harmonics (9 components). */
    irrep_sph_harm_cart_all(2, out_sh, r_hat);

    /* Unsold Addition Theorem on S^2: sum_{m=-l}^l |Y_{lm}(r_hat)|^2 == (2l+1)/(4*pi). */
    double s0 = out_sh[0] * out_sh[0];
    double s1 = out_sh[1] * out_sh[1] + out_sh[2] * out_sh[2] + out_sh[3] * out_sh[3];
    double s2 = 0.0;
    for (int i = 4; i < 9; i++) s2 += out_sh[i] * out_sh[i];

    double inv4pi = 1.0 / (4.0 * M_PI);
    if (fabs(s0 - 1.0 * inv4pi) > 1e-11) return -2;
    if (fabs(s1 - 3.0 * inv4pi) > 1e-11) return -3;
    if (fabs(s2 - 5.0 * inv4pi) > 1e-11) return -4;

    /* Verify runtime-dispatched batched kernel (AVX2+FMA on x86-64) matches scalar exactly. */
    double batch_in[4 * 3];
    double batch_out[4 * 9];
    for (int k = 0; k < 4; k++) {
        batch_in[k * 3 + 0] = r_hat[0];
        batch_in[k * 3 + 1] = r_hat[1];
        batch_in[k * 3 + 2] = r_hat[2];
    }
    irrep_sph_harm_cart_all_batch(2, 4, batch_in, batch_out);
    for (int k = 0; k < 4; k++) {
        for (int j = 0; j < 9; j++) {
            if (fabs(batch_out[k * 9 + j] - out_sh[j]) > 1e-11) return -5;
        }
    }

    if (out_total_norm_sq) {
        *out_total_norm_sq = s0 + s1 + s2;
    }
    return 0;
}
