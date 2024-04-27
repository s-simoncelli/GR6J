use gr6j::parameter::{
    Parameter, ParameterRange, X1Range as RsX1Range, X2Range as RsX2Range, X3Range as RsX3Range, X4Range as RsX4Range,
    X5Range as RsX5Range, X6Range as RsX6Range, X1 as RsX1, X2 as RsX2, X3 as RsX3, X4 as RsX4, X5 as RsX5, X6 as RsX6,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct X1(pub RsX1);

#[pymethods]
impl X1 {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let p = RsX1::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(*p))
    }
    #[staticmethod]
    pub fn max() -> PyResult<f64> {
        Ok(RsX1::max_value())
    }
    #[staticmethod]
    pub fn min() -> PyResult<f64> {
        Ok(RsX1::min_value())
    }
    #[staticmethod]
    pub fn unit() -> PyResult<String> {
        Ok(RsX1::unit().to_string())
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.value().to_string())
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct X2(pub RsX2);

#[pymethods]
impl X2 {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let p = RsX2::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(*p))
    }
    #[staticmethod]
    pub fn max() -> PyResult<f64> {
        Ok(RsX2::max_value())
    }
    #[staticmethod]
    pub fn min() -> PyResult<f64> {
        Ok(RsX2::min_value())
    }
    #[staticmethod]
    pub fn unit() -> PyResult<String> {
        Ok(RsX2::unit().to_string())
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.value().to_string())
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct X3(pub RsX3);

#[pymethods]
impl X3 {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let p = RsX3::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(*p))
    }
    #[staticmethod]
    pub fn max() -> PyResult<f64> {
        Ok(RsX3::max_value())
    }
    #[staticmethod]
    pub fn min() -> PyResult<f64> {
        Ok(RsX3::min_value())
    }
    #[staticmethod]
    pub fn unit() -> PyResult<String> {
        Ok(RsX3::unit().to_string())
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.value().to_string())
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct X4(pub RsX4);

#[pymethods]
impl X4 {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let p = RsX4::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(*p))
    }
    #[staticmethod]
    pub fn max() -> PyResult<f64> {
        Ok(RsX4::max_value())
    }
    #[staticmethod]
    pub fn min() -> PyResult<f64> {
        Ok(RsX4::min_value())
    }
    #[staticmethod]
    pub fn unit() -> PyResult<String> {
        Ok(RsX4::unit().to_string())
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.value().to_string())
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct X5(pub RsX5);

#[pymethods]
impl X5 {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let p = RsX5::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(*p))
    }
    #[staticmethod]
    pub fn max() -> PyResult<f64> {
        Ok(RsX5::max_value())
    }
    #[staticmethod]
    pub fn min() -> PyResult<f64> {
        Ok(RsX5::min_value())
    }
    #[staticmethod]
    pub fn unit() -> PyResult<String> {
        Ok(RsX5::unit().to_string())
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.value().to_string())
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct X6(pub RsX6);

#[pymethods]
impl X6 {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let p = RsX6::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self(*p))
    }
    #[staticmethod]
    pub fn max() -> PyResult<f64> {
        Ok(RsX6::max_value())
    }
    #[staticmethod]
    pub fn min() -> PyResult<f64> {
        Ok(RsX6::min_value())
    }
    #[staticmethod]
    pub fn unit() -> PyResult<String> {
        Ok(RsX6::unit().to_string())
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.0.value().to_string())
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct X1Range(pub Box<RsX1Range>);

#[pymethods]
impl X1Range {
    #[new]
    fn new(lower_bound: Option<f64>, upper_bound: Option<f64>) -> PyResult<Self> {
        Ok(Self(
            RsX1Range::new(
                lower_bound.unwrap_or(RsX1::min_value()),
                upper_bound.unwrap_or(RsX1::max_value()),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
    #[getter]
    pub fn lower_bound(&self) -> f64 {
        self.0.lower_bound
    }
    #[getter]
    pub fn upper_bound(&self) -> f64 {
        self.0.upper_bound
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("({},{})", self.0.lower_bound, self.0.upper_bound))
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct X2Range(pub Box<RsX2Range>);

#[pymethods]
impl X2Range {
    #[new]
    fn new(lower_bound: Option<f64>, upper_bound: Option<f64>) -> PyResult<Self> {
        Ok(Self(
            RsX2Range::new(
                lower_bound.unwrap_or(RsX2::min_value()),
                upper_bound.unwrap_or(RsX2::max_value()),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
    #[getter]
    pub fn lower_bound(&self) -> f64 {
        self.0.lower_bound
    }
    #[getter]
    pub fn upper_bound(&self) -> f64 {
        self.0.upper_bound
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("({},{})", self.0.lower_bound, self.0.upper_bound))
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct X3Range(pub Box<RsX3Range>);

#[pymethods]
impl X3Range {
    #[new]
    fn new(lower_bound: Option<f64>, upper_bound: Option<f64>) -> PyResult<Self> {
        Ok(Self(
            RsX3Range::new(
                lower_bound.unwrap_or(RsX3::min_value()),
                upper_bound.unwrap_or(RsX3::max_value()),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
    #[getter]
    pub fn lower_bound(&self) -> f64 {
        self.0.lower_bound
    }
    #[getter]
    pub fn upper_bound(&self) -> f64 {
        self.0.upper_bound
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("({},{})", self.0.lower_bound, self.0.upper_bound))
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct X4Range(pub Box<RsX4Range>);

#[pymethods]
impl X4Range {
    #[new]
    fn new(lower_bound: Option<f64>, upper_bound: Option<f64>) -> PyResult<Self> {
        Ok(Self(
            RsX4Range::new(
                lower_bound.unwrap_or(RsX4::min_value()),
                upper_bound.unwrap_or(RsX4::max_value()),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
    #[getter]
    pub fn lower_bound(&self) -> f64 {
        self.0.lower_bound
    }
    #[getter]
    pub fn upper_bound(&self) -> f64 {
        self.0.upper_bound
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("({},{})", self.0.lower_bound, self.0.upper_bound))
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct X5Range(pub Box<RsX5Range>);

#[pymethods]
impl X5Range {
    #[new]
    fn new(lower_bound: Option<f64>, upper_bound: Option<f64>) -> PyResult<Self> {
        Ok(Self(
            RsX5Range::new(
                lower_bound.unwrap_or(RsX5::min_value()),
                upper_bound.unwrap_or(RsX5::max_value()),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
    #[getter]
    pub fn lower_bound(&self) -> f64 {
        self.0.lower_bound
    }
    #[getter]
    pub fn upper_bound(&self) -> f64 {
        self.0.upper_bound
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("({},{})", self.0.lower_bound, self.0.upper_bound))
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct X6Range(pub Box<RsX6Range>);

#[pymethods]
impl X6Range {
    #[new]
    fn new(lower_bound: Option<f64>, upper_bound: Option<f64>) -> PyResult<Self> {
        Ok(Self(
            RsX6Range::new(
                lower_bound.unwrap_or(RsX6::min_value()),
                upper_bound.unwrap_or(RsX6::max_value()),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
    #[getter]
    pub fn lower_bound(&self) -> f64 {
        self.0.lower_bound
    }
    #[getter]
    pub fn upper_bound(&self) -> f64 {
        self.0.upper_bound
    }
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("({},{})", self.0.lower_bound, self.0.upper_bound))
    }
    pub fn __str__(&self) -> String {
        self.__repr__().unwrap()
    }
}
