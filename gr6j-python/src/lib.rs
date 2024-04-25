use crate::inputs::{CatchmentData, GR6JModelInputs, ModelPeriod, RunOffUnit};
use crate::outputs::{CalibrationMetric, GR6JOutputs, Metric, ModelStepData};
use ::gr6j::inputs::{GR6JModelInputs as RsGR6JModelInputs, ModelPeriod as RsModelPeriod};
use ::gr6j::model::GR6JModel as RsGR6JModel;
use inputs::StoreLevels;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

mod inputs;
mod outputs;

#[pyclass]
struct GR6JModel {
    rs_model: RsGR6JModel,
    run_period: RsModelPeriod,
}

#[pymethods]
impl GR6JModel {
    #[new]
    fn rs_new(inputs: GR6JModelInputs) -> PyResult<GR6JModel> {
        let run_period = inputs.run_period.rs_period;
        let inputs = RsGR6JModelInputs {
            time: &inputs.time,
            precipitation: &inputs.precipitation,
            evapotranspiration: &inputs.evapotranspiration,
            catchment: inputs.rs_catchment,
            run_period,
            warmup_period: inputs.warmup_period.map(|d| d.rs_period),
            destination: inputs.destination,
            observed_runoff: inputs.observed_runoff.as_deref(),
            run_off_unit: inputs.run_off_unit.unwrap_or_default().into(),
        };
        let model = GR6JModel {
            run_period,
            rs_model: RsGR6JModel::new(inputs).map_err(|e| PyValueError::new_err(e.to_string()))?,
        };
        Ok(model)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "GR6JModel(from={},to={})",
            self.run_period.start, self.run_period.end
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }

    /// Run the model
    fn run(&mut self) -> PyResult<GR6JOutputs> {
        let results = self
            .rs_model
            .run()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut model_results: Vec<Vec<ModelStepData>> = vec![];
        for r in results.catchment_outputs.iter() {
            model_results.push(
                r.0.to_vec()
                    .iter()
                    .map(|x| Into::<ModelStepData>::into(x.clone()))
                    .collect(),
            );
        }

        Ok(GR6JOutputs {
            catchment_outputs: model_results,
            time: results.time,
            run_off: results.run_off,
            metrics: results.metrics.map(Into::into),
        })
    }
}

#[pymodule]
fn gr6j(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StoreLevels>()?;
    m.add_class::<CatchmentData>()?;
    m.add_class::<Metric>()?;
    m.add_class::<CalibrationMetric>()?;
    m.add_class::<ModelPeriod>()?;
    m.add_class::<RunOffUnit>()?;
    m.add_class::<GR6JModelInputs>()?;
    m.add_class::<GR6JModel>()?;

    Ok(())
}
