use crate::error::ModelPeriodError;
use crate::parameter::{X1, X2, X3, X4, X5, X6};
use chrono::NaiveDate;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

/// Struct to define the store levels
#[derive(Debug, Clone, Copy)]
pub struct StoreLevels {
    /// The production store level (mm)
    pub production_store: f64,
    /// The routing store level (mm)
    pub routing_store: f64,
    /// The exponential store level (mm)
    pub exponential_store: f64,
}

impl Default for StoreLevels {
    fn default() -> Self {
        StoreLevels {
            production_store: 0.3,
            routing_store: 0.5,
            exponential_store: 0.0,
        }
    }
}

/// Struct to define a model time range
#[derive(Clone, Copy)]
pub struct ModelPeriod {
    /// The period start date
    pub start: NaiveDate,
    /// The period end date
    pub end: NaiveDate,
}

impl ModelPeriod {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self, ModelPeriodError> {
        if start >= end {
            return Err(ModelPeriodError::DateTooSmall(start, end));
        }
        Ok(ModelPeriod { start, end })
    }
}

impl Debug for ModelPeriod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// Define the type of catchment. This can be one catchment or an aggregation of sub-catchments (or
/// hydrological units).
#[derive(Debug)]
pub enum CatchmentType {
    /// One catchment with one GR6J model.
    OneCatchment(CatchmentData),
    /// A list of sub-catchments with multiple GR6J model. Each model will run independently.
    SubCatchments(Vec<CatchmentData>),
}

/// The data for the catchment or hydrological unit.
#[derive(Debug, Clone)]
pub struct CatchmentData {
    /// The catchment os sub-catchment area (km2).
    pub area: f64,
    /// Maximum capacity of the production store (mm/day).
    pub x1: Box<X1>,
    /// Inter-catchment (or groundwater) exchange coefficient (mm/day). X2 can be positive
    /// or negative to simulate imports or exports of water with deep aquifers or
    /// surrounding catchments.
    pub x2: Box<X2>,
    /// One-day-ahead maximum capacity of the routing store (mm/day).
    pub x3: Box<X3>,
    /// Time base of unit hydrograph `UH1` (days).
    pub x4: Box<X4>,
    /// Inter-catchment exchange threshold. This is a dimensionless threshold parameter that
    /// allows a change in the direction of the groundwater exchange depending on the capacity
    /// of the routing store level `R`.
    pub x5: Box<X5>,
    /// Time constant of exponential store (mm)
    pub x6: Box<X6>,
    /// The store levels
    pub store_levels: Option<StoreLevels>,
}

/// Convert the run-off to the desired unit of measurement
#[derive(Debug, Default, Clone)]
pub enum RunOffUnit {
    #[default]
    /// Keep the run-off in mm*km2/d
    NoConversion,
    /// Convert the run-off to m³/d
    CubicMetrePerDay,
    /// Convert the run-off to Ml/d
    MlPerDay,
    /// Convert the run-off to m³/s
    CubicMetrePerSecond,
}

impl RunOffUnit {
    /// Get the conversion factor to multiply with the run-off data.
    pub fn conv_factor(&self) -> f64 {
        match self {
            RunOffUnit::NoConversion => 1.0,
            RunOffUnit::CubicMetrePerDay => 1.0 / 1000.0,
            RunOffUnit::CubicMetrePerSecond => 86400.0 / 1000.0,
            RunOffUnit::MlPerDay => 1.0,
        }
    }

    /// Get the conversion factor unit.
    pub fn unit_label(&self) -> &str {
        match self {
            RunOffUnit::NoConversion => "-",
            RunOffUnit::CubicMetrePerDay => "m³/d",
            RunOffUnit::CubicMetrePerSecond => "m³/s",
            RunOffUnit::MlPerDay => "Ml/d",
        }
    }
}

/// Inputs to the GR6J model.
#[derive(Debug)]
pub struct GR6JModelInputs<'a> {
    /// Vector of time.
    pub time: &'a [NaiveDate],
    /// Input vector of total precipitation (mm/day).
    pub precipitation: &'a [f64],
    /// input vector of potential evapotranspiration (PE) (mm/day).
    pub evapotranspiration: &'a [f64],
    /// Area and GR6J parameters for the catchment or a list of areas and parameters if you would
    /// like to divide the catchment into sub-catchments or hydrological units (for example based
    /// on surface type). If more than one catchment is supplied, the tool will run the GR6J models
    /// independently and combine the total flow.
    pub catchment: CatchmentType,
    /// The start and end date of the model. The model can be run on a shorter time period
    /// compared to `time`.
    pub run_period: ModelPeriod,
    /// The start and end date of the warm-up period. If `None` and [`ModelPeriod::start`] allows,
    /// the one-year period preceding the [`ModelPeriod::start`] is used.
    pub warmup_period: Option<ModelPeriod>,
    /// Whether to export charts, the simulated run-off and other diagnostic file into a sub-folder
    /// inside the given destination folder. The sub-folder will be named with the run timestamp.
    pub destination: Option<PathBuf>,
    /// The time series of the observed run-off. The time-series and its FDC will be plotted against
    /// the simulated run-off if [`GR6JModelInputs::destination`] is provided.
    pub observed_runoff: Option<&'a [f64]>,
    /// Convert the run-off to the desired unit of measurement.
    pub run_off_unit: RunOffUnit,
}
