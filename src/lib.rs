#![warn(missing_debug_implementations, rust_2018_idioms)]
//!
//! The RBO indefinite rank similarity metric.
//!
//! This code implements the RBO metric, as described in:
//!
//! @article{wmz10:acmtois,
//!     author = "Webber, William and Moffat, Alistair and Zobel, Justin",
//!     title = "A similarity measure for indefinite rankings",
//!     journal = "ACM Transactions on Information Systems",
//!     year = {2010},
//!     note = "to appear",
//! }
//!
//!
//!
//! The fundamental step in the working of RBO is the calculation
//! of overlap `X_d`, or size of intersection, between the two rankings
//! at each depth.  The key insight is that:
//!
//!    $X_{d+1} = X_{d} + I(S_{d+1} \in T_{1:{d+1}})
//!                     + I(T_{d+1} \in S_{1:{d+1}})
//!
//! where $S$ and $T$ are the two lists, and $I$ is the indicator function,
//! return $1$ if the enclosed statement is true, $0$ otherwise.
//! That is, the overlap at the next depth is the overlap at the current
//! depth, plus one each if the next element in one list is found by
//! the next depth in the other list.  To implement this efficiently,
//! we keep a lookup set of the elements encountered in each list to date.
//! Note that we do not require separate lookup sets for each list: we
//! only record elements if they've only been encountered once.
//!
//! This code and docs were adapted from the original RBO codebase of William Webber
//!
//! ```
//! use rbo::rbo;
//!
//! let first = [1, 2, 3];
//! let second = [1, 3, 2];
//! let rbo_val = rbo(&first,&second,0.9);
//! ```

mod state;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RboError {
    #[error("Persistance parameter p must be 0.0 <= p < 1.0")]
    InvalidPersistance,
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
    fn permute_comparison_to_william_webber() {
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
    fn uneven_comparison_to_william_webber() {
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("test_data/uneven_abc.json");
        let test_data_file = std::fs::File::open(d).expect("open test data file");
        let test_cases: Vec<TestCase> =
            serde_json::from_reader(&test_data_file).expect("read test data");
        let first = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
        for t in test_cases {
            let second = t.other.chars().collect::<Vec<_>>();
            println!("first {:?} , second {:?}, p {}", &first, &second, t.p);
            let computed_rbo = super::rbo(&first, &second, t.p).expect("valid rbo");
            approx::assert_abs_diff_eq!(computed_rbo.extrapolated, t.rbo, epsilon = 0.001);
        }
    }
}
