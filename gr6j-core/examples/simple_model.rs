extern crate gr6j;

use chrono::NaiveDate;
use gr6j::inputs::{CatchmentData, CatchmentType, GR6JModelInputs, ModelPeriod, RunOffUnit};
use gr6j::model::GR6JModel;
use gr6j::parameter::{Parameter, X1, X2, X3, X4, X5, X6};
use gr6j::utils::example::load_data;
use log::LevelFilter;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable logging
    env_logger::builder().filter_level(LevelFilter::Info).init();

    // Collect hydrological data.
    let data = load_data()?;

    // Configure the model
    let start = NaiveDate::from_ymd_opt(1994, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(1998, 12, 31).unwrap();

    let inputs = GR6JModelInputs {
        time: &data.time,
        precipitation: &data.precipitation,
        evapotranspiration: &data.evapotranspiration,
        catchment: CatchmentType::OneCatchment(CatchmentData {
            area: 1.0,
            x1: X1::new(31.0)?,
            x2: X2::new(3.47)?,
            x3: X3::new(32.0)?,
            x4: X4::new(2.1)?,
            x5: X5::new(0.55)?,
            x6: X6::new(5.3)?,
            store_levels: None,
        }),
        run_period: ModelPeriod::new(start, end)?,
        warmup_period: None,
        destination: Some(Path::new(r"./gr6j-core/examples/results").to_path_buf()),
        observed_runoff: Some(&data.observed_runoff),
        run_off_unit: RunOffUnit::NoConversion,
    };
    let mut model = GR6JModel::new(inputs)?;
    model.run()?;
    Ok(())
}
