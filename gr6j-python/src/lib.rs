use ::gr6j::error::ModelPeriodError;
use ::gr6j::inputs::{
    CatchmentData as RsCatchmentData, GR6JModelInputs as RsGR6JModelInputs, ModelPeriod as RsModelPeriod,
    RunOffUnit as RsRunOffUnit, StoreLevels as RsStoreLevels,
};
use ::gr6j::metric::{CalibrationMetric as RsCalibrationMetric, Metric as RsMetric};
use ::gr6j::model::GR6JModel as RsGR6JModel;
use ::gr6j::outputs::ModelStepData as RsModelStepData;
use ::gr6j::parameter::{Parameter, X1, X2, X3, X4, X5, X6};
use chrono::NaiveDate;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::PyTypeInfo;
use std::fmt::Debug;
use std::path::PathBuf;

#[pyclass(get_all)]
#[derive(Debug, Clone, Copy)]
struct StoreLevels {
    production_store: f64,
    routing_store: f64,
    exponential_store: f64,
}

impl From<StoreLevels> for RsStoreLevels {
    fn from(s: StoreLevels) -> RsStoreLevels {
        RsStoreLevels {
            production_store: s.production_store,
            routing_store: s.routing_store,
            exponential_store: s.exponential_store,
        }
    }
}

#[pymethods]
impl StoreLevels {
    #[new]
    fn new(production_store: f64, routing_store: f64, exponential_store: f64) -> Self {
        StoreLevels {
            production_store,
            routing_store,
            exponential_store,
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "StoreLevels(production_store={},routing_store={},exponential_store={})",
            self.production_store, self.routing_store, self.exponential_store
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
struct CatchmentData {
    #[pyo3(get)]
    area: f64,
    #[pyo3(get)]
    x1: f64,
    #[pyo3(get)]
    x2: f64,
    #[pyo3(get)]
    x3: f64,
    #[pyo3(get)]
    x4: f64,
    #[pyo3(get)]
    x5: f64,
    #[pyo3(get)]
    x6: f64,
    #[pyo3(get)]
    store_levels: Option<StoreLevels>,
    rs_catchment: RsCatchmentData,
}

#[pymethods]
#[allow(clippy::too_many_arguments)]
impl CatchmentData {
    #[new]
    fn new(
        area: f64,
        x1: f64,
        x2: f64,
        x3: f64,
        x4: f64,
        x5: f64,
        x6: f64,
        store_levels: Option<StoreLevels>,
    ) -> PyResult<CatchmentData> {
        let rs_catchment = RsCatchmentData {
            area,
            x1: X1::new(x1).map_err(|e| PyValueError::new_err(e.to_string()))?,
            x2: X2::new(x2).map_err(|e| PyValueError::new_err(e.to_string()))?,
            x3: X3::new(x3).map_err(|e| PyValueError::new_err(e.to_string()))?,
            x4: X4::new(x4).map_err(|e| PyValueError::new_err(e.to_string()))?,
            x5: X5::new(x5).map_err(|e| PyValueError::new_err(e.to_string()))?,
            x6: X6::new(x6).map_err(|e| PyValueError::new_err(e.to_string()))?,
            store_levels: store_levels.map(Into::into),
        };
        Ok(CatchmentData {
            area,
            x1,
            x2,
            x3,
            x4,
            x5,
            x6,
            store_levels,
            rs_catchment,
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        let store_levels = if let Some(levels) = &self.store_levels {
            levels.__repr__().unwrap()
        } else {
            "None".to_string()
        };
        Ok(format!(
            "CatchmentData(area={},x1={},x2={},x3={},x4={},x5={},x6={},store_levels={})",
            self.area, self.x1, self.x2, self.x3, self.x4, self.x5, self.x6, store_levels
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone, Copy, Debug)]
struct ModelPeriod {
    #[pyo3(get)]
    start: NaiveDate,
    #[pyo3(get)]
    end: NaiveDate,
    rs_period: RsModelPeriod,
}

impl TryFrom<ModelPeriod> for RsModelPeriod {
    type Error = ModelPeriodError;
    fn try_from(s: ModelPeriod) -> Result<Self, Self::Error> {
        RsModelPeriod::new(s.start, s.end)
    }
}

#[pymethods]
impl ModelPeriod {
    #[new]
    fn new(start: NaiveDate, end: NaiveDate) -> PyResult<Self> {
        let rs_period = RsModelPeriod::new(start, end).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(ModelPeriod { start, end, rs_period })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ModelPeriod(start={},end={})", self.start, self.end,))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone, Debug, Default)]
enum RunOffUnit {
    #[pyo3(name = "NO_CONVERSION")]
    #[default]
    NoConversion,
    #[pyo3(name = "CUBIC_METRE_PER_DAY")]
    CubicMetrePerDay,
    #[pyo3(name = "ML_PER_DAY")]
    MlPerDay,
    #[pyo3(name = "CUBIC_METRE_PER_SECOND")]
    CubicMetrePerSecond,
}

impl From<RunOffUnit> for RsRunOffUnit {
    fn from(s: RunOffUnit) -> RsRunOffUnit {
        match s {
            RunOffUnit::NoConversion => RsRunOffUnit::NoConversion,
            RunOffUnit::CubicMetrePerDay => RsRunOffUnit::CubicMetrePerDay,
            RunOffUnit::MlPerDay => RsRunOffUnit::MlPerDay,
            RunOffUnit::CubicMetrePerSecond => RsRunOffUnit::CubicMetrePerSecond,
        }
    }
}

#[pyclass]
#[derive(Clone)]
struct CatchmentDataVec(Vec<CatchmentData>);

impl TryFrom<PyObject> for CatchmentDataVec {
    type Error = PyErr;
    fn try_from(value: PyObject) -> Result<Self, Self::Error> {
        // Convert the catchment data
        Python::with_gil(|py| {
            let module = PyModule::import_bound(py, "builtins")?;
            let isinstance = module.getattr("isinstance")?;
            let is_one: bool = isinstance
                .call1((&value, CatchmentData::type_object_bound(py)))?
                .extract()?;

            let is_list: bool = isinstance.call1((&value, PyList::type_object_bound(py)))?.extract()?;

            if is_one {
                return Ok(CatchmentDataVec(vec![value.extract::<CatchmentData>(py)?]));
            } else if is_list {
                // The list must contain instances of CatchmentData
                let res: Vec<PyObject> = value.extract(py)?;
                for data in res.iter() {
                    let is_catchment: bool = isinstance
                        .call1((data, CatchmentData::type_object_bound(py)))?
                        .extract()?;
                    if !is_catchment {
                        return Err(PyValueError::new_err(
                            "The catchment argument must be a list of CatchmentData classes",
                        ));
                    }
                }
                return Ok(CatchmentDataVec(value.extract::<Vec<CatchmentData>>(py)?));
            }
            Err(PyValueError::new_err(
                "The catchment argument must be an instance of one CatchmentData or a list of CatchmentData classes",
            ))
        })
    }
}

#[pymethods]
impl CatchmentDataVec {
    fn __repr__(&self) -> PyResult<String> {
        match self.0.len() {
            1 => Ok(self.0[0].__repr__().unwrap()),
            _ => {
                let mut str: String = "[".to_string();
                for data in self.0.iter() {
                    str += &data.__repr__().unwrap();
                    str += ",";
                }
                str.pop();
                str += "]";
                Ok(str)
            }
        }
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
struct GR6JModelInputs {
    #[pyo3(get)]
    time: Vec<NaiveDate>,
    #[pyo3(get)]
    precipitation: Vec<f64>,
    #[pyo3(get)]
    evapotranspiration: Vec<f64>,
    #[pyo3(get)] // keep this to allow user access
    catchment: CatchmentDataVec,
    rs_catchment: Vec<RsCatchmentData>,
    #[pyo3(get)]
    run_period: ModelPeriod,
    #[pyo3(get)]
    warmup_period: Option<ModelPeriod>,
    #[pyo3(get)]
    destination: Option<PathBuf>,
    #[pyo3(get)]
    observed_runoff: Option<Vec<f64>>,
    #[pyo3(get)]
    run_off_unit: Option<RunOffUnit>,
}

#[pymethods]
impl GR6JModelInputs {
    #[new]
    #[pyo3(signature = (time,precipitation,evapotranspiration,catchment,run_period,warmup_period=None,destination=None,observed_runoff=None,run_off_unit=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: Vec<NaiveDate>,
        precipitation: Vec<f64>,
        evapotranspiration: Vec<f64>,
        catchment: PyObject,
        run_period: ModelPeriod,
        warmup_period: Option<ModelPeriod>,
        destination: Option<PathBuf>,
        observed_runoff: Option<Vec<f64>>,
        run_off_unit: Option<RunOffUnit>,
    ) -> PyResult<Self> {
        let catchment = CatchmentDataVec::try_from(catchment)?;
        let rs_catchment = catchment.0.iter().map(|d| d.rs_catchment.clone()).collect();
        // let rs_catchment: Vec<RsCatchmentData> = catchment
        //     .clone()
        //     .0
        //     .into_iter()
        //     .map(TryInto::try_into)
        //     .collect::<Result<Vec<RsCatchmentData>, _>>()
        //     .map_err(|e: LoadModelError| PyValueError::new_err(e.to_string()))?;

        Ok(GR6JModelInputs {
            time,
            precipitation,
            evapotranspiration,
            catchment,
            rs_catchment,
            run_period,
            warmup_period,
            destination,
            observed_runoff,
            run_off_unit,
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        let warmup_period = match &self.warmup_period {
            None => "None".to_string(),
            Some(p) => p.__repr__().unwrap().to_string(),
        };
        let destination = match &self.destination {
            None => "None",
            Some(d) => d.to_str().unwrap(),
        };
        let run_off_unit = match &self.run_off_unit {
            None => "None",
            Some(u) => u.__pyo3__repr__(),
        };
        Ok(format!(
            "GR6JModelInputs(run_period={},warmup_period={},destination={},run_off_unit={})",
            self.run_period.__repr__().unwrap(),
            warmup_period,
            destination,
            run_off_unit
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass(get_all)]
#[derive(Clone)]
struct ModelStepData {
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

    fn __str__(&self) -> String {
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
            store_levels: StoreLevels {
                production_store: value.store_levels.production_store,
                exponential_store: value.store_levels.exponential_store,
                routing_store: value.store_levels.routing_store,
            },
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
struct GR6JOutputs {
    catchment_outputs: Vec<Vec<ModelStepData>>,
    time: Vec<NaiveDate>,
    run_off: Vec<f64>,
    metrics: Option<CalibrationMetric>,
}

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
}

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
