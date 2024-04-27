use crate::inputs::{ModelPeriod, RunOffUnit};
use crate::parameter::{X1Range, X2Range, X3Range, X4Range, X5Range, X6Range};
use ::gr6j::inputs::CalibrationCatchmentData as RsCalibrationCatchmentData;
use chrono::NaiveDate;
use gr6j::calibration::Calibration as RsCalibration;
use gr6j::inputs::CalibrationInputs as RsCalibrationInputs;
use gr6j::outputs::CalibrationOutputs as RsCalibrationOutputs;
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use pyo3::PyTypeInfo;
use std::path::PathBuf;

#[derive(FromPyObject, Debug, Clone)]
struct PyParameterRange(f64, f64);

impl IntoPy<PyObject> for PyParameterRange {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyTuple::new_bound(py, [self.0, self.1]).into_py(py)
    }
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
/// The data for the catchment or hydrological unit to calibrate.
pub struct CalibrationCatchmentData {
    area: f64,
    x1_range: X1Range,
    x2_range: X2Range,
    x3_range: X3Range,
    x4_range: X4Range,
    x5_range: X5Range,
    x6_range: X6Range,
}

#[pymethods]
impl CalibrationCatchmentData {
    #[new]
    fn new(
        area: f64,
        x1_range: X1Range,
        x2_range: X2Range,
        x3_range: X3Range,
        x4_range: X4Range,
        x5_range: X5Range,
        x6_range: X6Range,
    ) -> PyResult<CalibrationCatchmentData> {
        Ok(Self {
            area,
            x1_range,
            x2_range,
            x3_range,
            x4_range,
            x5_range,
            x6_range,
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "CalibrationCatchmentData(area={},x1_range={},x2_range={},x3_range={},x4_range={},x5_range={},x6_range={})",
            self.area,
            self.x1_range.__repr__().unwrap(),
            self.x2_range.__repr__().unwrap(),
            self.x3_range.__repr__().unwrap(),
            self.x4_range.__repr__().unwrap(),
            self.x5_range.__repr__().unwrap(),
            self.x6_range.__repr__().unwrap(),
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

// NOTE: cannot use enum because pyo3 does not support complex types
#[pyclass]
#[derive(Clone)]
pub struct CalibrationCatchmentDataVec(pub Vec<CalibrationCatchmentData>);

impl TryFrom<PyObject> for CalibrationCatchmentDataVec {
    type Error = PyErr;
    fn try_from(value: PyObject) -> Result<Self, Self::Error> {
        // Convert the catchment data
        Python::with_gil(|py| {
            let module = PyModule::import_bound(py, "builtins")?;
            let isinstance = module.getattr("isinstance")?;
            let is_one: bool = isinstance
                .call1((&value, CalibrationCatchmentData::type_object_bound(py)))?
                .extract()?;

            let is_list: bool = isinstance.call1((&value, PyList::type_object_bound(py)))?.extract()?;

            if is_one {
                return Ok(Self(vec![value.extract::<CalibrationCatchmentData>(py)?]));
            } else if is_list {
                // The list must contain instances of CatchmentData
                let res: Vec<PyObject> = value.extract(py)?;
                for data in res.iter() {
                    let is_catchment: bool = isinstance
                        .call1((data, CalibrationCatchmentData::type_object_bound(py)))?
                        .extract()?;
                    if !is_catchment {
                        return Err(PyValueError::new_err(
                            "The catchment argument must be a list of CalibrationCatchmentData classes",
                        ));
                    }
                }
                return Ok(Self(value.extract::<Vec<CalibrationCatchmentData>>(py)?));
            }
            Err(PyValueError::new_err(
                "The catchment argument must be an instance of one CalibrationCatchmentData or a list of CalibrationCatchmentData classes",
            ))
        })
    }
}

#[pymethods]
impl CalibrationCatchmentDataVec {
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
pub struct CalibrationInputs {
    #[pyo3(get)]
    pub time: Vec<NaiveDate>,
    #[pyo3(get)]
    pub precipitation: Vec<f64>,
    #[pyo3(get)]
    pub evapotranspiration: Vec<f64>,
    #[pyo3(get)]
    pub observed_runoff: Vec<f64>,
    #[pyo3(get)] // keep this to allow user access
    catchment: CalibrationCatchmentDataVec,
    pub rs_catchment: Vec<RsCalibrationCatchmentData>,
    #[pyo3(get)]
    pub calibration_period: ModelPeriod,
    #[pyo3(get)]
    pub destination: PathBuf,
    #[pyo3(get)]
    pub run_off_unit: RunOffUnit,
    #[pyo3(get)]
    pub sample_size: Option<usize>,
    #[pyo3(get)]
    pub generate_comparison_charts: Option<bool>,
}

#[pymethods]
impl CalibrationInputs {
    #[new]
    #[pyo3(signature = (time,precipitation,evapotranspiration,observed_runoff,catchment,calibration_period,destination,run_off_unit,sample_size=None,generate_comparison_charts=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        time: Vec<NaiveDate>,
        precipitation: Vec<f64>,
        evapotranspiration: Vec<f64>,
        observed_runoff: Vec<f64>,
        catchment: PyObject,
        calibration_period: ModelPeriod,
        destination: PathBuf,
        run_off_unit: RunOffUnit,
        sample_size: Option<usize>,
        generate_comparison_charts: Option<bool>,
    ) -> PyResult<Self> {
        let catchment = CalibrationCatchmentDataVec::try_from(catchment)?;
        let rs_catchment: Vec<RsCalibrationCatchmentData> = catchment
            .0
            .iter()
            .map(|d| RsCalibrationCatchmentData {
                area: d.area,
                x1: d.x1_range.0.clone(),
                x2: d.x2_range.0.clone(),
                x3: d.x3_range.0.clone(),
                x4: d.x4_range.0.clone(),
                x5: d.x5_range.0.clone(),
                x6: d.x6_range.0.clone(),
            })
            .collect();

        Ok(Self {
            time,
            precipitation,
            evapotranspiration,
            observed_runoff,
            catchment,
            rs_catchment,
            calibration_period,
            destination,
            run_off_unit,
            sample_size,
            generate_comparison_charts,
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "CalibrationInputs(calibration_period={},catchment={},destination={},run_off_unit={})",
            self.calibration_period.__repr__().unwrap(),
            self.catchment.__repr__().unwrap(),
            self.destination.to_str().unwrap(),
            self.run_off_unit.__repr__()
        ))
    }

    fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
pub struct Calibration(RsCalibrationOutputs);

#[pymethods]
impl Calibration {
    #[new]
    pub fn new(inputs: CalibrationInputs) -> PyResult<Self> {
        let inputs = RsCalibrationInputs {
            time: &inputs.time,
            precipitation: &inputs.precipitation,
            evapotranspiration: &inputs.evapotranspiration,
            observed_runoff: &inputs.observed_runoff,
            catchment: inputs.rs_catchment,
            calibration_period: inputs.calibration_period.0,
            destination: inputs.destination,
            run_off_unit: inputs.run_off_unit.into(),
            sample_size: inputs.sample_size,
            generate_comparison_charts: inputs.generate_comparison_charts.unwrap_or(true),
        };
        let mut rs_calibration = RsCalibration::new(inputs).map_err(|e| PyValueError::new_err(e.to_string()))?;

        // release the GIL to prevent deadlocks when model runs with threads
        let outputs = Python::with_gil(|py| {
            py.allow_threads(move || rs_calibration.run().map_err(|e| PyValueError::new_err(e.to_string())))
        })?;
        Ok(Calibration(outputs))
    }

    #[getter]
    pub fn time(&self) -> Vec<NaiveDate> {
        self.0.time.clone()
    }

    #[getter]
    pub fn run_off(&self) -> Vec<Vec<f64>> {
        self.0.run_off.clone()
    }

    pub fn x1_vec(&self, catchment_index: usize) -> PyResult<Vec<f64>> {
        match self.0.parameters.get(catchment_index) {
            None => Err(PyIndexError::new_err("Out of bounds")),
            Some(v) => Ok(v.to_vec_x1()),
        }
    }
    pub fn x2_vec(&self, catchment_index: usize) -> PyResult<Vec<f64>> {
        match self.0.parameters.get(catchment_index) {
            None => Err(PyIndexError::new_err("Out of bounds")),
            Some(v) => Ok(v.to_vec_x2()),
        }
    }
    pub fn x3_vec(&self, catchment_index: usize) -> PyResult<Vec<f64>> {
        match self.0.parameters.get(catchment_index) {
            None => Err(PyIndexError::new_err("Out of bounds")),
            Some(v) => Ok(v.to_vec_x3()),
        }
    }
    pub fn x4_vec(&self, catchment_index: usize) -> PyResult<Vec<f64>> {
        match self.0.parameters.get(catchment_index) {
            None => Err(PyIndexError::new_err("Out of bounds")),
            Some(v) => Ok(v.to_vec_x4()),
        }
    }
    pub fn x5_vec(&self, catchment_index: usize) -> PyResult<Vec<f64>> {
        match self.0.parameters.get(catchment_index) {
            None => Err(PyIndexError::new_err("Out of bounds")),
            Some(v) => Ok(v.to_vec_x5()),
        }
    }
    pub fn x6_vec(&self, catchment_index: usize) -> PyResult<Vec<f64>> {
        match self.0.parameters.get(catchment_index) {
            None => Err(PyIndexError::new_err("Out of bounds")),
            Some(v) => Ok(v.to_vec_x6()),
        }
    }

    #[getter]
    pub fn nash_sutcliffe(&self) -> Vec<f64> {
        self.0.metrics.nash_sutcliffe()
    }
    #[getter]
    pub fn log_nash_sutcliffe(&self) -> Vec<f64> {
        self.0.metrics.log_nash_sutcliffe()
    }
    #[getter]
    pub fn non_parametric_kling_gupta(&self) -> Vec<f64> {
        self.0.metrics.non_parametric_kling_gupta()
    }
    #[getter]
    pub fn rmse(&self) -> Vec<f64> {
        self.0.metrics.rmse()
    }
    #[getter]
    pub fn volume_error(&self) -> Vec<f64> {
        self.0.metrics.volume_error()
    }
}
