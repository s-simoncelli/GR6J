use std::fmt;
use std::fmt::Formatter;

pub trait Parameter<'a>: fmt::Display {
    fn new(value: f64) -> Result<Box<Self>, String>;

    /// Check that a value is within the min and max parameter bounds.
    fn check(value: f64) -> Result<(), String> {
        // check min
        if value < Self::min() {
            return Err(format!(
                "The {} must be larger than its minimum threshold ({})",
                Self::description(),
                Self::min(),
            ));
        }
        // check max
        if value > Self::max() {
            return Err(format!(
                "The {} must be smaller than its maximum threshold ({})",
                Self::description(),
                Self::max(),
            ));
        }

        Ok(())
    }

    /// Return the parameter value.
    fn value(&self) -> f64;

    /// Return the parameter minimum value.
    fn min() -> f64;

    /// Return the parameter maximum value.
    fn max() -> f64;

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

    fn min() -> f64 {
        1e-2
    }

    fn max() -> f64 {
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

    fn min() -> f64 {
        -5.0
    }

    fn max() -> f64 {
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

    fn min() -> f64 {
        1e-2
    }

    fn max() -> f64 {
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

    fn min() -> f64 {
        0.5
    }

    fn max() -> f64 {
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

    fn min() -> f64 {
        -4.0
    }

    fn max() -> f64 {
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

    fn min() -> f64 {
        1e-2
    }

    fn max() -> f64 {
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
