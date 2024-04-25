use ::gr6j::error::ModelPeriodError;
use ::gr6j::inputs::StoreLevels as RsStoreLevels;
use ::gr6j::inputs::{CatchmentData as RsCatchmentData, ModelPeriod as RsModelPeriod, RunOffUnit as RsRunOffUnit};
use chrono::NaiveDate;
use gr6j::parameter::{Parameter, X1, X2, X3, X4, X5, X6};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::PyTypeInfo;
use std::path::PathBuf;

#[pyclass(get_all)]
#[derive(Debug, Clone, Copy)]
pub struct StoreLevels {
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
    pub fn new(production_store: f64, routing_store: f64, exponential_store: f64) -> Self {
        StoreLevels {
            production_store,
            routing_store,
            exponential_store,
        }
    }

    pub fn __repr__(&self) -> PyResult<String> {
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
pub struct CatchmentData {
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
    pub rs_catchment: RsCatchmentData,
}

#[pymethods]
#[allow(clippy::too_many_arguments)]
impl CatchmentData {
    #[new]
    pub fn new(
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

    pub fn __repr__(&self) -> PyResult<String> {
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

    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct ModelPeriod {
    #[pyo3(get)]
    start: NaiveDate,
    #[pyo3(get)]
    end: NaiveDate,
    pub rs_period: RsModelPeriod,
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
    pub fn new(start: NaiveDate, end: NaiveDate) -> PyResult<Self> {
        let rs_period = RsModelPeriod::new(start, end).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(ModelPeriod { start, end, rs_period })
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ModelPeriod(start={},end={})", self.start, self.end,))
    }

    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone, Debug, Default)]
pub enum RunOffUnit {
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

impl RunOffUnit {
    pub fn __repr__(&self) -> &str {
        self.__pyo3__repr__()
    }
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
pub struct CatchmentDataVec(pub Vec<CatchmentData>);

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
    pub fn __repr__(&self) -> PyResult<String> {
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

    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct GR6JModelInputs {
    #[pyo3(get)]
    pub time: Vec<NaiveDate>,
    #[pyo3(get)]
    pub precipitation: Vec<f64>,
    #[pyo3(get)]
    pub evapotranspiration: Vec<f64>,
    #[pyo3(get)] // keep this to allow user access
    catchment: CatchmentDataVec,
    pub rs_catchment: Vec<RsCatchmentData>,
    #[pyo3(get)]
    pub run_period: ModelPeriod,
    #[pyo3(get)]
    pub warmup_period: Option<ModelPeriod>,
    #[pyo3(get)]
    pub destination: Option<PathBuf>,
    #[pyo3(get)]
    pub observed_runoff: Option<Vec<f64>>,
    #[pyo3(get)]
    pub run_off_unit: Option<RunOffUnit>,
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
            Some(u) => u.__repr__(),
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
