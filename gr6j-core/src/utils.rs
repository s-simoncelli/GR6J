use csv::Writer;
use float_cmp::{approx_eq, F64Margin};
use ndarray::Array;
use std::path::PathBuf;

/// Get the series max value
pub(crate) fn series_max(series: &[f64]) -> f64 {
    *series
        .iter()
        .max_by(|a, b| a.total_cmp(b))
        .expect("Cannot calculated max value")
}

/// Get the series min value
pub(crate) fn series_min(series: &[f64]) -> f64 {
    *series
        .iter()
        .min_by(|a, b| a.total_cmp(b))
        .expect("Cannot calculated min value")
}

/// Check if a vector contains NaN and returns the indices containing invalid nu,bers.
///
/// # Arguments
///
/// * `data`: The vector to check.
///
/// returns: Vec<String>
pub(crate) fn vector_nan_indices(data: &[f64]) -> Vec<String> {
    return data
        .iter()
        .enumerate()
        .filter(|(_, &r)| r.is_nan())
        .map(|(index, _)| index.to_string())
        .collect::<Vec<_>>();
}

/// Calculate the flow duration curve
#[derive(Clone)]
pub struct Fdc {
    /// The probability of exceedence (0-100)
    pub exceedence: Vec<f64>,
    /// The sorted run-off data to plot against the probability.
    pub sorted_run_off: Vec<f64>,
}

impl Fdc {
    /// Calculate the flow duration curve.
    ///
    /// # Arguments
    ///
    /// * `run_off`: The run-off time series.
    ///
    /// returns: Fdc
    pub fn new(run_off: &[f64]) -> Self {
        let exceedence = Array::range(1., run_off.len() as f64 + 1.0, 1.0) / run_off.len() as f64 * 100.0;
        let sorted_run_off = NaNVec(run_off).sort(SortType::Asc);

        Self {
            exceedence: exceedence.to_vec(),
            sorted_run_off: sorted_run_off.to_vec(),
        }
    }

    /// Export theflow duration curve and to a CSV file.
    ///
    /// # Arguments
    ///
    /// * `destination`: The destination CSV file.
    /// * `run_off_unit`: The run-off unit of measurement.
    ///
    /// returns: Result<(), csv::Error>
    pub fn to_csv(&self, destination: &PathBuf, run_off_unit: &str) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_path(destination)?;
        wtr.write_record(["Percentage exceedance", format!("Run-off ({})", run_off_unit).as_str()])?;

        for (pct, q) in self.exceedence.iter().zip(&self.sorted_run_off) {
            wtr.write_record([pct.to_string(), q.to_string()])?;
            wtr.flush()?;
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum SortType {
    Asc,
    Desc,
}

/// Operations on vector containing NaNs.
pub struct NaNVec<'a>(pub &'a [f64]);

impl NaNVec<'_> {
    /// Sort a vector of f64 using the IEEE 754 (2008 revision) floating point standard. This creates
    /// a new vector
    ///
    /// # Arguments
    ///
    /// * `vec`: The vector to sort.
    ///
    /// returns: `Vec<f64>`
    pub fn sort(&self, sort_type: SortType) -> Vec<f64> {
        let mut sorted_vec: Vec<f64> = (*self.0).to_vec().clone();
        sorted_vec.sort_by(|x: &f64, y: &f64| y.total_cmp(x));

        if sort_type == SortType::Desc {
            sorted_vec.reverse();
        }
        sorted_vec
    }

    /// Calculate the mean of a vector and excludes NaN values.
    ///
    /// # Arguments
    ///
    /// * `vec`: The vector to calculate the average of.
    ///
    /// returns: f64
    pub fn mean(&self) -> f64 {
        let nan_free_vec = self.remove_nans();
        let total = nan_free_vec.len() as f64;
        let sum: f64 = nan_free_vec.into_iter().sum();
        sum / total
    }

    /// Calculate the standard deviation of a vector and excludes NaN values.
    ///
    /// # Arguments
    ///
    /// * `vec`: The vector to calculate the standard deviation of.
    ///
    /// returns: f64
    pub fn std(&self) -> f64 {
        let nan_free_vec = self.remove_nans();
        let total = nan_free_vec.len() as f64;
        let mean = self.mean();

        let delta_sum: f64 = nan_free_vec.iter().map(|&x| (x - mean).powi(2)).sum();
        (delta_sum / total).powf(0.5)
    }

    /// Removed NaNs from a vector.
    ///
    /// # Arguments
    ///
    /// * `vec`: The vector to reduce.
    ///
    /// returns: `Vec<f64>`
    pub fn remove_nans(&self) -> Vec<f64> {
        self.0.iter().copied().filter(|x| !x.is_nan()).collect()
    }

    /// Calculate the log of a vector and convert zeros to NaNs.
    ///
    /// # Arguments
    ///
    /// * `vec`: The vector.
    /// * `vec2`: The second vector.
    ///
    /// returns: `Vec<f64>`
    pub fn log(&self) -> Vec<f64> {
        let mut log_numbers: Vec<f64> = vec![];
        for n in self.0.iter() {
            log_numbers.push(if *n <= 0.0 { f64::NAN } else { n.log10() });
        }
        log_numbers
    }

    /// Get the ranks of a vector values.  If n observations have the same rank, then each observation
    /// gets a fractional rank.
    ///
    /// # Arguments
    ///
    /// * `x`: The vector with the data to rank.
    ///
    /// returns: `Vec<f64>`
    pub fn rank(&self) -> Vec<f64> {
        let n = self.0.len();
        let mut rank: Vec<f64> = vec![];
        for i in 0..n {
            let mut r = 1.0;
            let mut s = 1.0;

            for j in 0..i {
                if self.0[j] < self.0[i] {
                    r += 1.0
                }
                if self.0[j] == self.0[i] {
                    s += 1.0
                }
            }

            for j in i + 1..=(n - 1) {
                if self.0[j] < self.0[i] {
                    r += 1.0
                }
                if self.0[j] == self.0[i] {
                    s += 1.0
                }
            }

            rank.push(r + (s - 1.0) * 0.5);
        }
        rank
    }

    /// Calculate the Spearman's rank correlation coefficient with another vector of the same size.
    ///
    /// # Arguments
    ///
    /// * `y`: The second vector.
    ///
    /// returns: f64
    pub fn spearman(&self, y: &[f64]) -> f64 {
        let (x, y) = self.remove_nans_from_pair(y).unwrap();
        let n = x.len() as f64;

        let x_rank = self.rank();
        let y_rank = NaNVec(y.as_slice()).rank();

        let d2_sum: f64 = x_rank.iter().zip(y_rank).map(|(x_r, y_r)| (x_r - y_r).powi(2)).sum();

        1.0 - (6.0 * d2_sum) / (n * (n.powi(2) - 1.0))
    }

    /// Removed NaNs from the stored vector and another one of the same length. If a NaN is present
    /// in one vector only, the number at the same index will also be removed from the second
    /// vector. The resulting vectors will have the same lengths.
    ///
    /// # Arguments
    ///
    /// * `y`: The second vector.
    ///
    /// returns: Result<(`Vec<f64>`, `Vec<f64>`), &str>
    pub fn remove_nans_from_pair<'a>(&self, y: &[f64]) -> Result<(Vec<f64>, Vec<f64>), &'a str> {
        if self.0.len() != y.len() {
            return Err("The vector must have the same length");
        }

        let combined: Vec<(f64, f64)> = self
            .0
            .iter()
            .copied()
            .zip(y.iter().copied())
            .filter(|(x, y)| !x.is_nan() && !y.is_nan())
            .collect();
        Ok(combined.into_iter().unzip())
    }
}

/// Compare two arrays of f64
#[allow(dead_code)]
pub(crate) fn assert_approx_array_eq(calculated_values: &[f64], expected_values: &Vec<f64>) {
    let margins = F64Margin {
        epsilon: 2.0,
        ulps: (f64::EPSILON * 2.0) as i64,
    };
    for (i, (calculated, expected)) in calculated_values.iter().zip(expected_values).enumerate() {
        if !approx_eq!(f64, *calculated, *expected, margins) {
            panic!(
                r#"assertion failed on item #{i:?}
                    actual: `{calculated:?}`,
                    expected: `{expected:?}`"#,
            )
        }
    }
}

pub mod example {
    use chrono::NaiveDate;
    use std::error::Error;
    use std::fs::File;
    use std::path::PathBuf;

    pub struct HydrologicalData {
        /// Vector of time.
        pub time: Vec<NaiveDate>,
        /// Input vector of total precipitation (mm/day).
        pub precipitation: Vec<f64>,
        /// Input vector of potential evapotranspiration (PE) (mm/day).
        pub evapotranspiration: Vec<f64>,
        /// Observed run-off (mm/day).
        pub observed_runoff: Vec<f64>,
    }

    pub fn load_data() -> Result<HydrologicalData, Box<dyn Error>> {
        let mut data_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_folder.push(r"src\test_data\airGR_L0123001_dataset.csv");

        // Collect hydrological data
        let file = File::open(r"gr6j-core\src\test_data\airGR_L0123001_dataset.csv")?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut time: Vec<NaiveDate> = vec![];
        let mut precipitation: Vec<f64> = vec![];
        let mut evapotranspiration: Vec<f64> = vec![];
        let mut observed_runoff: Vec<f64> = vec![];
        for result in rdr.records() {
            let record = result.unwrap();
            let t = NaiveDate::parse_from_str(record.get(0).unwrap(), "%d/%m/%Y")?;
            time.push(t);
            precipitation.push(record.get(1).unwrap().parse::<f64>()?);
            evapotranspiration.push(record.get(2).unwrap().parse::<f64>()?);
            let obs = record.get(3).unwrap();
            let obs = if obs == "NA" { "0.0" } else { obs };
            observed_runoff.push(obs.parse::<f64>()?);
        }

        Ok(HydrologicalData {
            time,
            precipitation,
            evapotranspiration,
            observed_runoff,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::utils::{assert_approx_array_eq, NaNVec};
    use float_cmp::{assert_approx_eq, F64Margin};

    const X: [f64; 10] = [106.0, 100.0, 86.0, 101.0, 99.0, 103.0, 97.0, 113.0, 112.0, 110.0];
    const Y: [f64; 10] = [7.0, 27.0, 2.0, 50.0, 28.0, 29.0, 20.0, 12.0, 6.0, 17.0];
    const MARGINS: F64Margin = F64Margin { epsilon: 0.0, ulps: 2 };

    #[test]
    fn test_rank_1() {
        let expected: Vec<f64> = vec![7.0, 4.0, 1.0, 5.0, 3.0, 6.0, 2.0, 10.0, 9.0, 8.0];
        assert_approx_array_eq(NaNVec(&X).rank().as_ref(), &expected);
    }

    #[test]
    fn test_rank_2() {
        let expected: Vec<f64> = vec![3.0, 7.0, 1.0, 10.0, 8.0, 9.0, 6.0, 4.0, 2.0, 5.0];
        assert_approx_array_eq(NaNVec(&Y).rank().as_ref(), &expected);
    }

    #[test]
    fn test_spearman_corr_1() {
        assert_approx_eq!(f64, NaNVec(&X).spearman(&Y), -0.17575757575757578, MARGINS);
    }

    #[test]
    fn test_spearman_corr_2() {
        let a = vec![1250.0, 0.3, 500.0, 5.2, 2.0, 10.0];
        let b = vec![150.0, 0.03, 200.0, 5.2, 20.0, 15.0];
        assert_approx_eq!(f64, NaNVec(&a).spearman(&b), 0.7714285714285715, MARGINS);
    }
}
