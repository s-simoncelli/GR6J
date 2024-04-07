use crate::utils::NaNVec;
use std::fmt;
use std::fmt::{Debug, Formatter};

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
            CalibrationMetric::NashSutcliffe => Self::nse(observed, &simulated),
            CalibrationMetric::LogNashSutcliffe => {
                Self::nse(NaNVec(observed).log().as_slice(), NaNVec(simulated).log().as_slice())
            }
            CalibrationMetric::KlingGupta2009 => !todo!(),
            CalibrationMetric::KlingGupta2012 => !todo!(),
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
}
