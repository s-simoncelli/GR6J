use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadModelError {
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
    #[error("The destination folder {0} cannot be created")]
    DestinationNotWritable(String),
}

#[derive(Error, Debug)]
pub enum RunModelError {
    #[error("The run-off conversion factor must be larger than 0")]
    WrongConversion(),
    #[error("The simulation end date was reached and the model cannot advanced anymore")]
    ReachedSimulationEnd(),
    #[error("The CSV file '{0}' cannot be exported because {1}")]
    CannotExportCsv(String, String),
    #[error("The {0} chart file cannot be generated because {1}")]
    CannotGenerateChart(String, String),
}
