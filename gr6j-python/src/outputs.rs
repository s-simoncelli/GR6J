use crate::inputs::StoreLevels;
use ::gr6j::metric::{CalibrationMetric as RsCalibrationMetric, Metric as RsMetric};
use ::gr6j::outputs::ModelStepData as RsModelStepData;
use chrono::NaiveDate;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(get_all)]
#[derive(Clone)]
pub struct ModelStepData {
    time: NaiveDate,
    evapotranspiration: f64,
    precipitation: f64,
    net_rainfall: f64,
    store_levels: StoreLevels,
    storage_p: f64,
    actual_evapotranspiration: f64,
    percolation: f64,
    pr: f64,
    exchange: f64,
    exchange_from_routing_store: f64,
    exchange_from_direct_branch: f64,
    actual_exchange: f64,
    routing_store_outflow: f64,
    exponential_store_outflow: f64,
    outflow_from_uh2_branch: f64,
    run_off: f64,
}

impl ModelStepData {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ModelStepData(t={})", self.time,))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass(get_all)]
#[derive(Clone)]
pub struct Metric {
    name: String,
    ideal_value: f64,
    value: f64,
}

impl Metric {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Metric(name={},value={})", self.name, self.value))
    }

    pub(crate) fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

impl From<RsMetric> for Metric {
    fn from(m: RsMetric) -> Self {
        Self {
            name: m.name,
            ideal_value: m.ideal_value,
            value: m.value,
        }
    }
}

#[pyclass(get_all)]
#[derive(Clone)]
pub struct CalibrationMetric {
    nash_sutcliffe: Metric,
    log_nash_sutcliffe: Metric,
    kling_gupta2009: Metric,
    kling_gupta2012: Metric,
    non_paramettric_kling_gupta: Metric,
}

impl From<RsCalibrationMetric> for CalibrationMetric {
    fn from(m: RsCalibrationMetric) -> Self {
        Self {
            nash_sutcliffe: m.nash_sutcliffe.into(),
            log_nash_sutcliffe: m.log_nash_sutcliffe.into(),
            kling_gupta2009: m.kling_gupta2009.into(),
            kling_gupta2012: m.kling_gupta2012.into(),
            non_paramettric_kling_gupta: m.non_parametric_kling_gupta.into(),
        }
    }
}

impl CalibrationMetric {
    fn __repr__(&self) -> PyResult<String> {
        Ok("CalibrationMetric()".to_string())
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

impl From<RsModelStepData> for ModelStepData {
    fn from(value: RsModelStepData) -> Self {
        ModelStepData {
            time: value.time,
            evapotranspiration: value.evapotranspiration,
            precipitation: value.precipitation,
            net_rainfall: value.net_rainfall,
            store_levels: StoreLevels::new(
                value.store_levels.production_store,
                value.store_levels.exponential_store,
                value.store_levels.routing_store,
            ),
            storage_p: value.storage_p,
            actual_evapotranspiration: value.actual_evapotranspiration,
            percolation: value.percolation,
            pr: value.pr,
            exchange: value.exchange,
            exchange_from_routing_store: value.exchange_from_routing_store,
            exchange_from_direct_branch: value.exchange_from_direct_branch,
            actual_exchange: value.actual_exchange,
            routing_store_outflow: value.routing_store_outflow,
            exponential_store_outflow: value.exponential_store_outflow,
            outflow_from_uh2_branch: value.outflow_from_uh2_branch,
            run_off: value.run_off,
        }
    }
}

#[pyclass(get_all)]
pub struct GR6JOutputs {
    pub catchment_outputs: Vec<Vec<ModelStepData>>,
    pub time: Vec<NaiveDate>,
    pub run_off: Vec<f64>,
    pub metrics: Option<CalibrationMetric>,
}

#[pymethods]
impl GR6JOutputs {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "GR6JOutputs with {} time steps (from={},to={})",
            self.time.len(),
            self.time.first().unwrap(),
            self.time.last().unwrap()
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }

    pub fn to_dataframe(&self) -> PyResult<PyObject> {
        let df: PyObject = Python::with_gil(|py| {
            let pd = py.import_bound("pandas")?;
            let builtins = PyModule::import_bound(py, "builtins")?;
            let data: PyObject = builtins
                .getattr("zip")?
                .call1((self.time.clone(), self.run_off.clone()))?
                .extract()?;
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("columns", ["Time", "Run off"])?;

            let df: PyObject = pd.call_method("DataFrame", (data,), Some(&kwargs))?.extract()?;
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("inplace", true)?;
            df.call_method_bound(py, "set_index", ("Time",), Some(&kwargs))?;
            Ok::<PyObject, PyErr>(df)
        })?;

        Ok(df)
    }
}
