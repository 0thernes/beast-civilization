/*
 * Live C-ABI Bridge to upstream `tsotchke/moonlab` Quantum Geometric Tensor & Topology engine (`qgt.c`).
 * Compiled directly via `zig cc` into `bin/tsotchke_moonlab_qgt_bridge.dll`.
 */

#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include "algorithms/quantum_geometry/qgt.h"

#if defined(_WIN32)
  #define BEAST_MOONLAB_EXPORT __declspec(dllexport)
#else
  #define BEAST_MOONLAB_EXPORT __attribute__((visibility("default")))
#endif

BEAST_MOONLAB_EXPORT int beast_tsotchke_moonlab_qgt_verify(
    int *out_qwz_chern_topo,
    int *out_qwz_chern_triv,
    int *out_ssh_winding,
    int *out_kitaev_z2,
    double *out_fs_trace,
    double *out_berry_omega
) {
    /* 1. Qi-Wu-Zhang (QWZ) Chern Insulator in topological phase m = -1.0 -> C = +1 */
    qgt_system_t *qwz_topo = qgt_model_qwz(-1.0);
    if (!qwz_topo) return -1;
    qgt_berry_grid_t grid_topo = {0};
    if (qgt_berry_grid_proj(qwz_topo, 24, &grid_topo) != 0) {
        qgt_free(qwz_topo);
        return -2;
    }
    int c_topo = (int)llround(grid_topo.chern);

    /* Exact closed-form Provost-Vallee Fubini-Study metric g_{mu,nu} & Berry curvature Omega_{xy} */
    double k_pt[2] = { 0.7, 0.5 };
    double g[4] = {0};
    double omega = 0.0;
    if (qgt_exact_curvature_at(qwz_topo, k_pt, g, &omega) != 0) {
        qgt_berry_grid_free(&grid_topo);
        qgt_free(qwz_topo);
        return -3;
    }
    /* Quantum Geometric Bound (positive semi-definiteness of QGT): det(g) >= (Omega/2)^2 */
    double det_g = g[0] * g[3] - g[1] * g[2];
    double trace_g = g[0] + g[3];
    if (!(trace_g > 0.0) || det_g + 1e-12 < 0.25 * omega * omega) {
        qgt_berry_grid_free(&grid_topo);
        qgt_free(qwz_topo);
        return -4;
    }
    qgt_berry_grid_free(&grid_topo);
    qgt_free(qwz_topo);

    /* 2. QWZ in trivial insulator phase m = +3.0 -> C = 0 */
    qgt_system_t *qwz_triv = qgt_model_qwz(3.0);
    if (!qwz_triv) return -5;
    qgt_berry_grid_t grid_triv = {0};
    if (qgt_berry_grid_proj(qwz_triv, 16, &grid_triv) != 0) {
        qgt_free(qwz_triv);
        return -6;
    }
    int c_triv = (int)llround(grid_triv.chern);
    qgt_berry_grid_free(&grid_triv);
    qgt_free(qwz_triv);

    /* 3. Su-Schrieffer-Heeger (SSH) 1D chiral chain (|t2| > |t1|) -> winding W = +1 */
    qgt_system_1d_t *ssh = qgt_model_ssh(0.5, 1.5);
    if (!ssh) return -7;
    int w_ssh = qgt_winding_1d(ssh, 64, NULL);
    qgt_free_1d(ssh);

    /* 4. Kitaev 1D p-wave Majorana chain (|mu| < 2|t|) -> Z_2 = 1 */
    qgt_system_1d_t *kitaev = qgt_model_kitaev_chain(1.0, 0.5, 0.8);
    if (!kitaev) return -8;
    int z2 = -1;
    if (qgt_z2_invariant_1d_bdg(kitaev, &z2) != 0) {
        qgt_free_1d(kitaev);
        return -9;
    }
    qgt_free_1d(kitaev);

    if (out_qwz_chern_topo) *out_qwz_chern_topo = c_topo;
    if (out_qwz_chern_triv) *out_qwz_chern_triv = c_triv;
    if (out_ssh_winding)    *out_ssh_winding    = w_ssh;
    if (out_kitaev_z2)      *out_kitaev_z2      = z2;
    if (out_fs_trace)       *out_fs_trace       = trace_g;
    if (out_berry_omega)    *out_berry_omega    = omega;

    if (c_topo != 1 || c_triv != 0 || w_ssh != 1 || z2 != 1) return -10;
    return 0;
}
