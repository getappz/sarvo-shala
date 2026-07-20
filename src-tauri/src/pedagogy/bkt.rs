use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub p_init: f64,
    pub p_learn: f64,
    pub p_slip: f64,
    pub p_guess: f64,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            p_init: 0.2,
            p_learn: 0.15,
            p_slip: 0.1,
            p_guess: 0.25,
        }
    }
}

/// Standard Bayesian Knowledge Tracing update.
pub fn update(p_known: f64, correct: bool, p: &Params) -> f64 {
    let post = if correct {
        let num = p_known * (1.0 - p.p_slip);
        num / (num + (1.0 - p_known) * p.p_guess)
    } else {
        let num = p_known * p.p_slip;
        num / (num + (1.0 - p_known) * (1.0 - p.p_guess))
    };
    (post + (1.0 - post) * p.p_learn).clamp(0.01, 0.99)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_answer_raises() {
        let p = Params::default();
        assert!(update(0.2, true, &p) > 0.2);
    }

    #[test]
    fn wrong_answer_lowers() {
        let p = Params::default();
        assert!(update(0.8, false, &p) < 0.8);
    }

    #[test]
    fn clamped() {
        let p = Params::default();
        assert!(update(0.99, true, &p) <= 0.99);
        assert!(update(0.01, false, &p) >= 0.01);
    }

    #[test]
    fn converges() {
        let p = Params::default();
        let mut pk = 0.2;
        for _ in 0..20 {
            pk = update(pk, true, &p);
        }
        assert!(pk > 0.9);
    }
}
