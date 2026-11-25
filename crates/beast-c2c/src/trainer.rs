//! Real analytical-gradient trainer for cross-family MIT `Swarm Telepathy` (SwaT) mappers
//! + empirical reproduction of the MIT 2x2 finding ("Augment the Cache, Replace the Activation"):
//!   - Trains `(W_K, W_V)` projecting sharer family (`dim_src`) -> receiver family (`dim_dst`)
//!   - Evaluates strictly on HELD-OUT test vectors (`B33`, zero train/test leakage!)
//!   - Empirically compares `Augment-Cache` (`[K_self; K_peer]`) vs destructive `Overwrite-Cache`
//!     on simultaneous self-context + peer-context recall probes!

use crate::{augment_cache_attend, KvCache, ModelFamily, SwatCrossFamilyMapper};
use beast_wire::philox::philox_f64_unit;

#[derive(Debug, Clone)]
pub struct SwatTrainingReport {
    pub steps: usize,
    pub init_heldout_mse: f32,
    pub final_heldout_mse: f32,
    pub mse_reduction_pct: f64,
    pub heldout_cosine_sim: f64,
    pub augment_self_recall_cos: f64,
    pub overwrite_self_recall_cos: f64,
    pub augment_beats_overwrite: bool,
}

fn matvec(w: &[f32], x: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; rows];
    for r in 0..rows {
        let base = r * cols;
        let mut acc = 0.0f32;
        for c in 0..cols {
            acc += w[base + c] * x[c];
        }
        out[r] = acc;
    }
    out
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f64 {
    let mut dot = 0.0f64;
    let mut na = 0.0f64;
    let mut nb = 0.0f64;
    for i in 0..a.len() {
        dot += (a[i] as f64) * (b[i] as f64);
        na += (a[i] as f64) * (a[i] as f64);
        nb += (b[i] as f64) * (b[i] as f64);
    }
    dot / (na.sqrt() * nb.sqrt()).max(1e-12)
}

/// Train a cross-family SwaT KV mapper on an orthogonal latent basis bridge (`src_dim -> dst_dim`)
/// using analytical SGD + momentum, evaluate strictly on held-out pairs, and verify MIT Law #1
/// (`Augment-Cache` preserves receiver self-recall where destructive `Overwrite` collapses it).
pub fn train_and_verify_swat_mapper(
    src_dim: usize,
    dst_dim: usize,
    steps: usize,
    seed: u64,
) -> (SwatCrossFamilyMapper, SwatTrainingReport) {
    // True unknown cross-family semantic projection ground truth `T_true` [dst_dim x src_dim]
    let scale = 1.0 / (src_dim as f32).sqrt();
    let t_true_k: Vec<f32> = (0..(dst_dim * src_dim))
        .map(|i| ((philox_f64_unit(seed, i as u64, 0x7001) * 2.0 - 1.0) as f32) * scale)
        .collect();
    let t_true_v: Vec<f32> = (0..(dst_dim * src_dim))
        .map(|i| ((philox_f64_unit(seed, i as u64, 0x7002) * 2.0 - 1.0) as f32) * scale)
        .collect();

    // Disjoint train (64 samples) and held-out test (32 samples) sets (`B33`)
    let n_train = 64usize;
    let n_test = 32usize;
    let gen_samples = |count: usize, domain_base: u32| -> Vec<(Vec<f32>, Vec<f32>)> {
        (0..count)
            .map(|s| {
                let xk: Vec<f32> = (0..src_dim)
                    .map(|d| (philox_f64_unit(seed, (s * src_dim + d) as u64, domain_base) * 2.0 - 1.0) as f32)
                    .collect();
                let xv: Vec<f32> = (0..src_dim)
                    .map(|d| (philox_f64_unit(seed, (s * src_dim + d) as u64, domain_base + 1) * 2.0 - 1.0) as f32)
                    .collect();
                (xk, xv)
            })
            .collect()
    };
    let train_x = gen_samples(n_train, 0x7100);
    let test_x = gen_samples(n_test, 0x7200);

    let mut mapper = SwatCrossFamilyMapper::new(ModelFamily::Qwen3, ModelFamily::TemRust, src_dim, dst_dim, seed + 99);
    mapper.layer_gate = 1.0;

    let eval_test = |m: &SwatCrossFamilyMapper| -> (f32, f64) {
        let mut mse = 0.0f64;
        let mut cos_sum = 0.0f64;
        for (xk, xv) in &test_x {
            let yk_true = matvec(&t_true_k, xk, dst_dim, src_dim);
            let yv_true = matvec(&t_true_v, xv, dst_dim, src_dim);
            let yk_pred = matvec(&m.w_k, xk, dst_dim, src_dim);
            let yv_pred = matvec(&m.w_v, xv, dst_dim, src_dim);
            for d in 0..dst_dim {
                let ek = (yk_pred[d] - yk_true[d]) as f64;
                let ev = (yv_pred[d] - yv_true[d]) as f64;
                mse += ek * ek + ev * ev;
            }
            cos_sum += 0.5 * (cosine_sim(&yk_pred, &yk_true) + cosine_sim(&yv_pred, &yv_true));
        }
        let mean_mse = (mse / ((2 * n_test * dst_dim) as f64)) as f32;
        (mean_mse, cos_sum / (n_test as f64))
    };

    let (init_heldout_mse, _) = eval_test(&mapper);

    // Analytical full-batch gradient descent with Nesterov/heavy-ball momentum
    let mut vel_k = vec![0.0f32; dst_dim * src_dim];
    let mut vel_v = vec![0.0f32; dst_dim * src_dim];
    let lr = 0.12f32;
    let mom = 0.85f32;
    let inv_n = 1.0 / (n_train as f32);

    for _step in 0..steps {
        let mut gk = vec![0.0f32; dst_dim * src_dim];
        let mut gv = vec![0.0f32; dst_dim * src_dim];
        for (xk, xv) in &train_x {
            let yk_true = matvec(&t_true_k, xk, dst_dim, src_dim);
            let yv_true = matvec(&t_true_v, xv, dst_dim, src_dim);
            let yk_pred = matvec(&mapper.w_k, xk, dst_dim, src_dim);
            let yv_pred = matvec(&mapper.w_v, xv, dst_dim, src_dim);
            for r in 0..dst_dim {
                let ek = (yk_pred[r] - yk_true[r]) * inv_n;
                let ev = (yv_pred[r] - yv_true[r]) * inv_n;
                let row = r * src_dim;
                for c in 0..src_dim {
                    gk[row + c] += ek * xk[c];
                    gv[row + c] += ev * xv[c];
                }
            }
        }
        for i in 0..(dst_dim * src_dim) {
            vel_k[i] = mom * vel_k[i] + lr * gk[i];
            vel_v[i] = mom * vel_v[i] + lr * gv[i];
            mapper.w_k[i] -= vel_k[i];
            mapper.w_v[i] -= vel_v[i];
        }
    }

    let (final_heldout_mse, heldout_cosine_sim) = eval_test(&mapper);
    let mse_reduction_pct = (1.0 - (final_heldout_mse as f64) / (init_heldout_mse as f64).max(1e-12)) * 100.0;

    // Empirical MIT 2x2 Test: Self-context recall query under Augment-Cache vs Destructive Overwrite!
    // Receiver stores a sharp self-fact at token 0 (`k_self[0] = q_self`, `v_self[0] = target_fact`)
    let mut self_kv = KvCache::deterministic(ModelFamily::TemRust, 8, dst_dim, seed + 1, 0x7300);
    let mut q_self = vec![0.0f32; dst_dim];
    let mut v_fact = vec![0.0f32; dst_dim];
    for d in 0..dst_dim {
        q_self[d] = if d % 2 == 0 { 1.5 } else { -1.5 };
        v_fact[d] = if d < dst_dim / 2 { 2.0 } else { -2.0 };
        self_kv.keys[d] = q_self[d];
        self_kv.values[d] = v_fact[d];
    }
    let peer_kv = KvCache::deterministic(ModelFamily::Qwen3, 12, src_dim, seed + 2, 0x7301);
    let mapped_peer = mapper.project_and_compact(&peer_kv, 8);

    // Augment-Cache attends over [self_kv; mapped_peer]
    let (aug_out, _) = augment_cache_attend(&q_self, &self_kv, &mapped_peer);
    let augment_self_recall_cos = cosine_sim(&aug_out, &v_fact);

    // Destructive Overwrite Mutant replaces self_kv with empty self + peer_kv only!
    let empty_self = KvCache { family: ModelFamily::TemRust, seq_len: 0, dim: dst_dim, keys: vec![], values: vec![] };
    let (over_out, _) = augment_cache_attend(&q_self, &empty_self, &mapped_peer);
    let overwrite_self_recall_cos = cosine_sim(&over_out, &v_fact);

    let report = SwatTrainingReport {
        steps,
        init_heldout_mse,
        final_heldout_mse,
        mse_reduction_pct,
        heldout_cosine_sim,
        augment_self_recall_cos,
        overwrite_self_recall_cos,
        augment_beats_overwrite: augment_self_recall_cos > overwrite_self_recall_cos + 0.5,
    };
    (mapper, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trained_swat_recovers_heldout_semantics_and_augment_beats_overwrite() {
        let (_m, rep) = train_and_verify_swat_mapper(32, 24, 140, 5000);
        assert!(rep.mse_reduction_pct > 85.0, "MSE drop too low: {:.1}%", rep.mse_reduction_pct);
        assert!(rep.heldout_cosine_sim > 0.90, "held-out cosine too low: {:.4}", rep.heldout_cosine_sim);
        assert!(rep.augment_beats_overwrite, "Augment ({:.3}) did not beat Overwrite ({:.3})", rep.augment_self_recall_cos, rep.overwrite_self_recall_cos);
    }
}
