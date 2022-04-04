#![warn(missing_debug_implementations, rust_2018_idioms)]
//!
//! The RBO indefinite rank similarity metric.
//!
//! This code implements the RBO metric, as described in:
//!
//! ```
//! @article{wmz10:acmtois,
//!     author = "Webber, William and Moffat, Alistair and Zobel, Justin",
//!     title = "A similarity measure for indefinite rankings",
//!     journal = "ACM Transactions on Information Systems",
//!     year = {2010},
//! }
//!```
//! # What is RBO (taken from the paper)
//!
//! The rank-biased overlap (RBO) measure is based on a simple probabilistic user
//! model. This measure is based on (but is not tied to) a simple user model in
//! which the user compares the overlap of the two rankings at incrementally
//! increasing depths. The user has a certain level of patience, parameterized
//! in the model, and after examining each depth has a fixed probability of stopping,
//! modelled as a Bernoulli random variable. RBO is then calculated as the
//! expected average overlap that the user observes in comparing the two lists. The measure
//! takes a parameter that specifies the user’s persistence `p`, that is, the probability that the user,
//! having examined the overlap at one rank, continues on to consider the overlap at the next.
//!
//! The (convergent) sum of the weights of the (potentially infinite) tail determines the
//! gap or `residual` between the `minimum` and maximum similarity scores that could be attained
//! on exhaustive evaluation. The minimum, maximum, and residual scores on partial RBO evaluation
//! are all monotonic in depth. A point score can also be `extrapolated`.
//!
//! # Correctness
//!
//! This code tests against the original `rbo_ext` implementation by William Webber and
//! against another reference implementation for `rbo_min` and `rbo_res`.
//!
//! # Example:
//!
//! ```
//! use rbo::rbo;
//!
//! let first = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
//! let second = "kxcnarvmwyp".chars().collect::<Vec<_>>();
//! let rbo_val = rbo(&first,&second,0.99).expect("valid rbo");
//! println!("{}",rbo_val);
//! ```

mod state;

use thiserror::Error;

/// Different RBO error conditions
#[derive(Error, Debug)]
pub enum RboError {
    /// Persistance parameter p must be 0.0 <= p < 1.0
    #[error("Persistance parameter p must be 0.0 <= p < 1.0")]
    InvalidPersistance,
    /// Individual ranked lists should not contain duplicates
    #[error("Individual ranked lists should not contain duplicates")]
    DuplicatesInList,
}

use state::RboState;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug)]
/// The result of the RBO computation
pub struct Rbo {
    /// Lower bound estimate of RBO (RBO_min in paper)
    pub min: f64,
    /// residual uncertainty attendant upon prefix, rather than full, evaluation
    /// Residual corresponding to min; min + res is an upper bound estimate
    pub residual: f64,
    /// point estimate by extrapolation from the visible lists,
    /// assuming that the degree of agreement seen up to depth k is continued indefinitely
    pub extrapolated: f64,
}

impl std::fmt::Display for Rbo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RBO(min={:.3},residual={:.3}, extrapolated={:.3})",
            self.min, self.residual, self.extrapolated
        )
    }
}

fn contains_duplicates<Item>(list: &[Item]) -> bool
where
    Item: Eq + Hash,
{
    let hash_set: std::collections::HashSet<_> = list.iter().collect();
    hash_set.len() != list.len()
}

///
/// Main RBO function implementing the computation of Rank-Biased Overlap
///
/// # Errors
///
/// - Will return `Err` if `p` is not 0 <= p < 1
/// - Will return `Err` if lists contain duplicate items
///
pub fn rbo<Item>(first: &[Item], second: &[Item], p: f64) -> Result<Rbo, RboError>
where
    Item: Eq + Hash,
{
    let mut rbo_state = RboState::with_persistence(p)?;

    // ensure we have no duplicates in lists first
    if contains_duplicates(first) || contains_duplicates(second) {
        return Err(crate::RboError::DuplicatesInList);
    }

    for (a, b) in first.iter().zip(second) {
        rbo_state.update(a, Some(b));
    }
    // ensure we process the remainder if unequal lists
    let remainder = match first.len().cmp(&second.len()) {
        Ordering::Less => Some(second.iter().skip(first.len())),
        Ordering::Equal => None,
        Ordering::Greater => Some(first.iter().skip(second.len())),
    };
    if let Some(items) = remainder {
        for item in items {
            rbo_state.update(item, None);
        }
    }
    // finalize
    Ok(rbo_state.into_result())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_residual() {
        let first: Vec<char> = "abcdefg".chars().collect();
        let second: Vec<char> = "abcdefg".chars().collect();
        let computed_rbo = super::rbo(&first, &second, 0.9).expect("valid rbo");
        approx::assert_abs_diff_eq!(computed_rbo.residual, 0.232_860, epsilon = 0.000_001_1);
        approx::assert_abs_diff_eq!(
            computed_rbo.residual + computed_rbo.min,
            1.0,
            epsilon = 0.000_001
        );
    }

    #[test]
    fn test_residual_uneven() {
        let first: Vec<char> = "abcdefg".chars().collect();
        let second: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        let computed_rbo = super::rbo(&first, &second, 0.9).expect("valid rbo");
        approx::assert_abs_diff_eq!(computed_rbo.residual, 0.232_860, epsilon = 0.000_001);
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct TestCase {
        other: String,
        p: f64,
        rbo: f64,
    }

    #[test]
    fn ext_permute_comparison_to_william_webber() {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_data/permute_abc.json");
        let test_data_file = std::fs::File::open(d).expect("open test data file");
        let test_cases: Vec<TestCase> =
            serde_json::from_reader(&test_data_file).expect("read test data");

        for t in test_cases {
            let second = t.other.chars().collect::<Vec<_>>();
            let mut first = second.clone();
            first.sort_unstable();
            let computed_rbo = super::rbo(&first, &second, t.p).expect("valid rbo");
            approx::assert_abs_diff_eq!(computed_rbo.extrapolated, t.rbo, epsilon = 0.001);
        }
    }

    #[test]
    fn ext_uneven_comparison_to_william_webber() {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_data/uneven_abc.json");
        let test_data_file = std::fs::File::open(d).expect("open test data file");
        let test_cases: Vec<TestCase> =
            serde_json::from_reader(&test_data_file).expect("read test data");
        let first = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
        for t in test_cases {
            let second = t.other.chars().collect::<Vec<_>>();
            let computed_rbo = super::rbo(&first, &second, t.p).expect("valid rbo");
            approx::assert_abs_diff_eq!(computed_rbo.extrapolated, t.rbo, epsilon = 0.001);
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct TestCaseSp {
        other: String,
        p: f64,
        rbo_ext: f64,
        rbo_min: f64,
        rbo_res: f64,
    }

    #[test]
    fn uneven_comparison_to_sp() {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_data/uneven_abc_sp.json");
        let test_data_file = std::fs::File::open(d).expect("open test data file");
        let test_cases: Vec<TestCaseSp> =
            serde_json::from_reader(&test_data_file).expect("read test data");
        let first = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
        for t in test_cases {
            let second = t.other.chars().collect::<Vec<_>>();
            let computed_rbo = super::rbo(&first, &second, t.p).expect("valid rbo");
            dbg!(&first, &second, &computed_rbo, t.p);
            approx::assert_abs_diff_eq!(computed_rbo.extrapolated, t.rbo_ext, epsilon = 0.001);
            approx::assert_abs_diff_eq!(computed_rbo.min, t.rbo_min, epsilon = 0.001);
            approx::assert_abs_diff_eq!(computed_rbo.residual, t.rbo_res, epsilon = 0.001);
        }
    }
}
