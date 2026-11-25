//! TEMM1E mathematical primitives (`temm1e-labs/temm1e`), implemented verbatim and tested.

/// λ-memory continuous attention curve: `s(t) = clamp(I + b, 0.1, 5.0) * exp(-lambda * dt_hours)`.
pub fn lambda_memory_score(importance: f64, boost: f64, lambda_per_hour: f64, dt_hours: f64) -> f64 {
    let i_eff = (importance + boost).clamp(0.1, 5.0);
    i_eff * (-lambda_per_hour * dt_hours.max(0.0)).exp()
}

/// Learning artifact score: `V = (alpha / (alpha + beta)) * exp(-0.015 * d) * (1 + 0.3 * ln(1 + u))`.
pub fn bayesian_artifact_value(alpha: f64, beta: f64, age_days: f64, uses: u64) -> f64 {
    let mean = alpha / (alpha + beta).max(1e-12);
    mean * (-0.015 * age_days.max(0.0)).exp() * (1.0 + 0.3 * ((1 + uses) as f64).ln())
}

/// Perpetuum Beta(1,1) smoothed slot activity estimate: `p_hat = (a + 1) / (n + 2)`.
pub fn perpetuum_beta_prob(active_slots: u64, eligible_slots: u64) -> f64 {
    ((active_slots + 1) as f64) / ((eligible_slots + 2) as f64)
}

/// Many-Tems swarm selection policy: `S = A^a * U^b * (1 - D)^g * (1 - F)^d * R^z`.
#[derive(Debug, Clone, Copy)]
pub struct ManyTemsExponents {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub delta: f64,
    pub zeta: f64,
}

impl Default for ManyTemsExponents {
    fn default() -> Self {
        Self { alpha: 1.0, beta: 1.2, gamma: 0.8, delta: 1.5, zeta: 1.0 }
    }
}

pub fn many_tems_priority(
    affinity: f64,
    urgency: f64,
    difficulty: f64,
    failure_rate: f64,
    reward: f64,
    exp: ManyTemsExponents,
) -> f64 {
    let a = affinity.clamp(0.01, 1.0);
    let u = urgency.clamp(0.01, 1.0);
    let one_m_d = (1.0 - difficulty.clamp(0.0, 0.99)).max(0.01);
    let one_m_f = (1.0 - failure_rate.clamp(0.0, 0.99)).max(0.01);
    let r = reward.clamp(0.01, 10.0);
    a.powf(exp.alpha)
        * u.powf(exp.beta)
        * one_m_d.powf(exp.gamma)
        * one_m_f.powf(exp.delta)
        * r.powf(exp.zeta)
}

/// Eigen-Tune Wilson score lower bound (rejects n == 0 or k > n).
/// At 99% two-sided confidence (`z = 2.5758293035489004`), 30/30 yields `0.818891`.
pub fn eigen_tune_wilson_lower(successes: u64, trials: u64, z: f64) -> Result<f64, &'static str> {
    if trials == 0 || successes > trials || !z.is_finite() || z <= 0.0 {
        return Err("invalid Wilson parameters");
    }
    let n = trials as f64;
    let p = (successes as f64) / n;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = p + z2 / (2.0 * n);
    let rad = z * ((p * (1.0 - p)) / n + z2 / (4.0 * n * n)).sqrt();
    Ok(((center - rad) / denom).clamp(0.0, 1.0))
}

/// Sequential Probability Ratio Test (SPRT) log-likelihood ratio for local distillation promotion.
pub fn sprt_log_likelihood_ratio(successes: u64, trials: u64, p0: f64, p1: f64) -> f64 {
    let k = successes as f64;
    let f = (trials.saturating_sub(successes)) as f64;
    k * (p1 / p0).ln() + f * ((1.0 - p1) / (1.0 - p0)).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_matches_temm1e_exact_30_of_30_reference() {
        // TEMM1E README exact reference: at 99% two-sided confidence, 30/30 -> 0.818891
        let l = eigen_tune_wilson_lower(30, 30, 2.5758293035489004).unwrap();
        assert!((l - 0.818891).abs() < 1e-5, "got {l}");
        assert!(eigen_tune_wilson_lower(0, 0, 2.5758).is_err());
        assert!(eigen_tune_wilson_lower(31, 30, 2.5758).is_err());
    }

    #[test]
    fn lambda_half_life_matches_69_3h() {
        let s0 = lambda_memory_score(2.0, 0.0, 0.01, 0.0);
        let s_half = lambda_memory_score(2.0, 0.0, 0.01, 2.0f64.ln() / 0.01);
        assert!((s_half - s0 * 0.5).abs() < 1e-12);
    }
}
