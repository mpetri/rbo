# Rank-Biased Overlap (RBO) [![Crates.io][crates-badge]][crates-url] [![Docs.rs][docs-badge]][docs-rs] [![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/rbo.svg
[crates-url]: https://crates.io/crates/rbo
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://opensource.org/licenses/MIT
[docs-rs]: https://docs.rs/rbo
[docs-badge]: https://img.shields.io/docsrs/rbo/0.2.6

The RBO indefinite rank similarity metric.

This code implements the RBO metric, as described in:

```
@article{wmz10:acmtois,
    author = "Webber, William and Moffat, Alistair and Zobel, Justin",
    title = "A similarity measure for indefinite rankings",
    journal = "ACM Transactions on Information Systems",
    year = {2010},
}
```
# What is RBO (taken from the paper)

The rank-biased overlap (RBO) measure is based on a simple probabilistic user
model. This measure is based on (but is not tied to) a simple user model in
which the user compares the overlap of the two rankings at incrementally
increasing depths. The user has a certain level of patience, parameterized
in the model, and after examining each depth has a fixed probability of stopping,
modelled as a Bernoulli random variable. RBO is then calculated as the
expected average overlap that the user observes in comparing the two lists. The measure
takes a parameter that specifies the user’s persistence `p`, that is, the probability that the user,
having examined the overlap at one rank, continues on to consider the overlap at the next.

The (convergent) sum of the weights of the (potentially infinite) tail determines the
gap or `residual` between the `minimum` and maximum similarity scores that could be attained
on exhaustive evaluation. The minimum, maximum, and residual scores on partial RBO evaluation
are all monotonic in depth. A point score can also be `extrapolated`.

# Usage

Either via `cargo install`

```
cargo install rbo
./rbo -p 0.8 first_list.txt second_list.txt
```

or as a library call

```rust
use rbo::rbo;

let first = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<_>>();
let second = "kxcnarvmwyp".chars().collect::<Vec<_>>();
let rbo_val = rbo(&first,&second,0.99).expect("valid rbo");
println!("{}",rbo_val);
```

# Correctness

This code tests against the original `rbo_ext` implementation by William Webber and
against another reference implementation for `rbo_min` and `rbo_res`.

# License

MIT
