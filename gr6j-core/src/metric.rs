use crate::utils::{Fdc, NaNVec};
use csv::Writer;
use std::fs::File;
use std::path::PathBuf;

/// The method to use to calculate the Kling-Gupta coefficient
pub enum KlingGuptaMethod {
    Y2009,
    Y2012,
    NonParametric,
}

//. A metric data
#[derive(Debug, Clone)]
pub struct Metric {
    /// The metric name
    pub name: String,
    /// The metric ideal value.
    pub ideal_value: f64,
    /// The metric value from the difference between the observed and simulated data.
    pub value: f64,
}

/// The type of calibration metric to calculate.
#[derive(Debug, Clone)]
pub struct CalibrationMetric {
    /// The Nash-Sutcliffe efficiency. An efficiency of 1 gives a perfect match of simulated to
    /// observed data. An efficiency of 0 indicates that the model predictions are as accurate as
    /// the mean of the observations, whereas an efficiency less than zero occurs when the
    /// observed mean is a better predictor than the model.   
    pub nash_sutcliffe: Metric,
    /// The Nash-Sutcliffe efficiency but the logarithm is applied to flow data to give more
    /// importance to low flow periods. An efficiency of 1 gives a perfect match of simulated to
    /// observed data.
    pub log_nash_sutcliffe: Metric,
    /// The 2009 Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
    /// of simulated to observed data. To calculate the alpha component the standard deviation is
    /// used.
    pub kling_gupta2009: Metric,
    /// The 2012 Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
    /// of simulated to observed data. To calculate the alpha component the ratio of the standard
    /// deviation and the mean is used.
    pub kling_gupta2012: Metric,
    /// The non-parametric Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
    /// of simulated to observed data. This differs from [`CalibrationMetric::kling_gupta2009`] and
    /// [`CalibrationMetric::kling_gupta2012`] because the alpha component is calculated using the
    /// flow percentile from the flow duration curve instead of using the standard deviation.
    /// See <https://www.tandfonline.com/doi/full/10.1080/02626667.2018.1552002>
    pub non_parametric_kling_gupta: Metric,
    /// The root-mean-square error. A small value indicates an overall small error and better
    /// simulated run off.
    pub rmse: Metric,
    /// The simulation volume error. A value of 0.0 suggests a good simulated flow. A negative value
    /// indicates that the simulation generates less volume than the observed. A positive value
    /// indicates instead that the simulation generates more volume than the observed.
    pub volume_error: Metric,
    /// A structure controlling whether to calculate additional metrics.
    optional_metrics: OptionalMetrics,
}

///Struct controlling the optional metrics to calculate
#[derive(Default, Debug, Clone)]
pub struct OptionalMetrics {
    /// Set this to `true` to calculate the 2009 Kling-Gupta efficiency.
    kling_gupta2009: bool,
    /// Set this to `true` to calculate the 2012 Kling-Gupta efficiency.
    kling_gupta2012: bool,
}

impl<'a> CalibrationMetric {
    /// Calculate the efficiency metrics between two flow time series. SOme metrics
    ///
    /// # Arguments
    ///
    /// * `observed`: The observed flow series.
    /// * `simulated`: The simulated flow series.
    /// * `optional_metrics`: The optional metric to calculate. If None this defaults to
    /// [`OptionalMetrics::default()`]. Some metrics are similar and are not calculated by default
    /// and must be enabled by the user.
    ///
    /// returns: `Result<CalibrationMetric, String>`
    pub fn new(
        observed: &'a [f64],
        simulated: &'a [f64],
        optional_metrics: Option<OptionalMetrics>,
    ) -> Result<Self, String> {
        if observed.len() != simulated.len() {
            return Err(format!(
                "The vector must have the same length. Observed has {} values and simulated has {} values",
                observed.len(),
                simulated.len()
            ));
        }
        let optional_metrics = optional_metrics.unwrap_or_default();
        let mut metrics = Self {
            nash_sutcliffe: Metric {
                name: "Nash-Sutcliffe".to_string(),
                ideal_value: 1.0,
                value: Self::nse(observed, simulated),
            },
            log_nash_sutcliffe: Metric {
                name: "Nash-Sutcliffe with log flows".to_string(),
                ideal_value: 1.0,
                value: Self::nse(NaNVec(observed).log().as_slice(), NaNVec(simulated).log().as_slice()),
            },
            kling_gupta2009: Metric {
                name: "Kling-Gupta (2009)".to_string(),
                ideal_value: 1.0,
                value: f64::NAN,
            },
            kling_gupta2012: Metric {
                name: "Kling-Gupta (2012)".to_string(),
                ideal_value: 1.0,
                value: f64::NAN,
            },
            non_parametric_kling_gupta: Metric {
                name: "Non-parametric Kling-Gupta".to_string(),
                ideal_value: 1.0,
                value: Self::kge(observed, simulated, KlingGuptaMethod::NonParametric),
            },
            rmse: Metric {
                name: "Root-mean-square error".to_string(),
                ideal_value: 0.0,
                value: Self::rmse(observed, simulated),
            },
            volume_error: Metric {
                name: "Volume error".to_string(),
                ideal_value: 0.0,
                value: Self::volume_error(observed, simulated),
            },
            optional_metrics: optional_metrics.clone(),
        };

        if optional_metrics.kling_gupta2009 {
            metrics.kling_gupta2009.value = Self::kge(observed, simulated, KlingGuptaMethod::Y2009);
        }
        if optional_metrics.kling_gupta2012 {
            metrics.kling_gupta2012.value = Self::kge(observed, simulated, KlingGuptaMethod::Y2012);
        }
        Ok(metrics)
    }

    /// Append the metric values to a CSV file as row.
    ///
    /// # Arguments
    ///
    /// * `wtr`: The file writer.
    /// * `index`: A string to write as index to identify the row number.
    ///
    /// returns: Result<(), csv::Error>
    pub fn append_row_to_csv(&self, wtr: &mut Writer<File>, index: Option<String>) -> Result<(), csv::Error> {
        let mut row = vec![];
        if let Some(i) = index {
            row.push(i);
        }
        row.push(self.nash_sutcliffe.value.to_string());
        row.push(self.log_nash_sutcliffe.value.to_string());
        if self.optional_metrics.kling_gupta2009 {
            row.push(self.kling_gupta2009.value.to_string());
        }
        if self.optional_metrics.kling_gupta2012 {
            row.push(self.kling_gupta2012.value.to_string());
        }
        row.push(self.non_parametric_kling_gupta.value.to_string());
        wtr.write_record(row)?;
        wtr.flush()?;

        Ok(())
    }

    /// Append the header row with the metric names to a CSV file.
    ///
    /// # Arguments
    ///
    /// * `wtr`: The file writer.
    /// * `index`: A string to write to identify the index name in the header.
    ///
    /// returns: Result<(), csv::Error>
    pub fn append_header_to_csv(&self, wtr: &mut Writer<File>, index: Option<String>) -> Result<(), csv::Error> {
        let mut row = vec![];
        if let Some(i) = index {
            row.push(i);
        }
        row.push(self.nash_sutcliffe.name.to_string());
        row.push(self.log_nash_sutcliffe.name.to_string());
        if self.optional_metrics.kling_gupta2009 {
            row.push(self.kling_gupta2009.name.to_string());
        }
        if self.optional_metrics.kling_gupta2012 {
            row.push(self.kling_gupta2012.name.to_string());
        }
        row.push(self.non_parametric_kling_gupta.name.to_string());
        wtr.write_record(row)?;
        wtr.flush()?;

        Ok(())
    }

    /// Export the calibration metric values and their information to a CSV file.
    ///
    /// # Arguments
    ///
    /// * `destination`: The destination CSV file.
    ///
    /// returns: Result<(), csv::Error>
    pub fn to_csv(&self, destination: PathBuf) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_path(destination)?;
        wtr.write_record(["Metric", "Value", "Ideal value"])?;
        wtr.write_record([
            self.nash_sutcliffe.name.to_string(),
            self.nash_sutcliffe.value.to_string(),
            self.nash_sutcliffe.ideal_value.to_string(),
        ])?;
        wtr.write_record([
            self.log_nash_sutcliffe.name.to_string(),
            self.log_nash_sutcliffe.value.to_string(),
            self.log_nash_sutcliffe.ideal_value.to_string(),
        ])?;
        if self.optional_metrics.kling_gupta2009 {
            wtr.write_record([
                self.kling_gupta2009.name.to_string(),
                self.kling_gupta2009.value.to_string(),
                self.kling_gupta2009.ideal_value.to_string(),
            ])?;
        }
        if self.optional_metrics.kling_gupta2012 {
            wtr.write_record([
                self.kling_gupta2012.name.to_string(),
                self.kling_gupta2012.value.to_string(),
                self.kling_gupta2012.ideal_value.to_string(),
            ])?;
        }
        wtr.write_record([
            self.non_parametric_kling_gupta.name.to_string(),
            self.non_parametric_kling_gupta.value.to_string(),
            self.non_parametric_kling_gupta.ideal_value.to_string(),
        ])?;
        wtr.flush()?;

        Ok(())
    }

    /// Calculate the Nash-Sutcliffe efficiency. A perfect model simulation returns 1.0.
    ///
    /// # Arguments
    ///
    /// * `observed`: The vector of observed data.
    /// * `simulated`: The vector of simulated values.
    ///
    /// returns: f64
    fn nse(observed: &[f64], simulated: &[f64]) -> f64 {
        let obs_mean = NaNVec(observed).mean();
        let mut n: f64 = 0.0;
        let mut d: f64 = 0.0;
        for (obs, sim) in observed.iter().zip(simulated) {
            if !obs.is_nan() {
                if !sim.is_nan() {
                    n += (obs - sim).powi(2);
                }
                d += (obs - obs_mean).powi(2);
            }
        }

        1.0 - n / d
    }

    /// Calculate the Kling-Gupta coefficient. A perfect model simulation returns 1.0.
    ///
    /// # Arguments
    ///
    /// * `observed`: The vector of observed data.
    /// * `simulated`: The vector of simulated values.
    /// * `method`: The method to use.
    ///
    /// returns: f64
    fn kge(observed: &[f64], simulated: &[f64], method: KlingGuptaMethod) -> f64 {
        // remove NaNs from both vectors
        let (observed, simulated) = NaNVec(observed).remove_nans_from_pair(simulated).unwrap();

        let obs = NaNVec(observed.as_slice());
        let sim = NaNVec(simulated.as_slice());
        let r = obs.spearman(simulated.as_slice());

        let obs_mean = obs.mean();
        let sim_mean = sim.mean();
        let beta = sim_mean / obs_mean;

        let alpha = match method {
            KlingGuptaMethod::Y2009 => sim.std() / obs.std(),
            KlingGuptaMethod::Y2012 => (sim.std() / sim_mean) / (obs.std() / obs_mean),
            KlingGuptaMethod::NonParametric => {
                let scaled_obs: Vec<f64> = observed
                    .iter()
                    .map(|x| x / (obs_mean * observed.len() as f64))
                    .collect();
                let obs_fdc = Fdc::new(scaled_obs.as_slice());

                let scaled_sim: Vec<f64> = simulated
                    .iter()
                    .map(|x| x / (sim_mean * simulated.len() as f64))
                    .collect();
                let sim_fdc = Fdc::new(scaled_sim.as_slice());

                let deltas: f64 = sim_fdc
                    .sorted_run_off
                    .iter()
                    .zip(obs_fdc.sorted_run_off)
                    .map(|(x1, x2)| (x1 - x2).abs())
                    .sum();
                1.0 - 0.5 * deltas
            }
        };

        1.0 - ((r - 1.0).powi(2) + (alpha - 1.0).powi(2) + (beta - 1.0).powi(2)).powf(0.5)
    }

    /// Calculate the root-mean-square deviation. A perfect model simulation returns 0.0.
    ///
    /// # Arguments
    ///
    /// * `observed`: The vector of observed data.
    /// * `simulated`: The vector of simulated values.
    ///
    /// returns: f64
    pub fn rmse(observed: &[f64], simulated: &[f64]) -> f64 {
        let mut error: f64 = 0.0;
        let mut d: usize = 0;
        for (obs, sim) in observed.iter().zip(simulated) {
            if !obs.is_nan() && !sim.is_nan() {
                error += (obs - sim).powi(2);
                d += 1;
            }
        }

        if d == 0 {
            f64::NAN
        } else {
            (error / d as f64).powf(0.5)
        }
    }

    /// Calculate the volume error as difference between the sum of the total simulated and
    /// observed flow. A good model simulation returns 0.0.
    ///
    /// # Arguments
    ///
    /// * `observed`: The vector of observed data.
    /// * `simulated`: The vector of simulated values.
    ///
    /// returns: f64
    pub fn volume_error(observed: &[f64], simulated: &[f64]) -> f64 {
        let mut obs_volume: f64 = 0.0;
        let mut sim_volume: f64 = 0.0;
        for (obs, sim) in observed.iter().zip(simulated) {
            if !obs.is_nan() && !sim.is_nan() {
                obs_volume += obs;
                sim_volume += sim;
            }
        }

        (sim_volume / obs_volume - 1.0) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::{CalibrationMetric, OptionalMetrics};
    use float_cmp::{assert_approx_eq, F64Margin};

    const A: [f64; 6] = [1250.0, 0.3, 500.0, 5.2, 2.0, 10.0];
    const B: [f64; 6] = [150.0, 0.03, 200.0, 5.2, 20.0, 15.0];
    const A_NAN: [f64; 6] = [f64::NAN, 0.3, 500.0, 5.2, 2.0, 10.0];
    const B_NAN: [f64; 6] = [150.0, 0.03, 200.0, 5.2, 20.0, f64::NAN];

    const MARGINS: F64Margin = F64Margin { epsilon: 0.0, ulps: 2 };
    const OPTIONAL_METRICS: Option<OptionalMetrics> = Some(OptionalMetrics {
        kling_gupta2009: true,
        kling_gupta2012: true,
    });

    #[test]
    fn test_ideal_values() {
        let metric = CalibrationMetric::new(&B, &B, OPTIONAL_METRICS).unwrap();
        assert_eq!(metric.nash_sutcliffe.ideal_value, 1.0);
        assert_eq!(metric.log_nash_sutcliffe.ideal_value, 1.0);
        assert_eq!(metric.kling_gupta2009.ideal_value, 1.0);
        assert_eq!(metric.kling_gupta2012.ideal_value, 1.0);
        assert_eq!(metric.non_parametric_kling_gupta.ideal_value, 1.0);
    }

    #[test]
    fn test_nse_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.nash_sutcliffe.value, -0.006497117928065954, MARGINS);
    }

    #[test]
    fn test_nse_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.nash_sutcliffe.value, 0.540371734977912, MARGINS);
    }

    #[test]
    fn test_nse_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.nash_sutcliffe.value, 0.5404989162123923, MARGINS);
    }

    #[test]
    fn test_log_nse_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.log_nash_sutcliffe.value, 0.6930355551239313, MARGINS);
    }

    #[test]
    fn test_log_nse_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.log_nash_sutcliffe.value, 0.612135455999324, MARGINS);
    }

    #[test]
    fn test_log_nse_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.log_nash_sutcliffe.value, 0.6176288105498396, MARGINS);
    }

    #[test]
    fn test_kg_2009_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.kling_gupta2009.value, -0.16047005836641337, MARGINS);
    }

    #[test]
    fn test_kg_2009_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.kling_gupta2009.value, 0.13079945561027917, MARGINS);
    }

    #[test]
    fn test_kg_2009_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.kling_gupta2009.value, 0.1481643733978315, MARGINS);
    }

    #[test]
    fn test_kg_2012_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.kling_gupta2012.value, 0.15721037908744573, MARGINS);
    }

    #[test]
    fn test_kg_2012_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.kling_gupta2012.value, 0.3625714406316686, MARGINS);
    }

    #[test]
    fn test_kg_2012_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.kling_gupta2012.value, 0.3950431057298418, MARGINS);
    }

    #[test]
    fn test_np_kg_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(
            f64,
            metric.non_parametric_kling_gupta.value,
            0.16491320894422312,
            MARGINS
        );
    }

    #[test]
    fn test_np_kg_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(
            f64,
            metric.non_parametric_kling_gupta.value,
            0.3714685584392162,
            MARGINS
        );
    }

    #[test]
    fn test_np_kg_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(
            f64,
            metric.non_parametric_kling_gupta.value,
            0.40091725404120415,
            MARGINS
        );
    }

    #[test]
    fn test_rmse_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.rmse.value, 465.537158004958, MARGINS);
    }

    #[test]
    fn test_rmse_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.rmse.value, 134.42401042968476, MARGINS);
    }

    #[test]
    fn test_rmse_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.rmse.value, 150.26981807735044, MARGINS);
    }

    #[test]
    fn test_volume_error_metric() {
        let metric = CalibrationMetric::new(&A, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.volume_error.value, -77.9219236209335, MARGINS);
    }

    #[test]
    fn test_volume_error_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.volume_error.value, -53.57874396135266, MARGINS);
    }

    #[test]
    fn test_volume_error_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN, OPTIONAL_METRICS).unwrap();
        assert_approx_eq!(f64, metric.volume_error.value, -55.61970443349754, MARGINS);
    }
}
