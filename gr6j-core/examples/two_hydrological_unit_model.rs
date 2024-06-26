extern crate gr6j;

use chrono::NaiveDate;
use gr6j::inputs::{CatchmentData, GR6JModelInputs, ModelPeriod, RunOffUnit};
use gr6j::model::GR6JModel;
use gr6j::parameter::{Parameter, X1, X2, X3, X4, X5, X6};
use gr6j::utils::example::load_data;
use log::LevelFilter;
use std::path::Path;

/// Use two hydrological units or sub-catchments to model two different responses from the same
/// catchment.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable logging
    env_logger::builder().filter_level(LevelFilter::Info).init();

    // Collect hydrological data.
    let data = load_data()?;

    // Configure the model
    let start = NaiveDate::from_ymd_opt(1984, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(1998, 12, 31).unwrap();

    let inputs = GR6JModelInputs {
        time: &data.time,
        precipitation: &data.precipitation,
        evapotranspiration: &data.evapotranspiration,
        catchment: vec![
            CatchmentData {
                area: 2.0,
                x1: X1::new(31.0)?,
                x2: X2::new(3.47)?,
                x3: X3::new(32.0)?,
                x4: X4::new(2.1)?,
                x5: X5::new(0.55)?,
                x6: X6::new(5.3)?,
                store_levels: None,
            },
            CatchmentData {
                area: 0.4,
                x1: X1::new(1000.0)?,
                x2: X2::new(1.0)?,
                x3: X3::new(3.0)?,
                x4: X4::new(1.2)?,
                x5: X5::new(3.0)?,
                x6: X6::new(1.3)?,
                store_levels: None,
            },
        ],
        run_period: ModelPeriod::new(start, end)?,
        warmup_period: None,
        destination: Some(Path::new(r"gr6j-core\examples\results").to_path_buf()),
        observed_runoff: Some(&data.observed_runoff),
        run_off_unit: RunOffUnit::NoConversion,
        logging: None,
    };
    let mut model = GR6JModel::new(inputs)?;
    model.run()?;
    Ok(())
}
