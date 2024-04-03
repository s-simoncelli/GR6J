extern crate gr6j;

use chrono::NaiveDate;
use gr6j::inputs::{CatchmentData, CatchmentType, GR6JModelInputs, ModelPeriod};
use gr6j::model::GR6JModel;
use gr6j::parameter::Parameter;
use log::LevelFilter;
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable logging
    env_logger::builder().filter_level(LevelFilter::Info).init();

    // Collect hydrological data
    let file = File::open(r"src\test_data\airGR_L0123001_dataset.csv")?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut time: Vec<NaiveDate> = vec![];
    let mut precipitation: Vec<f64> = vec![];
    let mut evapotranspiration: Vec<f64> = vec![];
    for result in rdr.records() {
        let record = result.unwrap();
        let t = NaiveDate::parse_from_str(record.get(0).unwrap(), "%d/%m/%Y")?;
        time.push(t);
        precipitation.push(record.get(1).unwrap().parse::<f64>()?);
        evapotranspiration.push(record.get(2).unwrap().parse::<f64>()?);
    }

    // Configure the model
    let start = NaiveDate::from_ymd_opt(1984, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(1998, 12, 31).unwrap();

    let inputs = GR6JModelInputs {
        time,
        precipitation: precipitation.clone(),
        evapotranspiration: evapotranspiration.clone(),
        catchment: CatchmentType::OneCatchment(CatchmentData {
            area: 2.0,
            x1: Parameter::X1(150.0),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(500.0),
            x4: Parameter::X4(5.2),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(10.0),
            store_levels: None,
        }),
        run_period: ModelPeriod { start, end },
        warmup_period: None,
        destination: Some(Path::new(r"./examples/results").to_path_buf()),
    };
    let mut model = GR6JModel::new(inputs)?;
    model.run()?;
    Ok(())
}
