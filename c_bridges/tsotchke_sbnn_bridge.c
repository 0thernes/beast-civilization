/*
 * Live C-ABI Bridge to upstream `tsotchke/spin_based_neural_network`:
 *   1. Universal Fibonacci Non-Abelian Anyons (`fibonacci_anyons.c`): Yang-Baxter B1*B2*B1 == B2*B1*B2 & F-matrix unitarity
 *   2. Spintronic Probabilistic-Bit (`p-bit`) Ising Machine (`pbit.c`): MaxCut simulated annealing
 * Compiled directly via `zig cc` into `bin/tsotchke_sbnn_bridge.dll`.
 */

#include <complex.h>
#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include "fibonacci_anyons/fibonacci_anyons.h"
#include "neuromorphic/pbit.h"

#if defined(_WIN32)
  #define BEAST_SBNN_EXPORT __declspec(dllexport)
#else
  #define BEAST_SBNN_EXPORT __attribute__((visibility("default")))
#endif

BEAST_SBNN_EXPORT int beast_tsotchke_sbnn_verify(
    uint64_t seed,
    double *out_yang_baxter_err,
    double *out_fibo_phi,
    double *out_pbit_final_energy,
    int *out_fibo_dim_8
) {
    /* 1. Fibonacci Non-Abelian Anyon Braiding: verify Yang-Baxter relation B1*B2*B1 == B2*B1*B2 */
    double _Complex b1[2][2], b2[2][2], tmp1[2][2], lhs[2][2], tmp2[2][2], rhs[2][2];
    fibo_braid_b1(b1);
    fibo_braid_b2(b2);
    fibo_mat2_mul(b1, b2, tmp1);
    fibo_mat2_mul(tmp1, b1, lhs);
    fibo_mat2_mul(b2, b1, tmp2);
    fibo_mat2_mul(tmp2, b2, rhs);

    double yb_err = fibo_operator_norm_distance(lhs, rhs);
    if (!isfinite(yb_err) || yb_err > 1e-11) return -1;

    /* Verify F-matrix unitarity: F^dagger * F == I */
    fibo_fmatrix_t fmat = fibo_f_matrix();
    double _Complex fdag[2][2], prod[2][2], eye[2][2];
    fibo_mat2_dagger(fmat.m, fdag);
    fibo_mat2_mul(fdag, fmat.m, prod);
    fibo_mat2_identity(eye);
    double f_err = fibo_operator_norm_distance(prod, eye);
    if (!isfinite(f_err) || f_err > 1e-11) return -2;

    long dim8 = fibo_hilbert_dim(8); /* Fibonacci(7) = 13 or Fibonacci(8) = 21 */
    if (dim8 <= 0) return -3;

    /* 2. Spintronic Probabilistic-Bit (p-bit) Ising Machine: 6-node MaxCut annealing */
    const int n = 6;
    const int edges_u[9] = { 0, 1, 2, 3, 4, 5, 0, 1, 2 };
    const int edges_v[9] = { 1, 2, 3, 4, 5, 0, 3, 4, 5 };
    const double weights[9] = { 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.5, 1.5, 1.5 };
    double J[36];
    if (pbit_maxcut_couplings(n, 9, edges_u, edges_v, weights, J) != 0) return -4;

    int spins[6] = { 1, 1, 1, 1, 1, 1 };
    pbit_net_t net;
    pbit_net_init(&net, n, J, NULL, 0.2, (unsigned long long)(seed ? seed : 5000ULL));
    double e_init = pbit_ising_energy(&net, spins);
    double e_final = 0.0;
    if (pbit_anneal(&net, spins, 0.2, 6.0, 200, &e_final) != 0) return -5;
    if (!isfinite(e_final) || !(e_final < e_init)) return -6;

    if (out_yang_baxter_err) *out_yang_baxter_err = yb_err;
    if (out_fibo_phi) *out_fibo_phi = FIBO_PHI;
    if (out_pbit_final_energy) *out_pbit_final_energy = e_final;
    if (out_fibo_dim_8) *out_fibo_dim_8 = (int)dim8;
    return 0;
}
