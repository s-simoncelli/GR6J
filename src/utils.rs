use ndarray::Array;

/// Get the series max value
pub(crate) fn series_max(series: &[f64]) -> f64 {
    *series.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

/// Get the series min value
pub(crate) fn series_min(series: &[f64]) -> f64 {
    *series.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

/// Calculate the flow duration curve.
///
/// # Arguments
///
/// * `run_off`: The run-off data.
///
/// returns: (Vec<f64>, Vec<f64>) The probability of exceedence (0-190) and the sorted run-off data.
pub(crate) fn calculate_fdc(run_off: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let exceedence = Array::range(1., run_off.len() as f64 + 1.0, 1.0) / run_off.len() as f64 * 100.0;

    let mut sorted_run_off: Vec<f64> = (*run_off).to_vec().clone();
    sorted_run_off.sort_by(|x: &f64, y: &f64| y.total_cmp(x));
    (exceedence.to_vec(), sorted_run_off.to_vec())
}
