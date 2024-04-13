use std::fmt;
use std::fmt::Formatter;

pub trait Parameter<'a>: fmt::Display {
    fn new(value: f64) -> Result<Box<Self>, String>;

    /// Check that a value is within the min and max parameter bounds.
    fn check(value: f64) -> Result<(), String> {
        // check min
        if value < Self::min_value() {
            return Err(format!(
                "The {} must be larger than its minimum threshold ({})",
                Self::description(),
                Self::min_value(),
            ));
        }
        // check max
        if value > Self::max_value() {
            return Err(format!(
                "The {} must be smaller than its maximum threshold ({})",
                Self::description(),
                Self::max_value(),
            ));
        }

        Ok(())
    }

    /// Return the parameter value.
    fn value(&self) -> f64;

    /// Return the parameter minimum value.
    fn min_value() -> f64;

    /// Return the parameter maximum value.
    fn max_value() -> f64;

    /// Return the parameter unit of measurement.
    fn unit() -> &'a str;

    /// Return the parameter description.
    fn description() -> &'a str;
}

/// Maximum capacity of the production store (mm/day)
#[derive(Debug, Clone, Copy)]
pub struct X1(f64);

impl<'a> Parameter<'a> for X1 {
    fn new(value: f64) -> Result<Box<Self>, String> {
        X1::check(value)?;
        Ok(Box::new(Self(value)))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn min_value() -> f64 {
        1e-2
    }

    fn max_value() -> f64 {
        2500.0
    }

    fn unit() -> &'a str {
        "mm"
    }

    fn description() -> &'a str {
        "production store capacity (X1)"
    }
}

impl fmt::Display for X1 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} (X1={})", X1::description(), self.0)
    }
}

/// Inter-catchment (or groundwater) exchange coefficient (mm/day). X2 can be positive
/// or negative to simulate imports or exports of water with deep aquifers or
/// surrounding catchments.
#[derive(Debug, Clone, Copy)]
pub struct X2(f64);

impl<'a> Parameter<'a> for X2 {
    fn new(value: f64) -> Result<Box<Self>, String> {
        X2::check(value)?;
        Ok(Box::new(Self(value)))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn min_value() -> f64 {
        -5.0
    }

    fn max_value() -> f64 {
        5.0
    }

    fn unit() -> &'a str {
        "mm"
    }

    fn description() -> &'a str {
        "maximum capacity of the routing store (X2)"
    }
}

impl fmt::Display for X2 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} (X2={})", X2::description(), self.0)
    }
}

/// One-day-ahead maximum capacity of the routing store (mm/day)
#[derive(Debug, Clone, Copy)]
pub struct X3(f64);

impl<'a> Parameter<'a> for X3 {
    fn new(value: f64) -> Result<Box<Self>, String> {
        X3::check(value)?;
        Ok(Box::new(Self(value)))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn min_value() -> f64 {
        1e-2
    }

    fn max_value() -> f64 {
        1000.0
    }

    fn unit() -> &'a str {
        "mm"
    }

    fn description() -> &'a str {
        "routing store capacity (X3)"
    }
}

impl fmt::Display for X3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} (X3={})", X3::description(), self.0)
    }
}

/// Time base of unit hydrograph `UH1` (days)
#[derive(Debug, Clone, Copy)]
pub struct X4(f64);

impl<'a> Parameter<'a> for X4 {
    fn new(value: f64) -> Result<Box<Self>, String> {
        X4::check(value)?;
        Ok(Box::new(Self(value)))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn min_value() -> f64 {
        0.5
    }

    fn max_value() -> f64 {
        10.0
    }

    fn unit() -> &'a str {
        "days"
    }

    fn description() -> &'a str {
        "time base of unit hydrograph (X4)"
    }
}

impl fmt::Display for X4 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} (X4={})", X4::description(), self.0)
    }
}

/// Inter-catchment exchange threshold. This is a dimensionless threshold parameter that
/// allows a change in the direction of the groundwater exchange depending on the capacity
/// of the routing store level `R`.
#[derive(Debug, Clone, Copy)]
pub struct X5(f64);

impl<'a> Parameter<'a> for X5 {
    fn new(value: f64) -> Result<Box<Self>, String> {
        X5::check(value)?;
        Ok(Box::new(Self(value)))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn min_value() -> f64 {
        -4.0
    }

    fn max_value() -> f64 {
        4.0
    }

    fn unit() -> &'a str {
        "-"
    }

    fn description() -> &'a str {
        "inter-catchment exchange threshold (X5)"
    }
}

impl fmt::Display for X5 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} (X5={})", X5::description(), self.0)
    }
}

/// Time constant of exponential store (mm)
#[derive(Debug, Clone, Copy)]
pub struct X6(f64);

impl<'a> Parameter<'a> for X6 {
    fn new(value: f64) -> Result<Box<Self>, String> {
        X6::check(value)?;
        Ok(Box::new(Self(value)))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn min_value() -> f64 {
        1e-2
    }

    fn max_value() -> f64 {
        20.0
    }

    fn unit() -> &'a str {
        "mm"
    }

    fn description() -> &'a str {
        "time constant of exponential store (X6)"
    }
}

impl fmt::Display for X6 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} (X6={})", X6::description(), self.0)
    }
}

/// For each model parameter, define the range of values to use during the calibration sub-sampling.
pub trait ParameterRange {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String>;

    /// Check that a value is within the min and max parameter bounds.
    fn check(lower_bound: f64, upper_bound: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
        // check min
        if lower_bound < min {
            return Err(format!(
                "The lower bound ({}) for '{}' must be larger than the parameter minimum threshold ({})",
                lower_bound, name, min,
            ));
        }
        // check max
        if upper_bound > max {
            return Err(format!(
                "The upper bound ({}) for '{}' must be smaller than the parameter maximum threshold ({})",
                upper_bound, name, max,
            ));
        }

        Ok(())
    }
}

/// Range for the maximum capacity of the production store (mm/day).
#[derive(Debug, Clone, Copy)]
pub struct X1Range {
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ParameterRange for X1Range {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String> {
        Self::check(
            lower_bound,
            upper_bound,
            X1::min_value(),
            X1::max_value(),
            X1::description(),
        )?;
        Ok(Box::new(Self {
            lower_bound,
            upper_bound,
        }))
    }
}

/// Range for one-day-ahead maximum capacity of the routing store (mm/day).
#[derive(Debug, Clone, Copy)]
pub struct X2Range {
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ParameterRange for X2Range {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String> {
        Self::check(
            lower_bound,
            upper_bound,
            X2::min_value(),
            X2::max_value(),
            X2::description(),
        )?;
        Ok(Box::new(Self {
            lower_bound,
            upper_bound,
        }))
    }
}

/// Range for time base of the unit hydrograph (days).
#[derive(Debug, Clone, Copy)]
pub struct X3Range {
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ParameterRange for X3Range {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String> {
        Self::check(
            lower_bound,
            upper_bound,
            X3::min_value(),
            X3::max_value(),
            X3::description(),
        )?;
        Ok(Box::new(Self {
            lower_bound,
            upper_bound,
        }))
    }
}

/// Range for the time base of unit hydrograph (days)
#[derive(Debug, Clone, Copy)]
pub struct X4Range {
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ParameterRange for X4Range {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String> {
        Self::check(
            lower_bound,
            upper_bound,
            X4::min_value(),
            X4::max_value(),
            X4::description(),
        )?;
        Ok(Box::new(Self {
            lower_bound,
            upper_bound,
        }))
    }
}

/// Range for the inter-catchment exchange threshold.
#[derive(Debug, Clone, Copy)]
pub struct X5Range {
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ParameterRange for X5Range {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String> {
        Self::check(
            lower_bound,
            upper_bound,
            X5::min_value(),
            X5::max_value(),
            X5::description(),
        )?;
        Ok(Box::new(Self {
            lower_bound,
            upper_bound,
        }))
    }
}

/// Range for the time constant of exponential store (mm).
#[derive(Debug, Clone, Copy)]
pub struct X6Range {
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl ParameterRange for X6Range {
    fn new(lower_bound: f64, upper_bound: f64) -> Result<Box<Self>, String> {
        Self::check(
            lower_bound,
            upper_bound,
            X6::min_value(),
            X6::max_value(),
            X6::description(),
        )?;
        Ok(Box::new(Self {
            lower_bound,
            upper_bound,
        }))
    }
}

/// The data to use to handle the calibration for the catchment or hydrological unit.
#[derive(Debug, Clone, Copy)]
pub struct VariableCatchmentData {
    /// The catchment os sub-catchment area (km2).
    pub area: f64,
    /// The range for the maximum capacity of the production store (mm/day)
    pub x1: X1Range,
    /// The range for the inter-catchment (or groundwater) exchange coefficient (mm/day).
    pub x2: X2Range,
    /// The range for the one-day-ahead maximum capacity of the routing store (mm/day)
    pub x3: X3Range,
    /// The range for the time base of unit hydrograph (days)
    pub x4: X4Range,
    /// The range for the inter-catchment exchange threshold.
    pub x5: X5Range,
    /// The range for the time constant of exponential store (mm)
    pub x6: X6Range,
}

#[cfg(test)]
mod tests {
    use crate::parameter::{Parameter, ParameterRange, X1Range, X1};

    #[test]
    fn test_too_small_parameter() {
        let x1 = X1::new(0.0);
        assert_eq!(
            x1.unwrap_err().to_string(),
            format!(
                "The {} must be larger than its minimum threshold ({})",
                X1::description(),
                X1::min_value()
            )
        )
    }

    #[test]
    fn test_too_large_parameter() {
        let x1 = X1::new(7000.0);
        assert_eq!(
            x1.unwrap_err().to_string(),
            format!(
                "The {} must be smaller than its maximum threshold ({})",
                X1::description(),
                X1::max_value()
            )
        )
    }

    #[test]
    fn test_too_small_min_bound() {
        let p = X1Range::new(-10.0, 100.0);
        assert_eq!(
            p.unwrap_err().to_string(),
            format!(
                "The lower bound (-10) for '{}' must be larger than the parameter minimum threshold ({})",
                X1::description(),
                X1::min_value()
            )
        )
    }

    #[test]
    fn test_too_large_max_bound() {
        let p = X1Range::new(0.1, 9000.0);
        assert_eq!(
            p.unwrap_err().to_string(),
            format!(
                "The upper bound (9000) for '{}' must be smaller than the parameter maximum threshold ({})",
                X1::description(),
                X1::max_value()
            )
        )
    }
}
