use crate::inputs::{CatchmentType, StoreLevels};
use crate::metric::CalibrationMetric;
use chrono::NaiveDate;

/// Outputs from a model time-step (one day)
#[derive(Debug, Clone)]
pub struct ModelStepData {
    /// The time
    pub time: NaiveDate,
    /// The potential evapotranspiration (mm)
    pub evapotranspiration: f64,
    /// The total precipitation (mm)
    pub precipitation: f64,
    /// Net rainfall (mm)
    pub net_rainfall: f64,
    /// The store levels
    pub store_levels: StoreLevels,
    /// part of the precipitation filling the production store (mm)
    pub storage_p: f64,
    /// actual evapotranspiration
    pub actual_evapotranspiration: f64,
    /// Catchment percolation (mm)
    pub percolation: f64,
    /// [`ModelStepData::net_rainfall`] - [`ModelStepData::storage_p`] + [`ModelStepData::percolation`] (mm)
    pub pr: f64,
    /// Potential third-exchange between catchments (mm)
    pub exchange: f64,
    /// Actual exchange between catchments from routing store (mm)
    pub exchange_from_routing_store: f64,
    /// Actual exchange between catchments from direct branch (after `UH2`) (mm)
    pub exchange_from_direct_branch: f64,
    /// Actual total exchange between catchments [`ModelStepData::exchange_from_routing_store`] + [`ModelStepData::exchange_from_direct_branch`] + [`ModelStepData::exchange`] (mm)
    pub actual_exchange: f64,
    /// Outflow from routing store (mm)
    pub routing_store_outflow: f64,
    /// Outflow from exponential store (mm)
    pub exponential_store_outflow: f64,
    /// Outflow from `UH2` branch after exchange (mm)
    pub outflow_from_uh2_branch: f64,
    /// Simulated outflow at catchment outlet (mm)
    pub run_off: f64,
}

/// A vector containing the results ([`ModelStepData`]) for each time step.
#[derive(Debug)]
pub struct ModelStepDataVector(pub Vec<ModelStepData>);

/// The model outputs
#[derive(Debug)]
pub struct GR6JOutputs {
    /// The results for each catchment model and time step.
    pub catchment_outputs: Vec<ModelStepDataVector>,
    /// The vector with the dates.
    pub time: Vec<NaiveDate>,
    /// The run-off for the catchment or the combined sub-catchment run-off in the unit of
    /// measurements specified in [`crate::inputs::RunOffUnit`].
    pub run_off: Vec<f64>,
    /// The calibration metrics. This is available only when [`crate::inputs::GR6JModelInputs::observed_runoff`]
    /// is provided.
    pub metrics: Option<CalibrationMetric>,
}

impl ModelStepDataVector {
    /// Get the time vector.
    pub fn time(&self) -> Vec<NaiveDate> {
        self.0.iter().map(|step_data| step_data.time).collect()
    }

    /// Get the run off in mm.
    pub fn run_off(&self, area: Option<f64>) -> Vec<f64> {
        self.0
            .iter()
            .map(|step_data| step_data.run_off * area.unwrap_or(1.0))
            .collect()
    }

    /// Get the production store level in mm.
    pub fn production_store(&self) -> Vec<f64> {
        self.0
            .iter()
            .map(|step_data| step_data.store_levels.production_store)
            .collect()
    }

    /// Get the routing store level in mm.
    pub fn routing_store(&self) -> Vec<f64> {
        self.0
            .iter()
            .map(|step_data| step_data.store_levels.routing_store)
            .collect()
    }

    /// Get the exponential store level in mm.
    pub fn exponential_store(&self) -> Vec<f64> {
        self.0
            .iter()
            .map(|step_data| step_data.store_levels.exponential_store)
            .collect()
    }
}

/// The value and ideal value of a calibration metric.
#[derive(Debug)]
pub struct CalibrationMetricOutput {
    pub value: f64,
    pub ideal_value: f64,
}

/// The model calibration outputs
#[derive(Debug)]
pub struct CalibrationOutputs {
    /// The vector with the dates.
    pub time: Vec<NaiveDate>,
    /// The run-off for each simulated model. The size of the vector is [`crate::inputs::CalibrationInputs::sample_size`].
    pub run_off: Vec<Vec<f64>>,
    /// The list of catchment data (for one catchment or multiple sub-catchments) for each
    /// simulated model. The size of this vector is [`crate::inputs::CalibrationInputs::sample_size`].
    /// Each vector item contains the calibration parameters generated by the Latin Hypercube method.
    pub catchment: Vec<CatchmentType>,
    /// The list of calibration metrics for each simulated model. Use this to assess the calibration
    /// accuracy. The size of this vector is [`crate::inputs::CalibrationInputs::sample_size`].
    pub metrics: Vec<CalibrationMetric>,
}
