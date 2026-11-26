//! Eshkol-Style Zero-Finite-Difference Exact Forward + Reverse + Symbolic Automatic Differentiation
//! (`crates/beast-model/src/eshkol_ad.rs` — closes `P067`, `F063`, `F099`).
//!
//! Proves 3 independent exact differentiation modes agree to machine epsilon (`< 1e-13`) on a
//! non-linear mutated Chimera activation `f(x) = silu(1.3*x - 0.4) * tanh(0.7*x) + sin(x^2)`:
//!   1. Forward-Mode Dual Numbers (`Dual { val, deriv }`)
//!   2. Reverse-Mode Wengert Tape (`Tape`)
//!   3. Symbolic AST Differentiation (`Expr::diff("x")` -> algebraic AST -> exact eval)

use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Dual {
    pub val: f64,
    pub deriv: f64,
}

impl Dual {
    pub fn var(x: f64) -> Self { Self { val: x, deriv: 1.0 } }
    pub fn cst(c: f64) -> Self { Self { val: c, deriv: 0.0 } }
    pub fn sin(self) -> Self { Self { val: self.val.sin(), deriv: self.val.cos() * self.deriv } }
    pub fn exp(self) -> Self { let e = self.val.exp(); Self { val: e, deriv: e * self.deriv } }
    pub fn tanh(self) -> Self {
        let t = self.val.tanh();
        Self { val: t, deriv: (1.0 - t * t) * self.deriv }
    }
    pub fn silu(self) -> Self {
        let sig = 1.0 / (1.0 + (-self.val).exp());
        let d_silu = sig * (1.0 + self.val * (1.0 - sig));
        Self { val: self.val * sig, deriv: d_silu * self.deriv }
    }
}

impl Add for Dual {
    type Output = Self;
    fn add(self, r: Self) -> Self { Self { val: self.val + r.val, deriv: self.deriv + r.deriv } }
}
impl Sub for Dual {
    type Output = Self;
    fn sub(self, r: Self) -> Self { Self { val: self.val - r.val, deriv: self.deriv - r.deriv } }
}
impl Mul for Dual {
    type Output = Self;
    fn mul(self, r: Self) -> Self {
        Self { val: self.val * r.val, deriv: self.deriv * r.val + self.val * r.deriv }
    }
}

// ---- Reverse-Mode Wengert Tape ----
#[derive(Debug, Clone)]
struct TapeNode {
    val: f64,
    parents: [(usize, f64); 2],
    n_parents: usize,
}

#[derive(Debug, Default)]
pub struct ReverseTape {
    nodes: Vec<TapeNode>,
}

impl ReverseTape {
    pub fn var(&mut self, v: f64) -> usize {
        let id = self.nodes.len();
        self.nodes.push(TapeNode { val: v, parents: [(0, 0.0); 2], n_parents: 0 });
        id
    }
    pub fn add(&mut self, a: usize, b: usize) -> usize {
        let id = self.nodes.len();
        let val = self.nodes[a].val + self.nodes[b].val;
        self.nodes.push(TapeNode { val, parents: [(a, 1.0), (b, 1.0)], n_parents: 2 });
        id
    }
    pub fn scale_shift(&mut self, a: usize, k: f64, b: f64) -> usize {
        let id = self.nodes.len();
        let val = k * self.nodes[a].val + b;
        self.nodes.push(TapeNode { val, parents: [(a, k), (0, 0.0)], n_parents: 1 });
        id
    }
    pub fn mul(&mut self, a: usize, b: usize) -> usize {
        let id = self.nodes.len();
        let va = self.nodes[a].val;
        let vb = self.nodes[b].val;
        self.nodes.push(TapeNode { val: va * vb, parents: [(a, vb), (b, va)], n_parents: 2 });
        id
    }
    pub fn sin(&mut self, a: usize) -> usize {
        let id = self.nodes.len();
        let va = self.nodes[a].val;
        self.nodes.push(TapeNode { val: va.sin(), parents: [(a, va.cos()), (0, 0.0)], n_parents: 1 });
        id
    }
    pub fn tanh(&mut self, a: usize) -> usize {
        let id = self.nodes.len();
        let t = self.nodes[a].val.tanh();
        self.nodes.push(TapeNode { val: t, parents: [(a, 1.0 - t * t), (0, 0.0)], n_parents: 1 });
        id
    }
    pub fn silu(&mut self, a: usize) -> usize {
        let id = self.nodes.len();
        let v = self.nodes[a].val;
        let sig = 1.0 / (1.0 + (-v).exp());
        let d = sig * (1.0 + v * (1.0 - sig));
        self.nodes.push(TapeNode { val: v * sig, parents: [(a, d), (0, 0.0)], n_parents: 1 });
        id
    }
    pub fn backward(&self, root: usize) -> Vec<f64> {
        let mut adj = vec![0.0f64; self.nodes.len()];
        adj[root] = 1.0;
        for i in (0..=root).rev() {
            let g = adj[i];
            for k in 0..self.nodes[i].n_parents {
                let (p, local_d) = self.nodes[i].parents[k];
                adj[p] += g * local_d;
            }
        }
        adj
    }
}

// ---- Symbolic AST Differentiator ----
#[derive(Debug, Clone)]
pub enum SymExpr {
    Const(f64),
    Var,
    Add(Box<SymExpr>, Box<SymExpr>),
    Mul(Box<SymExpr>, Box<SymExpr>),
    Sin(Box<SymExpr>),
    Cos(Box<SymExpr>),
    Tanh(Box<SymExpr>),
    Silu(Box<SymExpr>),
    Sigmoid(Box<SymExpr>),
}

impl SymExpr {
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Self::Const(c) => *c,
            Self::Var => x,
            Self::Add(a, b) => a.eval(x) + b.eval(x),
            Self::Mul(a, b) => a.eval(x) * b.eval(x),
            Self::Sin(u) => u.eval(x).sin(),
            Self::Cos(u) => u.eval(x).cos(),
            Self::Tanh(u) => u.eval(x).tanh(),
            Self::Sigmoid(u) => 1.0 / (1.0 + (-u.eval(x)).exp()),
            Self::Silu(u) => {
                let v = u.eval(x);
                v / (1.0 + (-v).exp())
            }
        }
    }

    pub fn diff(&self) -> SymExpr {
        match self {
            Self::Const(_) => Self::Const(0.0),
            Self::Var => Self::Const(1.0),
            Self::Add(a, b) => Self::Add(Box::new(a.diff()), Box::new(b.diff())),
            Self::Mul(a, b) => Self::Add(
                Box::new(Self::Mul(Box::new(a.diff()), b.clone())),
                Box::new(Self::Mul(a.clone(), Box::new(b.diff()))),
            ),
            Self::Sin(u) => Self::Mul(Box::new(Self::Cos(u.clone())), Box::new(u.diff())),
            Self::Cos(u) => Self::Mul(
                Box::new(Self::Mul(Box::new(Self::Const(-1.0)), Box::new(Self::Sin(u.clone())))),
                Box::new(u.diff()),
            ),
            Self::Tanh(u) => {
                let t = Self::Tanh(u.clone());
                let t2 = Self::Mul(Box::new(t.clone()), Box::new(t));
                let one_minus_t2 = Self::Add(Box::new(Self::Const(1.0)), Box::new(Self::Mul(Box::new(Self::Const(-1.0)), Box::new(t2))));
                Self::Mul(Box::new(one_minus_t2), Box::new(u.diff()))
            }
            Self::Sigmoid(u) => {
                let s = Self::Sigmoid(u.clone());
                let one_minus_s = Self::Add(Box::new(Self::Const(1.0)), Box::new(Self::Mul(Box::new(Self::Const(-1.0)), Box::new(s.clone()))));
                Self::Mul(Box::new(Self::Mul(Box::new(s), Box::new(one_minus_s))), Box::new(u.diff()))
            }
            Self::Silu(u) => {
                // d/dx [u * sig(u)] = sig(u) * (1 + u * (1 - sig(u))) * du/dx
                let s = Self::Sigmoid(u.clone());
                let one_minus_s = Self::Add(Box::new(Self::Const(1.0)), Box::new(Self::Mul(Box::new(Self::Const(-1.0)), Box::new(s.clone()))));
                let inner = Self::Add(Box::new(Self::Const(1.0)), Box::new(Self::Mul(u.clone(), Box::new(one_minus_s))));
                Self::Mul(Box::new(Self::Mul(Box::new(s), Box::new(inner))), Box::new(u.diff()))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EshkolAdReport {
    pub x: f64,
    pub value: f64,
    pub forward_deriv: f64,
    pub reverse_deriv: f64,
    pub symbolic_deriv: f64,
    pub max_tri_discrepancy: f64,
    pub exact_match: bool,
}

/// Evaluates `f(x) = silu(1.3*x - 0.4) * tanh(0.7*x) + sin(x * x)` via ALL THREE exact AD modes!
pub fn verify_eshkol_tri_mode_ad(x: f64) -> Result<EshkolAdReport, String> {
    // 1. Forward Dual
    let dx = Dual::var(x);
    let f_dual = (Dual::cst(1.3) * dx - Dual::cst(0.4)).silu()
        * (Dual::cst(0.7) * dx).tanh()
        + (dx * dx).sin();

    // 2. Reverse Wengert Tape
    let mut tape = ReverseTape::default();
    let ix = tape.var(x);
    let i_lin1 = tape.scale_shift(ix, 1.3, -0.4);
    let i_silu = tape.silu(i_lin1);
    let i_lin2 = tape.scale_shift(ix, 0.7, 0.0);
    let i_tanh = tape.tanh(i_lin2);
    let i_prod = tape.mul(i_silu, i_tanh);
    let i_x2 = tape.mul(ix, ix);
    let i_sin = tape.sin(i_x2);
    let i_out = tape.add(i_prod, i_sin);
    let grads = tape.backward(i_out);
    let rev_d = grads[ix];

    // 3. Symbolic AST Differentiation
    let expr = SymExpr::Add(
        Box::new(SymExpr::Mul(
            Box::new(SymExpr::Silu(Box::new(SymExpr::Add(
                Box::new(SymExpr::Mul(Box::new(SymExpr::Const(1.3)), Box::new(SymExpr::Var))),
                Box::new(SymExpr::Const(-0.4)),
            )))),
            Box::new(SymExpr::Tanh(Box::new(SymExpr::Mul(
                Box::new(SymExpr::Const(0.7)),
                Box::new(SymExpr::Var),
            )))),
        )),
        Box::new(SymExpr::Sin(Box::new(SymExpr::Mul(
            Box::new(SymExpr::Var),
            Box::new(SymExpr::Var),
        )))),
    );
    let sym_deriv_ast = expr.diff();
    let sym_d = sym_deriv_ast.eval(x);

    let d1 = (f_dual.deriv - rev_d).abs();
    let d2 = (f_dual.deriv - sym_d).abs();
    let d3 = (rev_d - sym_d).abs();
    let max_disc = d1.max(d2).max(d3);
    let exact_match = max_disc < 1e-13 && (expr.eval(x) - f_dual.val).abs() < 1e-13;
    if !exact_match {
        return Err(format!("Eshkol tri-mode AD mismatch: fwd={} rev={} sym={} err={max_disc:.2e}", f_dual.deriv, rev_d, sym_d));
    }

    Ok(EshkolAdReport {
        x,
        value: f_dual.val,
        forward_deriv: f_dual.deriv,
        reverse_deriv: rev_d,
        symbolic_deriv: sym_d,
        max_tri_discrepancy: max_disc,
        exact_match,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eshkol_forward_reverse_symbolic_agree_to_machine_eps() {
        let rep = verify_eshkol_tri_mode_ad(0.83).expect("tri-mode ad");
        assert!(rep.exact_match);
        assert!(rep.max_tri_discrepancy < 1e-13);
    }
}
