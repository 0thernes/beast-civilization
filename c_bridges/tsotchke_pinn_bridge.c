/*
 * Live C-ABI Bridge to upstream `tsotchke/PINN` (Physics-Informed Neural Network in C).
 * Compiled directly via `zig cc` into `bin/tsotchke_pinn_bridge.dll`.
 */

#include <stdint.h>
#include <math.h>
#include "neural_network.h"

#if defined(_WIN32)
  #define BEAST_PINN_EXPORT __declspec(dllexport)
#else
  #define BEAST_PINN_EXPORT __attribute__((visibility("default")))
#endif

BEAST_PINN_EXPORT int beast_tsotchke_pinn_verify(double x0, double x1, double out3[3]) {
    NeuralNetwork nn;
    initialize_neural_network(&nn);
    if (!validate_neural_network_initialization(&nn)) {
        return -1;
    }
    double inp[INPUT_SIZE] = { x0, x1 };
    forward_pass(&nn, inp, out3, TANH);
    for (int i = 0; i < OUTPUT_SIZE; i++) {
        if (!isfinite(out3[i])) return -2;
    }
    return 0;
}
