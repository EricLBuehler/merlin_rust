//Taken from rust's stats.rs
//I have not included their LICENSE as I only took small portions. The code is unchanged.

fn local_sort(v: &mut [f64]) {
    v.sort_by(|x: &f64, y: &f64| x.total_cmp(y));
}

// Helper function: extract a value representing the `pct` percentile of a sorted sample-set, using
// linear interpolation. If samples are not sorted, return nonsensical value.
fn percentile_of_sorted(sorted_samples: &[f64], pct: f64) -> f64 {
    assert!(!sorted_samples.is_empty());
    if sorted_samples.len() == 1 {
        return sorted_samples[0];
    }
    let zero: f64 = 0.0;
    assert!(zero <= pct);
    let hundred = 100_f64;
    assert!(pct <= hundred);
    if pct == hundred {
        return sorted_samples[sorted_samples.len() - 1];
    }
    let length = (sorted_samples.len() - 1) as f64;
    let rank = (pct / hundred) * length;
    let lrank = rank.floor();
    let d = rank - lrank;
    let n = lrank as usize;
    let lo = sorted_samples[n];
    let hi = sorted_samples[n + 1];
    lo + (hi - lo) * d
}

/// Winsorize a set of samples, replacing values above the `100-pct` percentile
/// and below the `pct` percentile with those percentiles themselves. This is a
/// way of minimizing the effect of outliers, at the cost of biasing the sample.
/// It differs from trimming in that it does not change the number of samples,
/// just changes the values of those that are outliers.
///
/// See: <https://en.wikipedia.org/wiki/Winsorising>
pub fn winsorize(samples: &mut [f64], pct: f64) {
    let mut tmp = samples.to_vec();
    local_sort(&mut tmp);
    let lo = percentile_of_sorted(&tmp, pct);
    let hundred = 100_f64;
    let hi = percentile_of_sorted(&tmp, hundred - pct);
    for samp in samples {
        if *samp > hi {
            *samp = hi
        } else if *samp < lo {
            *samp = lo
        }
    }
}