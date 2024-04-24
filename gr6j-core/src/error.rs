use chrono::NaiveDate;
use csv::Error;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelPeriodError {
    #[error("The end {0} date must be smaller than the start date {1}")]
    DateTooSmall(NaiveDate, NaiveDate),
}

#[derive(Error, Debug)]
pub enum LoadModelError {
    #[error("The {0} must be larger than its minimum threshold ({1})")]
    ParameterTooSmall(String, f64),
    #[error("The {0} must be smaller than its maximum threshold ({1})")]
    ParameterTooLarge(String, f64),
    #[error("The lower bound ({0}) for '{1}' must be larger than its upper bound ({2})")]
    ParameterBounds(f64, String, f64),
    #[error("The lower bound ({0}) for '{1}' must be larger than the parameter minimum threshold ({2})")]
    ParameterTooSmallLowerBound(f64, String, f64),
    #[error("The upper bound ({0}) for '{1}' must be smaller than the parameter maximum threshold ({2})")]
    ParameterTooLargeUpperBound(f64, String, f64),
    #[error("The time and {0} vectors must have the same length")]
    MismatchedLength(String),
    #[error("{0}")]
    LoadModel(String),
    #[error("The time vector must have continuous dates")]
    NotContinuousDates(),
    #[error("The {0} must be larger or equal to the {1} in the time vector")]
    DateOutsideTVector(String, String),
    #[error("The {0} date must be smaller than the run start date")]
    DateTooSmall(String),
    #[error("The warm-up period (end date: {0}) is not directly before the model run period (start date: {1})")]
    TooFarWarmUpPeriod(String, String),
    #[error("The destination folder {0} does not exist")]
    DestinationNotFound(String),
    #[error(
        "The {0} series contains at least one NA value at the following indices: {1:?}. Missing values are not allowed"
    )]
    NanData(String, Vec<String>),
    #[error("{0}")]
    Generic(String),
}

#[derive(Error, Debug)]
pub enum RunModelError {
    #[error("The destination folder {0} cannot be created")]
    DestinationNotWritable(String),
    #[error("The run-off conversion factor must be larger than 0")]
    WrongConversion(),
    #[error("The simulation end date was reached and the model cannot advanced anymore")]
    ReachedSimulationEnd(),
    #[error("The simulation metrics cannot be calculated because {0}")]
    CannotCalculateMetrics(String),
    #[error("A CSV file cannot be exported because {0}")]
    CannotExportCsv(String),
    #[error("The {0} chart file cannot be generated because {1}")]
    CannotGenerateChart(String, String),
    #[error("Cannot load the calibration model #{0} because: {1}")]
    CalibrationError(usize, String),
}

impl From<csv::Error> for RunModelError {
    fn from(value: Error) -> Self {
        RunModelError::CannotExportCsv(value.to_string())
    }
}

impl From<io::Error> for RunModelError {
    fn from(value: io::Error) -> Self {
        RunModelError::CannotExportCsv(value.to_string())
    }
}
