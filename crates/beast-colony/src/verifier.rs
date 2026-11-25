//! Engram FSRS-4.5 mastery scheduler + blind examiner + Crucible pre-registration
//! + StrictlyBetter noise-aware monotonic merge gate.

/// FSRS-4.5 power-law forgetting curve retrievability: `R(t, S) = (1 + t / (9 * S))^-1`.
pub fn fsrs45_retrievability(elapsed_days: f64, stability_days: f64) -> f64 {
    let s = stability_days.max(0.01);
    let t = elapsed_days.max(0.0);
    (1.0 + t / (9.0 * s)).powi(-1)
}

/// FSRS-4.5 next optimal interval in days to maintain target retention `r_target` (e.g. 0.90).
pub fn fsrs45_next_interval(stability_days: f64, r_target: f64) -> f64 {
    let r = r_target.clamp(0.5, 0.99);
    9.0 * stability_days.max(0.01) * (1.0 / r - 1.0)
}

/// StrictlyBetter noise-aware paired research loop gate (`nagisanzenin/strictlybetter`).
/// Rejects candidate unless mean delta > min_effect AND win fraction >= 0.70 across locked seeds.
#[derive(Debug, Clone)]
pub struct StrictlyBetterVerdict {
    pub accepted: bool,
    pub pairs: usize,
    pub wins: usize,
    pub losses: usize,
    pub mean_delta: f64,
    pub reason: &'static str,
}

pub fn strictly_better_gate(baseline: &[f64], candidate: &[f64], min_effect: f64) -> StrictlyBetterVerdict {
    if baseline.len() != candidate.len() || baseline.len() < 8 {
        return StrictlyBetterVerdict {
            accepted: false,
            pairs: baseline.len().min(candidate.len()),
            wins: 0,
            losses: 0,
            mean_delta: 0.0,
            reason: "insufficient paired seeds (need N >= 8)",
        };
    }
    let n = baseline.len();
    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut sum_d = 0.0f64;
    for i in 0..n {
        let d = candidate[i] - baseline[i];
        sum_d += d;
        if d > 1e-12 {
            wins += 1;
        } else if d < -1e-12 {
            losses += 1;
        }
    }
    let mean_delta = sum_d / (n as f64);
    let win_rate = (wins as f64) / (n as f64);
    if mean_delta >= min_effect && win_rate >= 0.70 && wins > losses {
        StrictlyBetterVerdict {
            accepted: true,
            pairs: n,
            wins,
            losses,
            mean_delta,
            reason: "strictly better across locked seed band",
        }
    } else {
        StrictlyBetterVerdict {
            accepted: false,
            pairs: n,
            wins,
            losses,
            mean_delta,
            reason: "failed noise-aware monotonic threshold",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fsrs_and_strictly_better_reject_noise_mutant() {
        let r9 = fsrs45_retrievability(10.0, 10.0);
        assert!((r9 - 0.9).abs() < 1e-9);
        assert!((fsrs45_next_interval(10.0, 0.9) - 10.0).abs() < 1e-9);

        let base = vec![1.0; 24];
        let mut cand_good = vec![1.2; 24];
        assert!(strictly_better_gate(&base, &cand_good, 0.1).accepted);

        // Planted Mutant: noisy candidate that regresses on half the seeds -> MUST REJECT
        for i in (0..24).step_by(2) {
            cand_good[i] = 0.85;
        }
        assert!(!strictly_better_gate(&base, &cand_good, 0.05).accepted);
    }
}
