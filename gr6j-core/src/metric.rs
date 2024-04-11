use crate::utils::NaNVec;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// The method to use to calculate the Kling-Gupta coefficient
pub enum KlingGuptaMethod {
    Y2009,
    Y2012,
}

pub enum CalibrationMetric {
    /// The Nash-Sutcliffe efficiency. An efficiency of 1 gives a perfect match of simulated to
    /// observed data. An efficiency of 0 indicates that the model predictions are as accurate as
    /// the mean of the observations, whereas an efficiency less than zero occurs when the
    /// observed mean is a better predictor than the model.   
    NashSutcliffe,
    /// The Nash-Sutcliffe efficiency but the logarithm is applied to flow data to give more
    /// importance to low flow periods. An efficiency of 1 gives a perfect match of simulated to
    /// observed data.
    LogNashSutcliffe,
    KlingGupta2009,
    KlingGupta2012,
}

impl Debug for CalibrationMetric {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CalibrationMetric::NashSutcliffe => write!(f, "Nash-Sutcliffe"),
            CalibrationMetric::LogNashSutcliffe => write!(f, "Nash-Sutcliffe for log flows"),
            CalibrationMetric::KlingGupta2009 => write!(f, "Ling-Gupta (2009)"),
            CalibrationMetric::KlingGupta2012 => write!(f, "Ling-Gupta (2012)"),
        }
    }
}

impl CalibrationMetric {
    /// Get the ideal value for a metric.
    pub fn ideal_value(&self) -> f64 {
        match self {
            CalibrationMetric::NashSutcliffe => 1.0,
            CalibrationMetric::LogNashSutcliffe => 1.0,
            CalibrationMetric::KlingGupta2009 => 1.0,
            CalibrationMetric::KlingGupta2012 => 1.0,
        }
    }

    /// Calculate the metric value.
    ///
    /// # Arguments
    ///
    /// * `observed`: The time-series of observed values.  
    /// * `simulated`: The time-series of simulated values.
    ///
    /// returns: f64
    pub fn value(&self, observed: &[f64], simulated: &[f64]) -> Result<f64, &str> {
        if observed.len() != simulated.len() {
            return Err("The vector must have the same length");
        }

        let v = match self {
            CalibrationMetric::NashSutcliffe => Self::nse(observed, simulated),
            CalibrationMetric::LogNashSutcliffe => {
                Self::nse(NaNVec(observed).log().as_slice(), NaNVec(simulated).log().as_slice())
            }
            CalibrationMetric::KlingGupta2009 => Self::kge(observed, simulated, KlingGuptaMethod::Y2009),
            CalibrationMetric::KlingGupta2012 => Self::kge(observed, simulated, KlingGuptaMethod::Y2012),
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
        };

        1.0 - ((r - 1.0).powi(2) + (alpha - 1.0).powi(2) + (beta - 1.0).powi(2)).powf(0.5)
    }
}

#[cfg(test)]
mod tests {
    use crate::metric::CalibrationMetric;
    use float_cmp::{assert_approx_eq, F64Margin};

    const A: [f64; 6] = [1250.0, 0.3, 500.0, 5.2, 2.0, 10.0];
    const B: [f64; 6] = [150.0, 0.03, 200.0, 5.2, 20.0, 15.0];
    const A_NAN: [f64; 6] = [f64::NAN, 0.3, 500.0, 5.2, 2.0, 10.0];
    const B_NAN: [f64; 6] = [150.0, 0.03, 200.0, 5.2, 20.0, f64::NAN];

    const MARGINS: F64Margin = F64Margin { epsilon: 0.0, ulps: 2 };

    #[test]
    fn test_ideal_values() {
        assert_eq!(
            CalibrationMetric::NashSutcliffe.value(&A, &A),
            Ok(CalibrationMetric::NashSutcliffe.ideal_value())
        );
        assert_eq!(
            CalibrationMetric::LogNashSutcliffe.value(&A, &A),
            Ok(CalibrationMetric::LogNashSutcliffe.ideal_value())
        );
    }

    #[test]
    fn test_nse_metric() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::NashSutcliffe.value(&A, &B).unwrap(),
            -0.006497117928065954,
            MARGINS
        );
    }

    #[test]
    fn test_nse_metric_with_nan_1() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::NashSutcliffe.value(&A_NAN, &B).unwrap(),
            0.540371734977912,
            MARGINS
        );
    }

    #[test]
    fn test_nse_metric_with_nan_2() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::NashSutcliffe.value(&A_NAN, &B_NAN).unwrap(),
            0.5404989162123923,
            MARGINS
        );
    }

    #[test]
    fn test_log_nse_metric() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::LogNashSutcliffe.value(&A, &B).unwrap(),
            0.6930355551239313,
            MARGINS
        );
    }

    #[test]
    fn test_log_nse_metric_with_nan_1() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::LogNashSutcliffe.value(&A_NAN, &B).unwrap(),
            0.612135455999324,
            MARGINS
        );
    }

    #[test]
    fn test_log_nse_metric_with_nan_2() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::LogNashSutcliffe.value(&A_NAN, &B_NAN).unwrap(),
            0.6176288105498396,
            MARGINS
        );
    }

    #[test]
    fn test_log_kg_2009_metric() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::KlingGupta2009.value(&A, &B).unwrap(),
            -0.16047005836641337,
            MARGINS
        );
    }

    #[test]
    fn test_log_kg_2009_metric_with_nan_1() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::KlingGupta2009.value(&A_NAN, &B).unwrap(),
            0.13079945561027917,
            MARGINS
        );
    }

    #[test]
    fn test_log_kg_2009_metric_with_nan_2() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::KlingGupta2009.value(&A_NAN, &B_NAN).unwrap(),
            0.1481643733978315,
            MARGINS
        );
    }

    #[test]
    fn test_log_kg_2012_metric() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::KlingGupta2012.value(&A, &B).unwrap(),
            0.15721037908744573,
            MARGINS
        );
    }

    #[test]
    fn test_log_kg_2012_metric_with_nan_1() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::KlingGupta2012.value(&A_NAN, &B).unwrap(),
            0.3625714406316686,
            MARGINS
        );
    }

    #[test]
    fn test_log_kg_2012_metric_with_nan_2() {
        assert_approx_eq!(
            f64,
            CalibrationMetric::KlingGupta2012.value(&A_NAN, &B_NAN).unwrap(),
            0.3950431057298418,
            MARGINS
        );
    }
}
