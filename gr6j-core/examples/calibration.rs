use chrono::NaiveDate;
use gr6j::calibration::Calibration;
use gr6j::inputs::{CalibrationCatchmentData, CalibrationInputs, ModelPeriod, RunOffUnit};
use gr6j::parameter::{ParameterRange, X1Range, X2Range, X3Range, X4Range, X5Range, X6Range};
use gr6j::utils::example::load_data;
use log::LevelFilter;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable relevant logs
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .filter_module("gr6j::model", LevelFilter::Off)
        .init();

    // Collect the hydrological data
    let data = load_data()?;

    // Configure the model
    let start = NaiveDate::from_ymd_opt(1986, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(1988, 12, 31).unwrap();

    // Set the parameter ranges to generate in the Latin Hyper-cube sampling
    let catchment = vec![CalibrationCatchmentData {
        area: 1.0,
        x1: X1Range::default(),
        x2: X2Range::default(),
        x3: X3Range::default(),
        x4: X4Range::default(),
        x5: X5Range::default(),
        x6: X6Range::default(),
    }];

    let mut destination = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    destination.push(r"examples\results");

    let inputs = CalibrationInputs {
        time: &data.time,
        precipitation: &data.precipitation,
        evapotranspiration: &data.evapotranspiration,
        catchment,
        calibration_period: ModelPeriod { start, end },
        observed_runoff: &data.observed_runoff,
        destination,
        sample_size: Some(50),
        run_off_unit: RunOffUnit::NoConversion,
        generate_comparison_charts: true,
    };

    let mut model = Calibration::new(inputs)?;
    let outputs = model.run()?;

    // get the vector of X1 parameter values generated for the first catchment
    println!("{:?}", outputs.parameters[0].to_vec_x1());

    // get the list of Nash-Sutcliffe coefficients calculated by comparing the simulated and
    // observed flow
    println!("{:?}", outputs.metrics.nash_sutcliffe());

    Ok(())
}
