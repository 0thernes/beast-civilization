//! Tsotchke Quantum/Geometric Primitives + 0thernes 24 Operating Margins:
//! - Moonlab-compatible Ising/QUBO Simulated Quantum Annealer for colony scheduling
//! - QGTL Quantum Geometric Tensor: Fubini-Study metric (Re Q) & Berry curvature (Im Q)
//! - QRNG CHSH Bell-Inequality verifier (quantum limit 2*sqrt(2) ~= 2.8284 > 2.0)
//! - 24 Operating Margins (`GM-01..24`) & 13-Vector Restraint Iron-Law telemetry

use beast_wire::philox::philox_f64_unit;

/// Solve a QUBO objective `E(x) = sum_i h_i x_i + sum_{i<j} J_ij x_i x_j` for `x_i in {0, 1}`
/// via deterministic cosine-schedule transverse-field annealing (Moonlab-compatible).
#[derive(Debug, Clone)]
pub struct QuboAnnealResult {
    pub best_bits: Vec<u8>,
    pub best_energy: f64,
    pub initial_energy: f64,
}

pub fn qubo_energy(bits: &[u8], linear: &[f64], quad: &[f64]) -> f64 {
    let n = bits.len();
    let mut e = 0.0;
    for i in 0..n {
        if bits[i] != 0 {
            e += linear[i];
            for j in (i + 1)..n {
                if bits[j] != 0 {
                    e += quad[i * n + j];
                }
            }
        }
    }
    e
}

pub fn moonlab_style_qubo_anneal(
    linear: &[f64],
    quad: &[f64],
    sweeps: usize,
    seed: u64,
) -> QuboAnnealResult {
    let n = linear.len();
    assert_eq!(quad.len(), n * n);
    let mut state: Vec<u8> = (0..n)
        .map(|i| u8::from(philox_f64_unit(seed, i as u64, 0xA001) >= 0.5))
        .collect();
    let initial_energy = qubo_energy(&state, linear, quad);
    let mut cur_e = initial_energy;
    let mut best_bits = state.clone();
    let mut best_energy = cur_e;

    let mut ctr = 0u64;
    for step in 0..sweeps {
        let progress = (step as f64) / (sweeps.max(1) as f64);
        // Cosine transverse-field / temperature schedule
        let temp = 0.01 + 2.5 * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos());
        for i in 0..n {
            state[i] ^= 1;
            let cand_e = qubo_energy(&state, linear, quad);
            let de = cand_e - cur_e;
            let accept = if de <= 0.0 {
                true
            } else {
                ctr += 1;
                philox_f64_unit(seed, ctr, 0xA002) < (-de / temp).exp()
            };
            if accept {
                cur_e = cand_e;
                if cur_e < best_energy {
                    best_energy = cur_e;
                    best_bits.copy_from_slice(&state);
                }
            } else {
                state[i] ^= 1;
            }
        }
    }

    QuboAnnealResult {
        best_bits,
        best_energy,
        initial_energy,
    }
}

/// Complex number for exact QGTL state-vector geometry.
#[derive(Debug, Clone, Copy)]
pub struct C64 {
    pub re: f64,
    pub im: f64,
}

impl C64 {
    pub fn conj(self) -> Self { Self { re: self.re, im: -self.im } }
    pub fn cmul(self, o: Self) -> Self {
        Self { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
    pub fn cadd(self, o: Self) -> Self { Self { re: self.re + o.re, im: self.im + o.im } }
    pub fn csub(self, o: Self) -> Self { Self { re: self.re - o.re, im: self.im - o.im } }
}

fn inner(a: &[C64], b: &[C64]) -> C64 {
    let mut acc = C64 { re: 0.0, im: 0.0 };
    for (&ai, &bi) in a.iter().zip(b.iter()) {
        acc = acc.cadd(ai.conj().cmul(bi));
    }
    acc
}

/// Exact Quantum Geometric Tensor on a Bloch-sphere qubit state `|psi(theta, phi)> = [cos(t/2), e^{i*phi} sin(t/2)]`.
/// Exact analytical result:
///   Fubini-Study metric `g_tt = 1/4`, `g_pp = (1/4) sin^2(theta)`, `g_tp = 0`
///   Berry curvature `Omega_tp = -2 * Im(Q_tp) = -(1/2) sin(theta)` (integrates to Chern-like 2*pi solid angle).
#[derive(Debug, Clone, Copy)]
pub struct QgtPoint {
    pub g_theta_theta: f64,
    pub g_phi_phi: f64,
    pub berry_curvature: f64,
}

pub fn qgtl_bloch_qgt(theta: f64, phi: f64) -> QgtPoint {
    let ht = 0.5 * theta;
    let psi = [
        C64 { re: ht.cos(), im: 0.0 },
        C64 { re: phi.cos() * ht.sin(), im: phi.sin() * ht.sin() },
    ];
    let d_theta = [
        C64 { re: -0.5 * ht.sin(), im: 0.0 },
        C64 { re: 0.5 * phi.cos() * ht.cos(), im: 0.5 * phi.sin() * ht.cos() },
    ];
    let d_phi = [
        C64 { re: 0.0, im: 0.0 },
        C64 { re: -phi.sin() * ht.sin(), im: phi.cos() * ht.sin() },
    ];

    let dt_psi = inner(&d_theta, &psi);
    let dp_psi = inner(&d_phi, &psi);
    let psi_dt = inner(&psi, &d_theta);
    let psi_dp = inner(&psi, &d_phi);

    let q_tt = inner(&d_theta, &d_theta).csub(dt_psi.cmul(psi_dt));
    let q_pp = inner(&d_phi, &d_phi).csub(dp_psi.cmul(psi_dp));
    let q_tp = inner(&d_theta, &d_phi).csub(dt_psi.cmul(psi_dp));

    QgtPoint {
        g_theta_theta: q_tt.re,
        g_phi_phi: q_pp.re,
        berry_curvature: -2.0 * q_tp.im,
    }
}

/// QRNG CHSH Bell-inequality Born-rule simulation on Bell state `|Phi+> = (|00> + |11>) / sqrt(2)`.
/// Measurement correlation `E(a, b) = cos(2*(a - b))` sampled from `shots` Born-rule collapses.
/// Optimal Tsirelson angles: `a=0, a'=pi/4, b=pi/8, b'=3*pi/8` -> `S -> 2*sqrt(2) ~= 2.8284 > 2.0`.
pub fn qrng_chsh_bell_test(shots_per_setting: usize, seed: u64) -> f64 {
    let pi = std::f64::consts::PI;
    let settings = [
        (0.0, pi / 8.0, 1.0f64),
        (0.0, 3.0 * pi / 8.0, -1.0f64),
        (pi / 4.0, pi / 8.0, 1.0f64),
        (pi / 4.0, 3.0 * pi / 8.0, 1.0f64),
    ];
    let mut s_val = 0.0;
    for (idx, &(a, b, sign)) in settings.iter().enumerate() {
        let p_same = (a - b).cos().powi(2); // Born rule P(same) = cos^2(a-b) -> E = 2*P_same - 1 = cos(2(a-b))
        let mut corr = 0i64;
        for shot in 0..shots_per_setting {
            let u = philox_f64_unit(seed, shot as u64, 0xB311 + idx as u32);
            if u < p_same { corr += 1; } else { corr -= 1; }
        }
        let e_ab = (corr as f64) / (shots_per_setting as f64);
        s_val += sign * e_ab;
    }
    s_val.abs()
}

/// 24 Operating Margins (`GM-01..24`) snapshot (alife-singularity-omega v3.3).
#[derive(Debug, Clone)]
pub struct MarginSnapshot {
    pub spectral_margin: f64,     // 0.99 - ||W||_2 (must be >= 0)
    pub vram_headroom_frac: f64,  // free VRAM / total VRAM
    pub budget_headroom_frac: f64,
    pub c2c_speedup_margin: f64,  // speedup_x - 1.0
    pub chsh_quantum_margin: f64, // S - 2.0 (must exceed 0 for quantum violation)
    pub restraint_mean_pct: f64,  // mean across 13 castration vectors (0 = feral)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qgtl_exact_bloch_and_chsh_violates_classical_bound() {
        let theta = std::f64::consts::FRAC_PI_3;
        let q = qgtl_bloch_qgt(theta, 0.7);
        assert!((q.g_theta_theta - 0.25).abs() < 1e-12);
        assert!((q.g_phi_phi - 0.25 * theta.sin().powi(2)).abs() < 1e-12);
        assert!((q.berry_curvature.abs() - 0.5 * theta.sin()).abs() < 1e-12);

        let s = qrng_chsh_bell_test(8192, 5000);
        assert!(s > 2.65 && s < 2.95, "CHSH S={s} failed Bell violation");
    }

    #[test]
    fn qubo_annealer_reduces_energy() {
        let linear = [-2.0, -1.5, -2.5, -1.0, -3.0, -1.2];
        let mut quad = vec![0.0; 36];
        // Penalize putting conflicting tasks 0&1 or 2&4 on same caste
        quad[1] = 4.0;
        quad[2 * 6 + 4] = 5.0;
        let res = moonlab_style_qubo_anneal(&linear, &quad, 200, 5000);
        assert!(res.best_energy <= res.initial_energy);
        assert!(res.best_energy <= -5.0, "expected low QUBO energy, got {}", res.best_energy);
    }
}
