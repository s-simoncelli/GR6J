use crate::parameter::Parameter;
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

impl Debug for ModelPeriod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

#[derive(Debug)]
pub enum CatchmentType {
    OneCatchment(CatchmentData),
    SubCatchments(Vec<CatchmentData>),
}

/// The data for the catchment or hydrological unit.
#[derive(Debug, Clone, Copy)]
pub struct CatchmentData {
    /// The catchment os sub-catchment area (km2).
    pub area: f64,
    /// Maximum capacity of the production store (mm/day)
    pub x1: Parameter,
    /// Inter-catchment (or groundwater) exchange coefficient (mm/day). X2 can be positive
    /// or negative to simulate imports or exports of water with deep aquifers or
    /// surrounding catchments.
    pub x2: Parameter,
    /// One-day-ahead maximum capacity of the routing store (mm/day)
    pub x3: Parameter,
    /// Time base of unit hydrograph `UH1` (days)
    pub x4: Parameter,
    /// Inter-catchment exchange threshold. This is a dimensionless threshold parameter that
    /// allows a change in the direction of the groundwater exchange depending on the capacity
    /// of the routing store level `R`.
    pub x5: Parameter,
    /// Time constant of exponential store (mm)
    pub x6: Parameter,
    /// The store levels
    pub store_levels: Option<StoreLevels>,
}

// TODO add doc explaining how this works
#[derive(Debug)]
pub struct GR6JModelInputs {
    /// Vector of time
    pub time: Vec<NaiveDate>,
    /// Input vector of total precipitation (mm/day)
    pub precipitation: Vec<f64>,
    /// input vector of potential evapotranspiration (PE) (mm/day)
    pub evapotranspiration: Vec<f64>,
    /// Area and GR6J parameters for the catchment or a list of areas and parameters if you would
    /// like to divide the catchment into sub-catchments or hydrological units (for example based
    /// on surface type).
    pub catchment: CatchmentType,
    /// The start and end date of the model. The model can be run on a shorter time period
    /// compared to `time`
    pub run_period: ModelPeriod,
    /// The start and end date of the warm-up period. If `None` and the `run_period.start` allows,
    /// the one-year period preceding the `run_period.start` is used.
    pub warmup_period: Option<ModelPeriod>,
    /// Whether to export charts, the simulated run-off and other diagnostic file into a sub-folder
    /// inside the given destination folder. The sub-folder will be named with the run timestamp.
    pub destination: Option<PathBuf>,
}
