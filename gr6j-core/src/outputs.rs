use crate::inputs::{CatchmentData, StoreLevels};
use crate::metric::CalibrationMetric;
use crate::parameter::Parameter;
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

/// This structure contains a vector with the metric values.
#[derive(Debug)]
pub struct CalibrationMetricVector(pub(crate) Vec<CalibrationMetric>);

impl CalibrationMetricVector {
    /// Get the vector of the Nash-Sutcliffe coefficients for all models.
    pub fn nash_sutcliffe(&self) -> Vec<f64> {
        self.0.iter().map(|m| m.nash_sutcliffe.value).collect()
    }
    /// Get the vector of the log Nash-Sutcliffe coefficients for all models.
    pub fn log_nash_sutcliffe(&self) -> Vec<f64> {
        self.0.iter().map(|m| m.log_nash_sutcliffe.value).collect()
    }
    /// Get the vector of the non-parametric Kling-Gupta coefficients for all models.
    pub fn non_parametric_kling_gupta(&self) -> Vec<f64> {
        self.0.iter().map(|m| m.non_parametric_kling_gupta.value).collect()
    }
    /// Get the vector of the root-mean-square errors for all models.
    pub fn rmse(&self) -> Vec<f64> {
        self.0.iter().map(|m| m.rmse.value).collect()
    }
    /// Get the vector of the volume errors for all models.
    pub fn volume_error(&self) -> Vec<f64> {
        self.0.iter().map(|m| m.volume_error.value).collect()
    }
    /// Get the vector with the names of the calculated metrics in the vector.
    pub fn metric_names(&self) -> [String; 5] {
        [
            self.0.first().unwrap().nash_sutcliffe.name.clone(),
            self.0[0].log_nash_sutcliffe.name.clone(),
            self.0[0].non_parametric_kling_gupta.name.clone(),
            self.0[0].rmse.name.clone(),
            self.0[0].volume_error.name.clone(),
        ]
    }
}
/// The parameter values generated during the calibration
#[derive(Debug)]
pub struct CalibrationParameterValues {
    pub x1: f64,
    pub x2: f64,
    pub x3: f64,
    pub x4: f64,
    pub x5: f64,
    pub x6: f64,
}

impl From<CatchmentData> for CalibrationParameterValues {
    fn from(data: CatchmentData) -> Self {
        Self {
            x1: data.x1.value(),
            x2: data.x2.value(),
            x3: data.x3.value(),
            x4: data.x4.value(),
            x5: data.x5.value(),
            x6: data.x6.value(),
        }
    }
}

/// This structure contains a vector with the parameter values generated by the Latin Hypercube
/// method for a model.
#[derive(Debug)]
pub struct CalibrationParameterValueVector(pub Vec<CalibrationParameterValues>);

impl CalibrationParameterValueVector {
    /// Get the vector containing the vector of parameter values.
    pub fn to_vec(&self) -> Vec<Vec<f64>> {
        vec![
            self.to_vec_x1(),
            self.to_vec_x2(),
            self.to_vec_x3(),
            self.to_vec_x4(),
            self.to_vec_x5(),
            self.to_vec_x6(),
        ]
    }
    /// Get the vector of X1 values for all models.
    pub fn to_vec_x1(&self) -> Vec<f64> {
        self.0.iter().map(|c| c.x1).collect()
    }
    /// Get the vector of X2 values for all models.
    pub fn to_vec_x2(&self) -> Vec<f64> {
        self.0.iter().map(|c| c.x2).collect()
    }
    /// Get the vector of X3 values for all models.
    pub fn to_vec_x3(&self) -> Vec<f64> {
        self.0.iter().map(|c| c.x3).collect()
    }
    /// Get the vector of X4 values for all models.
    pub fn to_vec_x4(&self) -> Vec<f64> {
        self.0.iter().map(|c| c.x4).collect()
    }
    /// Get the vector of X5 values for all models.
    pub fn to_vec_x5(&self) -> Vec<f64> {
        self.0.iter().map(|c| c.x5).collect()
    }
    /// Get the vector of X6 values for all models.
    pub fn to_vec_x6(&self) -> Vec<f64> {
        self.0.iter().map(|c| c.x6).collect()
    }
}

/// The model calibration outputs
#[derive(Debug)]
pub struct CalibrationOutputs {
    /// The vector with the dates.
    pub time: Vec<NaiveDate>,
    /// The run-off for each simulated model. The size of the vector is
    /// [`crate::inputs::CalibrationInputs::sample_size`].
    pub run_off: Vec<Vec<f64>>,
    /// The vector of parameter values grouped by catchment. The size of this vector equals the
    /// number of sub-catchments provided in [`crate::inputs::CalibrationInputs::catchment`] (this
    /// is 1 if [`crate::inputs::CalibrationInputs::catchment`] is
    /// [`crate::inputs::CalibrationCatchmentType::OneCatchment`]). Each vector then contains
    /// another vector of size [`crate::inputs::CalibrationInputs::sample_size`] with the parameter
    /// values generated by the Latin Hypercube method.
    pub parameters: Vec<CalibrationParameterValueVector>,
    /// The list of calibration metrics for each simulated model. Use this to assess the calibration
    /// accuracy. The size of this vector is [`crate::inputs::CalibrationInputs::sample_size`].
    pub metrics: CalibrationMetricVector,
}
