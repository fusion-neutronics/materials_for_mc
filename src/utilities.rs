// ...existing code...
/// Utility functions for materials_for_mc

/// Linear interpolation on a linear scale
/// 
/// Given arrays of x and y values, interpolate to find the y value at x_new.
/// If x_new is outside the range of x, returns the first or last y value.
pub fn interpolate_linear(x: &[f64], y: &[f64], x_new: f64) -> f64 {
    debug_assert_eq!(x.len(), y.len(), "x and y must be same length");
    if x.is_empty() { return f64::NAN; }
    if x.len() == 1 { return y[0]; }
    if x_new <= x[0] { return y[0]; }
    if x_new >= x[x.len()-1] { return y[y.len()-1]; }
    let idx = find_interval_bitwise_f64(x, x_new);
    let x1 = x[idx]; let x2 = x[idx+1]; let y1 = y[idx]; let y2 = y[idx+1];
    y1 + (x_new - x1) * (y2 - y1) / (x2 - x1)
}

/// Log-log interpolation
/// 
/// Given arrays of x and y values, interpolate on a log-log scale to find the y value at x_new.
/// If x_new is outside the range of x, returns the first or last y value.
/// All x and y values must be positive.
pub fn interpolate_log_log(x: &[f64], y: &[f64], x_new: f64) -> f64 {
    debug_assert_eq!(x.len(), y.len(), "x and y must be same length");
    if x.is_empty() { return f64::NAN; }
    if x.len() == 1 { return y[0]; }
    if x_new <= x[0] { return y[0]; }
    if x_new >= x[x.len()-1] { return y[y.len()-1]; }
    let idx = find_interval_bitwise_f64(x, x_new);
    let x1 = x[idx]; let x2 = x[idx+1]; let y1 = y[idx]; let y2 = y[idx+1];
    let log_x1 = x1.ln();
    let log_x2 = x2.ln();
    let log_y1 = y1.ln();
    let log_y2 = y2.ln();
    let log_x_new = x_new.ln();
    let log_y_new = log_y1 + (log_x_new - log_x1) * (log_y2 - log_y1) / (log_x2 - log_x1);
    log_y_new.exp()
}

/// Bitwise binary search interval finder for f64 arrays.
/// Returns i such that x[i] <= x_new < x[i+1]. Assumes:
///  * x sorted ascending
///  * all values finite and non-negative (so IEEE-754 bit ordering matches numeric ordering)
pub fn find_interval_bitwise_f64(x: &[f64], x_new: f64) -> usize {
    debug_assert!(x.len() >= 2);
    let target = x_new.to_bits();
    let mut low: isize = 0;
    let mut high: isize = (x.len() as isize) - 2; // last valid start index
    while low <= high {
        let mid = (low + high) >> 1; // divide by 2
        let m = mid as usize;
        let mid_next_bits = x[m+1].to_bits();
        if mid_next_bits <= target { // x[m+1] <= x_new => move right
            low = mid + 1;
        } else if x[m].to_bits() > target { // x[m] > x_new => move left
            high = mid - 1;
        } else {
            return m; // bracket found
        }
    }
    (low.saturating_sub(1)) as usize
}

/// Attempt O(1) localized interval search using a mutable hint index.
/// Falls back to bitwise binary search if the hint is not correct or adjacent.
pub fn find_interval_with_hint(x: &[f64], x_new: f64, hint: &mut usize) -> usize {
    let n = x.len();
    debug_assert!(n >= 2);
    if *hint >= n-1 { *hint = n-2; }
    // Fast path: if x_new is within current hinted bracket
    if x[*hint] <= x_new && x_new <= x[*hint+1] { return *hint; }
    // Try move right while next upper bound still below x_new (limited steps)
    while *hint + 1 < n-1 && x[*hint+1] < x_new {
        *hint += 1;
        if x[*hint] <= x_new && x_new <= x[*hint+1] { return *hint; }
    }
    // Try move left
    while *hint > 0 && x[*hint] > x_new {
        *hint -= 1;
        if x[*hint] <= x_new && x_new <= x[*hint+1] { return *hint; }
    }
    // Fallback
    let idx = find_interval_bitwise_f64(x, x_new);
    *hint = idx;
    idx
}

/// Precompute slopes ( (y[i+1]-y[i]) / (x[i+1]-x[i]) ) for linear interpolation.
pub fn precompute_slopes(x: &[f64], y: &[f64]) -> Vec<f64> {
    debug_assert_eq!(x.len(), y.len());
    if x.len() < 2 { return Vec::new(); }
    let mut slopes = Vec::with_capacity(x.len()-1);
    for i in 0..x.len()-1 { slopes.push( (y[i+1]-y[i]) / (x[i+1]-x[i]) ); }
    slopes
}

/// Fast linear interpolation using precomputed slopes and mutable hint.
pub fn interpolate_linear_fast(x: &[f64], y: &[f64], slopes: &[f64], x_new: f64, hint: &mut usize) -> f64 {
    if x.is_empty() { return f64::NAN; }
    if x.len() == 1 { return y[0]; }
    if x_new <= x[0] { *hint = 0; return y[0]; }
    if x_new >= x[x.len()-1] { *hint = x.len()-2; return y[y.len()-1]; }
    let i = find_interval_with_hint(x, x_new, hint);
    y[i] + (x_new - x[i]) * slopes[i]
}

/// Helper for dependency-ordered processing of MTs (children before parents)
pub fn add_to_processing_order(
    mt: i32,
    sum_rules: &std::collections::HashMap<i32, Vec<i32>>,
    processed: &mut std::collections::HashSet<i32>,
    order: &mut Vec<i32>,
    restrict: &std::collections::HashSet<i32>,
) {
    if processed.contains(&mt) || !restrict.contains(&mt) {
        return;
    }
    if let Some(constituents) = sum_rules.get(&mt) {
        for &constituent in constituents {
            add_to_processing_order(constituent, sum_rules, processed, order, restrict);
        }
    }
    processed.insert(mt);
    order.push(mt);
}
