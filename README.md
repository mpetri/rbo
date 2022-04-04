# Rank-Biased Overlap (RBO) 

The RBO indefinite rank similarity metric.

This code implements the RBO metric, as described in:

```
@article{wmz10:acmtois,
    author = "Webber, William and Moffat, Alistair and Zobel, Justin",
    title = "A similarity measure for indefinite rankings",
    journal = "ACM Transactions on Information Systems",
    year = {2010},
    note = "to appear",
}
```


The fundamental step in the working of RBO is the calculation
of overlap `X_d`, or size of intersection, between the two rankings
at each depth.  The key insight is that:

   $X_{d+1} = X_{d} + I(S_{d+1} \in T_{1:{d+1}})
                    + I(T_{d+1} \in S_{1:{d+1}})

where $S$ and $T$ are the two lists, and $I$ is the indicator function,
return $1$ if the enclosed statement is true, $0$ otherwise.
That is, the overlap at the next depth is the overlap at the current
depth, plus one each if the next element in one list is found by
the next depth in the other list.  To implement this efficiently,
we keep a lookup set of the elements encountered in each list to date.
Note that we do not require separate lookup sets for each list: we
only record elements if they've only been encountered once.

This code and docs were adapted from the original RBO codebase of William Webber

```rust
use rbo::rbo;

let first = [1, 2, 3];
let second = [1, 3, 2];
let rbo_val = rbo(&first,&second,0.9);
println!("{}",rbo_val);
```

# Correctness

This code tests against the original `rbo_ext` implementation by William Webber and 
against another reference implementation for `rbo_min` and `rbo_res`.

# License

MIT
