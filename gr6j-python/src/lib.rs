use ::gr6j::inputs::{
    CatchmentData as RsCatchmentData, CatchmentType as RsCatchmentType, GR6JModelInputs as RsGR6JModelInputs,
    ModelPeriod as RsModelPeriod, RunOffUnit as RsRunOffUnit, StoreLevels as RsStorelevels,
};
use ::gr6j::model::GR6JModel as RsGR6JModel;
use ::gr6j::outputs::ModelStepData as RsModelStepData;
use ::gr6j::parameter::Parameter;
use chrono::NaiveDate;
use pyo3::exceptions::PyValueError;
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

impl From<StoreLevels> for RsStorelevels {
    fn from(s: StoreLevels) -> RsStorelevels {
        RsStorelevels {
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

#[pyclass(get_all)]
#[derive(Clone)]
struct CatchmentData {
    area: f64,
    x1: f64,
    x2: f64,
    x3: f64,
    x4: f64,
    x5: f64,
    x6: f64,
    store_levels: Option<StoreLevels>,
}

impl From<CatchmentData> for RsCatchmentData {
    fn from(s: CatchmentData) -> RsCatchmentData {
        RsCatchmentData {
            area: s.area,
            x1: Parameter::X1(s.x1),
            x2: Parameter::X2(s.x2),
            x3: Parameter::X3(s.x3),
            x4: Parameter::X4(s.x4),
            x5: Parameter::X5(s.x5),
            x6: Parameter::X6(s.x6),
            store_levels: s.store_levels.map(|l| l.into()),
        }
    }
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
    ) -> CatchmentData {
        CatchmentData {
            area,
            x1,
            x2,
            x3,
            x4,
            x5,
            x6,
            store_levels,
        }
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

#[pyclass(get_all)]
#[derive(Clone, Debug)]
struct ModelPeriod {
    start: NaiveDate,
    end: NaiveDate,
}

impl From<ModelPeriod> for RsModelPeriod {
    fn from(s: ModelPeriod) -> RsModelPeriod {
        RsModelPeriod {
            start: s.start,
            end: s.end,
        }
    }
}

#[pymethods]
impl ModelPeriod {
    #[new]
    fn new(start: NaiveDate, end: NaiveDate) -> Self {
        ModelPeriod { start, end }
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
struct CatchmentType {
    one_catchment: Option<CatchmentData>,
    more_catchments: Option<Vec<CatchmentData>>,
}

impl From<CatchmentType> for RsCatchmentType {
    fn from(s: CatchmentType) -> RsCatchmentType {
        if let Some(c) = s.more_catchments {
            RsCatchmentType::SubCatchments(c.into_iter().map(Into::<RsCatchmentData>::into).collect())
        } else {
            let data = s.one_catchment.expect("Missing CatchmentData");
            RsCatchmentType::OneCatchment(Into::<RsCatchmentData>::into(data))
        }
    }
}

impl TryFrom<PyObject> for CatchmentType {
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
                return Ok::<CatchmentType, PyErr>(CatchmentType {
                    one_catchment: Some(value.extract::<CatchmentData>(py)?),
                    more_catchments: None,
                });
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
                return Ok::<CatchmentType, PyErr>(CatchmentType {
                    one_catchment: None,
                    more_catchments: Some(value.extract::<Vec<CatchmentData>>(py)?),
                });
            }
            Err(PyValueError::new_err(
                "The catchment argument must be an instance of one CatchmentData or a list of CatchmentData classes",
            ))
        })
    }
}

#[pymethods]
impl CatchmentType {
    fn __repr__(&self) -> PyResult<String> {
        if let Some(c) = &self.one_catchment {
            return Ok(c.__repr__().unwrap());
        } else if let Some(c) = &self.more_catchments {
            let mut str: String = "[".to_string();
            for data in c.iter() {
                str += &data.__repr__().unwrap();
                str += ",";
            }
            str.pop();
            str += "]";
            return Ok(str);
        }
        Err(PyValueError::new_err("Cannot find any catchment data"))
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
    #[pyo3(get)]
    catchment: PyObject, // keep this to allow user access
    rs_catchment: CatchmentType,
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
        let cat_data = catchment.clone();
        Ok(GR6JModelInputs {
            time,
            precipitation,
            evapotranspiration,
            catchment,
            rs_catchment: CatchmentType::try_from(cat_data)?,
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
        let run_period: RsModelPeriod = inputs.run_period.clone().into();
        let inputs = RsGR6JModelInputs {
            time: inputs.time,
            precipitation: inputs.precipitation,
            evapotranspiration: inputs.evapotranspiration,
            catchment: inputs.rs_catchment.into(),
            run_period: inputs.run_period.into(),
            warmup_period: inputs.warmup_period.map(|i| i.into()),
            destination: inputs.destination,
            observed_runoff: inputs.observed_runoff,
            run_off_unit: inputs.run_off_unit.unwrap_or_default().into(),
        };
        let model = GR6JModel {
            run_period,
            rs_model: RsGR6JModel::new(inputs).map_err(|e| PyValueError::new_err(e.to_string()))?,
        };
        println!("{:?}", model.rs_model);
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
        let results = self.rs_model.run().map_err(|e| PyValueError::new_err(e.to_string()))?;

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
        })
    }
}

#[pymodule]
fn gr6j(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StoreLevels>()?;
    m.add_class::<CatchmentData>()?;
    m.add_class::<ModelPeriod>()?;
    m.add_class::<RunOffUnit>()?;
    m.add_class::<GR6JModelInputs>()?;
    m.add_class::<GR6JModel>()?;

    Ok(())
}
