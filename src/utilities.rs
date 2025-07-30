use crate::data::get_all_mt_descendants;
/// Utility functions for materials_for_mc

/// Linear interpolation on a linear scale
/// 
/// Given arrays of x and y values, interpolate to find the y value at x_new.
/// If x_new is outside the range of x, returns the first or last y value.
pub fn interpolate_linear(x: &[f64], y: &[f64], x_new: f64) -> f64 {
    // Handle edge cases
    if x_new <= x[0] {
        return y[0];
    }
    if x_new >= x[x.len() - 1] {
        return y[y.len() - 1];
    }
    
    // Find the two points to interpolate between
    let mut idx = 0;
    while idx < x.len() - 1 && x[idx + 1] < x_new {
        idx += 1;
    }
    
    // Linear interpolation formula: y = y1 + (x - x1) * (y2 - y1) / (x2 - x1)
    let x1 = x[idx];
    let x2 = x[idx + 1];
    let y1 = y[idx];
    let y2 = y[idx + 1];
    
    y1 + (x_new - x1) * (y2 - y1) / (x2 - x1)
}

/// Log-log interpolation
/// 
/// Given arrays of x and y values, interpolate on a log-log scale to find the y value at x_new.
/// If x_new is outside the range of x, returns the first or last y value.
/// All x and y values must be positive.
pub fn interpolate_log_log(x: &[f64], y: &[f64], x_new: f64) -> f64 {
    // Handle edge cases
    if x_new <= x[0] {
        return y[0];
    }
    if x_new >= x[x.len() - 1] {
        return y[y.len() - 1];
    }
    
    // Find the two points to interpolate between
    let mut idx = 0;
    while idx < x.len() - 1 && x[idx + 1] < x_new {
        idx += 1;
    }
    
    // Log-log interpolation formula: ln(y) = ln(y1) + ln(x/x1) * ln(y2/y1) / ln(x2/x1)
    let x1 = x[idx];
    let x2 = x[idx + 1];
    let y1 = y[idx];
    let y2 = y[idx + 1];
    
    let log_x1 = x1.ln();
    let log_x2 = x2.ln();
    let log_y1 = y1.ln();
    let log_y2 = y2.ln();
    let log_x_new = x_new.ln();
    
    let log_y_new = log_y1 + (log_x_new - log_x1) * (log_y2 - log_y1) / (log_x2 - log_x1);
    log_y_new.exp()
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

/// Helper to expand a list of MT numbers to include all descendants (for sum rules)
pub fn expand_mt_filter(mt_filter: &Vec<i32>) -> std::collections::HashSet<i32> {
    let mut set = std::collections::HashSet::new();
    for &mt in mt_filter {
        set.insert(mt);
        for child in get_all_mt_descendants(mt) {
            set.insert(child);
        }
    }
    set
}