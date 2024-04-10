extern crate gr6j;

use chrono::NaiveDate;
use gr6j::inputs::{CatchmentData, CatchmentType, GR6JModelInputs, ModelPeriod, RunOffUnit};
use gr6j::model::GR6JModel;
use gr6j::parameter::Parameter;
use log::LevelFilter;
use std::fs::File;
use std::path::Path;

/// Use two hydrological units or sub-catchments to model two different responses from the same
/// catchment.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable logging
    env_logger::builder().filter_level(LevelFilter::Info).init();

    // Collect hydrological data
    let file = File::open(r"gr6j-core\src\test_data\airGR_L0123001_dataset.csv")?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut time: Vec<NaiveDate> = vec![];
    let mut precipitation: Vec<f64> = vec![];
    let mut evapotranspiration: Vec<f64> = vec![];
    let mut observed: Vec<f64> = vec![];
    for result in rdr.records() {
        let record = result.unwrap();
        let t = NaiveDate::parse_from_str(record.get(0).unwrap(), "%d/%m/%Y")?;
        time.push(t);
        precipitation.push(record.get(1).unwrap().parse::<f64>()?);
        evapotranspiration.push(record.get(2).unwrap().parse::<f64>()?);
        let obs = record.get(3).unwrap();
        let obs = if obs == "NA" { "0.0" } else { obs };
        observed.push(obs.parse::<f64>()?);
    }

    // Configure the model
    let start = NaiveDate::from_ymd_opt(1984, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(1998, 12, 31).unwrap();

    let inputs = GR6JModelInputs {
        time,
        precipitation: precipitation.clone(),
        evapotranspiration: evapotranspiration.clone(),
        catchment: CatchmentType::SubCatchments(vec![
            CatchmentData {
                area: 2.0,
                x1: Parameter::X1(31.0),
                x2: Parameter::X2(3.47),
                x3: Parameter::X3(32.0),
                x4: Parameter::X4(2.1),
                x5: Parameter::X5(0.55),
                x6: Parameter::X6(5.3),
                store_levels: None,
            },
            CatchmentData {
                area: 0.4,
                x1: Parameter::X1(1000.0),
                x2: Parameter::X2(1.00),
                x3: Parameter::X3(3.0),
                x4: Parameter::X4(1.2),
                x5: Parameter::X5(3.0),
                x6: Parameter::X6(1.3),
                store_levels: None,
            },
        ]),
        run_period: ModelPeriod::new(start, end)?,
        warmup_period: None,
        destination: Some(Path::new(r"gr6j-core\examples\results").to_path_buf()),
        observed_runoff: Some(observed),
        run_off_unit: RunOffUnit::NoConversion,
    };
    let mut model = GR6JModel::new(inputs)?;
    model.run()?;
    Ok(())
}
