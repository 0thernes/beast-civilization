/*
 * Live C-ABI Bridge to upstream `tsotchke/quantum_geometric_tensor` (`complex_arithmetic.c`).
 * Compiled directly via `zig cc` into `bin/tsotchke_qgtl_bridge.dll` (12th live native DLL!).
 */

#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include "quantum_geometric/core/complex_arithmetic.h"

#if defined(_WIN32)
  #define BEAST_QGTL_EXPORT __declspec(dllexport)
#else
  #define BEAST_QGTL_EXPORT __attribute__((visibility("default")))
#endif

/* Verifies Bloch-sphere quantum state inner products & Fubini-Study fidelity using upstream QGTL complex arithmetic:
 *   |psi(theta, phi)> = [ cos(theta/2), e^{i*phi} sin(theta/2) ]
 *   Checks:
 *     1. Normalization <psi|psi> == 1.0 + 0i
 *     2. Orthogonal antipodal state |psi_perp> = [ -sin(theta/2), e^{i*phi} cos(theta/2) ] has <psi|psi_perp> == 0
 *     3. Fubini-Study overlap |<psi(t1)|psi(t2)>|^2 == cos^2((t2 - t1)/2) for identical phi!
 */
BEAST_QGTL_EXPORT int beast_tsotchke_qgtl_verify(
    float theta1, float theta2, float phi,
    float *out_fidelity,
    float *out_ortho_err
) {
    ComplexFloat psi1[2] = {
        { cosf(theta1 * 0.5f), 0.0f },
        { cosf(phi) * sinf(theta1 * 0.5f), sinf(phi) * sinf(theta1 * 0.5f) }
    };
    ComplexFloat psi1_perp[2] = {
        { -sinf(theta1 * 0.5f), 0.0f },
        { cosf(phi) * cosf(theta1 * 0.5f), sinf(phi) * cosf(theta1 * 0.5f) }
    };
    ComplexFloat psi2[2] = {
        { cosf(theta2 * 0.5f), 0.0f },
        { cosf(phi) * sinf(theta2 * 0.5f), sinf(phi) * sinf(theta2 * 0.5f) }
    };

    ComplexFloat norm1 = complex_add(
        complex_multiply(complex_conjugate(psi1[0]), psi1[0]),
        complex_multiply(complex_conjugate(psi1[1]), psi1[1])
    );
    if (fabsf(norm1.real - 1.0f) > 1e-5f || fabsf(norm1.imag) > 1e-5f) return -1;

    ComplexFloat ortho = complex_add(
        complex_multiply(complex_conjugate(psi1[0]), psi1_perp[0]),
        complex_multiply(complex_conjugate(psi1[1]), psi1_perp[1])
    );
    float oerr = complex_abs(ortho);
    if (oerr > 1e-5f) return -2;

    ComplexFloat ov = complex_add(
        complex_multiply(complex_conjugate(psi1[0]), psi2[0]),
        complex_multiply(complex_conjugate(psi1[1]), psi2[1])
    );
    float fid = complex_abs_squared(ov);
    float expected_fid = cosf((theta2 - theta1) * 0.5f);
    expected_fid *= expected_fid;
    if (fabsf(fid - expected_fid) > 1e-5f) return -3;

    if (out_fidelity) *out_fidelity = fid;
    if (out_ortho_err) *out_ortho_err = oerr;
    return 0;
}
