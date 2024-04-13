use crate::utils::{calculate_fdc, NaNVec};

/// The method to use to calculate the Kling-Gupta coefficient
pub enum KlingGuptaMethod {
    Y2009,
    Y2012,
    NonParametric,
}

/// The type of calibration metric to calculate.
#[derive(Clone, Copy)]
pub enum CalibrationMetricType {
    /// The Nash-Sutcliffe efficiency. An efficiency of 1 gives a perfect match of simulated to
    /// observed data. An efficiency of 0 indicates that the model predictions are as accurate as
    /// the mean of the observations, whereas an efficiency less than zero occurs when the
    /// observed mean is a better predictor than the model.   
    NashSutcliffe,
    /// The Nash-Sutcliffe efficiency but the logarithm is applied to flow data to give more
    /// importance to low flow periods. An efficiency of 1 gives a perfect match of simulated to
    /// observed data.
    LogNashSutcliffe,
    /// The 2009 Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
    /// of simulated to observed data. To calculate the alpha component the standard deviation is
    /// used.
    KlingGupta2009,
    /// The 2012 Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
    /// of simulated to observed data. To calculate the alpha component the ratio of the standard
    /// deviation and the mean is used.
    KlingGupta2012,
    /// The non-parametric Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
    /// of simulated to observed data. This differs from [`Self::KlingGupta2009`] and
    /// [`Self::KlingGupta2012`] because the alpha component is calculated using the flow percentile
    /// from the flow duration curve instead of using the standard deviation.
    /// See <https://www.tandfonline.com/doi/full/10.1080/02626667.2018.1552002>
    NonParamettricKlingGupta,
}

/// Calculate a calibration metric by comparing the observed and simulated run-off series.
pub struct CalibrationMetric<'a> {
    observed: &'a [f64],
    simulated: &'a [f64],
}

impl<'a> CalibrationMetric<'a> {
    pub fn new(observed: &'a [f64], simulated: &'a [f64]) -> Result<Self, String> {
        if observed.len() != simulated.len() {
            return Err(format!(
                "The vector must have the same length. Observed has {} values and simulated has {} values",
                observed.len(),
                simulated.len()
            ));
        }

        Ok(Self { observed, simulated })
    }

    /// Get the metric full name.
    ///
    /// # Arguments
    ///
    /// * `metric_type`: The type of metric.
    ///
    /// returns: &str
    pub fn full_name(metric_type: CalibrationMetricType) -> &'a str {
        match metric_type {
            CalibrationMetricType::NashSutcliffe => "Nash-Sutcliffe",
            CalibrationMetricType::LogNashSutcliffe => "Nash-Sutcliffe with log flows",
            CalibrationMetricType::KlingGupta2009 => "Kling-Gupta (2009)",
            CalibrationMetricType::KlingGupta2012 => "Kling-Gupta (2012)",
            CalibrationMetricType::NonParamettricKlingGupta => "Non-parametric Kling-Gupta",
        }
    }

    /// Get the ideal value for a metric.
    ///
    /// # Arguments
    ///
    /// * `metric_type`: The type of metric.
    ///
    /// returns: &str
    pub fn ideal_value(metric_type: CalibrationMetricType) -> f64 {
        match metric_type {
            CalibrationMetricType::NashSutcliffe => 1.0,
            CalibrationMetricType::LogNashSutcliffe => 1.0,
            CalibrationMetricType::KlingGupta2009 => 1.0,
            CalibrationMetricType::KlingGupta2012 => 1.0,
            CalibrationMetricType::NonParamettricKlingGupta => 1.0,
        }
    }

    /// Calculate the metric value.
    ///
    /// # Arguments
    ///
    /// * `metric_type`: The type of metric to calculate.  
    ///
    /// returns: f64
    pub fn value(&self, metric_type: CalibrationMetricType) -> Result<f64, &str> {
        let v = match metric_type {
            CalibrationMetricType::NashSutcliffe => Self::nse(self.observed, self.simulated),
            CalibrationMetricType::LogNashSutcliffe => Self::nse(
                NaNVec(self.observed).log().as_slice(),
                NaNVec(self.simulated).log().as_slice(),
            ),
            CalibrationMetricType::KlingGupta2009 => Self::kge(self.observed, self.simulated, KlingGuptaMethod::Y2009),
            CalibrationMetricType::KlingGupta2012 => Self::kge(self.observed, self.simulated, KlingGuptaMethod::Y2012),
            CalibrationMetricType::NonParamettricKlingGupta => {
                Self::kge(self.observed, self.simulated, KlingGuptaMethod::NonParametric)
            }
        };
        Ok(v)
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
                let (_, obs_fdc) = calculate_fdc(scaled_obs.as_slice());

                let scaled_sim: Vec<f64> = simulated
                    .iter()
                    .map(|x| x / (sim_mean * simulated.len() as f64))
                    .collect();
                let (_, sim_fdc) = calculate_fdc(scaled_sim.as_slice());

                let deltas: f64 = sim_fdc.iter().zip(obs_fdc).map(|(x1, x2)| (x1 - x2).abs()).sum();
                1.0 - 0.5 * deltas
            }
        };

        1.0 - ((r - 1.0).powi(2) + (alpha - 1.0).powi(2) + (beta - 1.0).powi(2)).powf(0.5)
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::{CalibrationMetric, CalibrationMetricType};
    use float_cmp::{assert_approx_eq, F64Margin};

    const A: [f64; 6] = [1250.0, 0.3, 500.0, 5.2, 2.0, 10.0];
    const B: [f64; 6] = [150.0, 0.03, 200.0, 5.2, 20.0, 15.0];
    const A_NAN: [f64; 6] = [f64::NAN, 0.3, 500.0, 5.2, 2.0, 10.0];
    const B_NAN: [f64; 6] = [150.0, 0.03, 200.0, 5.2, 20.0, f64::NAN];

    const MARGINS: F64Margin = F64Margin { epsilon: 0.0, ulps: 2 };

    #[test]
    fn test_ideal_values() {
        let metric = CalibrationMetric::new(&B, &B).unwrap();
        assert_eq!(
            metric.value(CalibrationMetricType::NashSutcliffe),
            Ok(CalibrationMetric::ideal_value(CalibrationMetricType::NashSutcliffe))
        );
        assert_eq!(
            metric.value(CalibrationMetricType::LogNashSutcliffe),
            Ok(CalibrationMetric::ideal_value(CalibrationMetricType::LogNashSutcliffe))
        );
    }

    #[test]
    fn test_nse_metric() {
        let metric = CalibrationMetric::new(&A, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::NashSutcliffe).unwrap(),
            -0.006497117928065954,
            MARGINS
        );
    }

    #[test]
    fn test_nse_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::NashSutcliffe).unwrap(),
            0.540371734977912,
            MARGINS
        );
    }

    #[test]
    fn test_nse_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::NashSutcliffe).unwrap(),
            0.5404989162123923,
            MARGINS
        );
    }

    #[test]
    fn test_log_nse_metric() {
        let metric = CalibrationMetric::new(&A, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::LogNashSutcliffe).unwrap(),
            0.6930355551239313,
            MARGINS
        );
    }

    #[test]
    fn test_log_nse_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::LogNashSutcliffe).unwrap(),
            0.612135455999324,
            MARGINS
        );
    }

    #[test]
    fn test_log_nse_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::LogNashSutcliffe).unwrap(),
            0.6176288105498396,
            MARGINS
        );
    }

    #[test]
    fn test_kg_2009_metric() {
        let metric = CalibrationMetric::new(&A, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::KlingGupta2009).unwrap(),
            -0.16047005836641337,
            MARGINS
        );
    }

    #[test]
    fn test_kg_2009_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::KlingGupta2009).unwrap(),
            0.13079945561027917,
            MARGINS
        );
    }

    #[test]
    fn test_kg_2009_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::KlingGupta2009).unwrap(),
            0.1481643733978315,
            MARGINS
        );
    }

    #[test]
    fn test_kg_2012_metric() {
        let metric = CalibrationMetric::new(&A, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::KlingGupta2012).unwrap(),
            0.15721037908744573,
            MARGINS
        );
    }

    #[test]
    fn test_kg_2012_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::KlingGupta2012).unwrap(),
            0.3625714406316686,
            MARGINS
        );
    }

    #[test]
    fn test_kg_2012_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::KlingGupta2012).unwrap(),
            0.3950431057298418,
            MARGINS
        );
    }

    #[test]
    fn test_np_kg_metric() {
        let metric = CalibrationMetric::new(&A, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::NonParamettricKlingGupta).unwrap(),
            0.16491320894422312,
            MARGINS
        );
    }

    #[test]
    fn test_np_kg_metric_with_nan_1() {
        let metric = CalibrationMetric::new(&A_NAN, &B).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::NonParamettricKlingGupta).unwrap(),
            0.3714685584392162,
            MARGINS
        );
    }

    #[test]
    fn test_np_kg_metric_with_nan_2() {
        let metric = CalibrationMetric::new(&A_NAN, &B_NAN).unwrap();
        assert_approx_eq!(
            f64,
            metric.value(CalibrationMetricType::NonParamettricKlingGupta).unwrap(),
            0.40091725404120415,
            MARGINS
        );
    }
}
