use std::collections::HashSet;
use std::hash::Hash;

const VALID_P_RANGE: std::ops::Range<f64> = 0.0..1.0;

pub(crate) struct RboState<'a, Item: Eq + Hash> {
    // the items we have seen so far
    seen: HashSet<&'a Item>,
    // depth is the current depth, counting from 1.
    depth_long: f64,
    // depth is the current depth, counting from 1.
    depth_short: f64,
    // the current overlap.
    cur_overlap: f64,
    // the current overlap (X_d in the paper).
    overlap: Vec<f64>,
    // the p value being used.
    persistence: f64,
}

impl<'a, Item: Eq + Hash> RboState<'a, Item> {
    // Initialize the RBO state with persistance `p`
    pub(crate) fn with_persistence(p: f64) -> Result<Self, crate::RboError> {
        if !VALID_P_RANGE.contains(&p) {
            return Err(crate::RboError::InvalidPersistance);
        }
        Ok(Self {
            seen: HashSet::with_capacity(4096),
            depth_long: 0.0,
            depth_short: 0.0,
            cur_overlap: 0.0,
            overlap: vec![0.0],
            persistence: p,
        })
    }

    // Update the RBO state with two new elements.
    pub(crate) fn update(&mut self, first: &'a Item, second: Option<&'a Item>) {
        match second.map(|s| s.eq(first)) {
            Some(true) => {
                self.depth_short += 1.0;
                self.cur_overlap += 1.0;
            }
            Some(false) => {
                self.depth_short += 1.0;
                for item in [first, second.unwrap()] {
                    if self.seen.remove(item) {
                        // have we seen this before
                        self.cur_overlap += 1.0;
                    } else {
                        self.seen.insert(item);
                    }
                }
            }
            None => {
                if self.seen.remove(first) {
                    // have we seen this before
                    self.cur_overlap += 1.0;
                }
            }
        }
        self.overlap.push(self.cur_overlap);
        self.depth_long += 1.0;
    }

    // compute quation 30 for RBO_res
    fn compute_residual(&mut self) -> f64 {
        let s = self.depth_short;
        let us = s as usize;
        let l = self.depth_long;
        let ul = l as usize;
        let x_l = self.cur_overlap;
        // the rank at which maximum agreement becomes 1
        let f = s + l - x_l;
        let p = self.persistence;
        let uf = f as usize;
        let sum_s: f64 = (us + 1..=uf).map(|d| p.powf(d as f64) / d as f64).sum();
        let sum_l: f64 = (ul + 1..=uf).map(|d| p.powf(d as f64) / d as f64).sum();
        let sum_t: f64 = (1..=uf).map(|i| (p.powf(i as f64) / i as f64)).sum();
        let p_s = p.powf(s);
        let p_l = p.powf(l);
        let p_f = p.powf(f);
        let ln_1p = (1.0 / (1.0 - p)).ln();
        p_s + p_l - p_f - ((1.0 - p) / p) * (s * sum_s + l * sum_l + x_l * (ln_1p - sum_t))
    }

    // equation 11 in the paper
    fn compute_min(&self) -> f64 {
        let p = self.persistence;
        let k = self.depth_short as usize;
        let x_k = self.overlap[k];
        let x_d = &self.overlap;
        let other: f64 = (1..k)
            .map(|d| (x_d[d] - x_k) * p.powf(d as f64) / d as f64)
            .sum();
        (1.0 - p) / p * (other - (x_k * (1.0 - p).ln()))
    }

    // equation 32 in the paper
    fn compute_extrapolated(&self) -> f64 {
        let p = self.persistence;
        let l = self.depth_long as usize;
        let p_l = p.powf(l as f64);
        let s = self.depth_short as usize;
        let x_s = self.overlap[s];
        let x_l = self.overlap[l];
        let x_d = &self.overlap;
        let first: f64 = (1..=l).map(|d| x_d[d] * p.powf(d as f64) / d as f64).sum();
        let second: f64 = (s + 1..=l)
            .map(|d| (x_s * (d - s) as f64) / (s * d) as f64 * p.powf(d as f64))
            .sum();
        let third = (x_l - x_s) / l as f64 + (x_s / s as f64) * p_l;
        (1.0 - p) / p * (first + second) + third
    }

    // we extrapolate the final RBO value and compute the residual
    pub(crate) fn into_result(mut self) -> crate::Rbo {
        crate::Rbo {
            min: self.compute_min(),
            residual: self.compute_residual(),
            extrapolated: self.compute_extrapolated(),
        }
    }
}
