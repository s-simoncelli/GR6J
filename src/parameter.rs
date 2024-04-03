use std::fmt;
use std::fmt::{Debug, Formatter};

/// Six free parameters for a GR6J model
#[derive(Debug, Copy, Clone)]
pub enum Parameter {
    // Maximum capacity of the production store (mm/day)
    X1(f64),
    /// Inter-catchment (or groundwater) exchange coefficient (mm/day). X2 can be positive
    /// or negative to simulate imports or exports of water with deep aquifers or
    /// surrounding catchments.
    X2(f64),
    /// One-day-ahead maximum capacity of the routing store (mm/day)
    X3(f64),
    /// Time base of unit hydrograph `UH1` (days)
    X4(f64),
    /// Inter-catchment exchange threshold. This is a dimensionless threshold parameter that
    /// allows a change in the direction of the groundwater exchange depending on the capacity
    /// of the routing store level `R`.
    X5(f64),
    /// Time constant of exponential store (mm)
    X6(f64),
}

impl Parameter {
    /// Get the parameter value.
    ///
    /// returns: f64
    pub fn value(&self) -> f64 {
        match self {
            Parameter::X1(v) => *v,
            Parameter::X2(v) => *v,
            Parameter::X3(v) => *v,
            Parameter::X4(v) => *v,
            Parameter::X5(v) => *v,
            Parameter::X6(v) => *v,
        }
    }

    /// Get the minimum allowed value for the parameter.
    ///
    /// returns: f64
    pub fn min_threshold(&self) -> f64 {
        match self {
            Parameter::X1(_) => 1e-2,
            Parameter::X2(_) => -5.0,
            Parameter::X3(_) => 1e-2,
            Parameter::X4(_) => 0.5,
            Parameter::X5(_) => -4.0,
            Parameter::X6(_) => 1e-2,
        }
    }

    /// Get the maximum allowed value for the parameter.
    ///
    /// returns: f64
    pub fn max_threshold(&self) -> f64 {
        match self {
            Parameter::X1(_) => 2500.0,
            Parameter::X2(_) => 5.0,
            Parameter::X3(_) => 1000.0,
            Parameter::X4(_) => 10.0,
            Parameter::X5(_) => 4.0,
            Parameter::X6(_) => 20.0,
        }
    }

    /// Get the parameter unit.
    ///
    /// returns: &str
    pub fn unit(&self) -> &str {
        match self {
            Parameter::X1(_) => "mm",
            Parameter::X2(_) => "mm",
            Parameter::X3(_) => "mm",
            Parameter::X4(_) => "days",
            Parameter::X5(_) => "-",
            Parameter::X6(_) => "mm",
        }
    }

    /// Get the parameter description.
    ///
    /// returns: &str
    pub fn description(&self) -> &str {
        match self {
            Parameter::X1(_) => "production store capacity",
            Parameter::X2(_) => "maximum capacity of the routing store",
            Parameter::X3(_) => "routing store capacity",
            Parameter::X4(_) => "time base of unit hydrograph",
            Parameter::X5(_) => "inter-catchment exchange threshold",
            Parameter::X6(_) => "time constant of exponential store",
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Parameter::X1(v) => write!(f, "{} (X1={})", self.description(), v),
            Parameter::X2(v) => write!(f, "{} (X2={})", self.description(), v),
            Parameter::X3(v) => write!(f, "{} (X3={})", self.description(), v),
            Parameter::X4(v) => write!(f, "{} (X4={})", self.description(), v),
            Parameter::X5(v) => write!(f, "{} (X5={})", self.description(), v),
            Parameter::X6(v) => write!(f, "{} (X6={})", self.description(), v),
        }
    }
}
