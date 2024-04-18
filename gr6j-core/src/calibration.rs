use crate::error::{LoadModelError, RunModelError};
use crate::inputs::{
    CalibrationCatchmentData, CalibrationCatchmentType, CalibrationInputs, CatchmentData, CatchmentType,
    GR6JModelInputs,
};
use crate::metric::CalibrationMetric;
use crate::model::GR6JModel;
use crate::outputs::CalibrationOutputs;
use crate::parameter::{Parameter, X1, X2, X3, X4, X5, X6};
use chrono::{Local, NaiveDate};
use csv::Writer;
use egobox_doe::{Lhs, LhsKind, SamplingMethod};
use log::{debug, info};
use ndarray::{arr2, s, Array2};
use rayon::prelude::*;
use std::fs::create_dir;
use std::mem;
use std::path::PathBuf;

/// Perform the model calibration to pick the best calibration parameters using comparison charts
/// for the flow and flow duration curves and calibration metrics (such as Nash-Sutcliffe).
///
/// The calibration steps are as follows:
///   1) Generate [`CalibrationInputs::sample_size`] samples using the Latin Hyper-cube sampling technique.
///   2) Each sub-sample will contain a random combination of the model parameters based on the
///     ranges given in [`CalibrationCatchmentData`].
///   3) The toll will run a total of [`CalibrationInputs::sample_size`] models in parallel and generate a set of
///      calibration metrics.
///   4) Check the metrics (looking ot the metric desired values), the simulated vs. observed flow
///      and flow duration charts
///   5) Refine the calibration by reducing the parameter ranges
///   6) Pick the best calibration parameter set.
pub struct Calibration<'a> {
    /// The vector with model inputs to run a [`GR6JModel`].
    run_inputs: Option<Vec<GR6JModelInputs<'a>>>,
    /// The destination where to save the charts and diagnostic data.
    destination: PathBuf,
}

/// The data collected by the parallel loop from each GR6J models.
struct ParData {
    /// The data of all hydrological units.
    catchment: CatchmentType,
    /// The simulated run-off.
    run_off: Vec<f64>,
    /// The metrics to use to assess the model performance.
    metrics: CalibrationMetric,
}

impl<'a> Calibration<'a> {
    /// Initialise the GR6J models to run for the calibration. This will initialise the inputs of
    /// [`crate::inputs::CalibrationInputs::sample_size`] GR6J models with a different combination
    /// of parameters.
    /// # Arguments
    ///
    /// * `inputs`: The calibration input data.
    ///
    /// returns: `Result<Calibration, LoadModelError>`
    pub fn new(inputs: CalibrationInputs<'a>) -> Result<Self, LoadModelError> {
        if !inputs.destination.exists() {
            return Err(LoadModelError::DestinationNotFound(
                inputs.destination.to_str().unwrap().to_string(),
            ));
        }

        let destination = inputs
            .destination
            .join(Local::now().format("calibration_%Y%m%d_%H%M").to_string());
        let sample_size: usize = inputs.sample_size.unwrap_or(200);

        let mut run_inputs: Vec<GR6JModelInputs> = vec![];

        // Collect the model inputs
        match inputs.catchment {
            CalibrationCatchmentType::OneCatchment(data) => {
                debug!("Collecting input data for one-catchment models");
                let all_samples = Self::sample(&data, sample_size);

                for sample_idx in 0..all_samples.nrows() {
                    let sample = all_samples.slice(s![sample_idx, ..]);
                    run_inputs.push(GR6JModelInputs {
                        time: inputs.time,
                        precipitation: inputs.precipitation,
                        evapotranspiration: inputs.evapotranspiration,
                        catchment: CatchmentType::OneCatchment(CatchmentData {
                            area: data.area,
                            x1: X1::new(sample[0])?,
                            x2: X2::new(sample[1])?,
                            x3: X3::new(sample[2])?,
                            x4: X4::new(sample[3])?,
                            x5: X5::new(sample[4])?,
                            x6: X6::new(sample[5])?,
                            store_levels: None,
                        }),
                        run_period: inputs.calibration_period,
                        warmup_period: None,
                        destination: None,
                        observed_runoff: Some(inputs.observed_runoff),
                        run_off_unit: inputs.run_off_unit.clone(),
                    });
                }
            }
            CalibrationCatchmentType::SubCatchments(data_vec) => {
                debug!("Collecting input data for multiple-catchment models");
                // Generate the samples
                let all_samples: Vec<Array2<f64>> =
                    data_vec.iter().map(|data| Self::sample(data, sample_size)).collect();

                for sample_idx in 0..all_samples[0].nrows() {
                    // Collect the data for all catchments
                    let mut cal_data_vec: Vec<CatchmentData> = vec![];
                    for (uh_idx, data) in data_vec.iter().enumerate() {
                        let sample = all_samples[uh_idx].slice(s![sample_idx, ..]);
                        cal_data_vec.push(CatchmentData {
                            area: data.area,
                            x1: X1::new(sample[0])?,
                            x2: X2::new(sample[1])?,
                            x3: X3::new(sample[2])?,
                            x4: X4::new(sample[3])?,
                            x5: X5::new(sample[4])?,
                            x6: X6::new(sample[5])?,
                            store_levels: None,
                        });
                    }

                    // Add inputs
                    run_inputs.push(GR6JModelInputs {
                        time: inputs.time,
                        precipitation: inputs.precipitation,
                        evapotranspiration: inputs.evapotranspiration,
                        catchment: CatchmentType::SubCatchments(cal_data_vec),
                        run_period: inputs.calibration_period,
                        warmup_period: None,
                        destination: None,
                        observed_runoff: Some(inputs.observed_runoff),
                        run_off_unit: inputs.run_off_unit.clone(),
                    });
                }
            }
        };

        info!("Created {:?} models", run_inputs.len());

        Ok(Self {
            run_inputs: Some(run_inputs),
            destination,
        })
    }

    /// Run the calibration. This will run the GR6J models using threads; the parallel loop will
    /// stop if [`GR6JModel`] throws an error.
    ///
    /// returns: `Result<CalibrationOutputs, RunModelError>`
    pub fn run(&mut self) -> Result<CalibrationOutputs, RunModelError> {
        let run_inputs = self.run_inputs.take().unwrap();
        let time: Vec<NaiveDate> = run_inputs[0].time.to_vec();

        let par_data: Result<Vec<_>, _> = run_inputs
            .into_par_iter()
            .enumerate()
            .map(|(model, model_inputs)| {
                info!("Running model #{}", model + 1);
                let data = model_inputs.catchment.clone();

                let mut model =
                    GR6JModel::new(model_inputs).map_err(|e| RunModelError::CalibrationError(model, e.to_string()))?;
                let results = model.run()?;
                Ok::<ParData, RunModelError>(ParData {
                    catchment: data,
                    run_off: results.run_off,
                    metrics: results.metrics.unwrap(),
                })
            })
            .collect();

        // Create the destination folder
        if !self.destination.exists() {
            create_dir(&self.destination)
                .map_err(|_| RunModelError::DestinationNotWritable(self.destination.to_str().unwrap().to_string()))?;
        }

        let metric_dest = self.destination.join("Metrics.csv");
        let metric_dest_string = metric_dest.to_str().unwrap().to_string();
        let mut metric_wtr = Writer::from_path(metric_dest)?;

        let mut par_data = par_data?;

        // Parameter writers
        let parameter_header = ["Simulation", "X1", "X2", "X3", "X4", "X5", "X6"];
        let first_catchment_data = par_data.first().expect("Cannot find any results").catchment.clone();
        let mut param_files: Vec<String> = vec![];
        let mut params_wtrs = match first_catchment_data {
            CatchmentType::OneCatchment(_) => {
                let destination = self.destination.join("Parameters.csv");
                param_files.push(destination.to_str().unwrap().to_string());
                let mut wtr = Writer::from_path(&destination)?;
                wtr.write_record(parameter_header)?;
                wtr.flush()?;
                vec![wtr]
            }
            CatchmentType::SubCatchments(p) => {
                let mut writers = vec![];
                for ci in 0..p.len() {
                    let destination = self.destination.join(format!("Parameters_HU{}.csv", ci + 1));
                    param_files.push(destination.to_str().unwrap().to_string());
                    let mut wtr = Writer::from_path(&destination)?;
                    wtr.write_record(parameter_header)?;
                    wtr.flush()?;
                    writers.push(wtr);
                }
                writers
            }
        };

        let mut write_headers = true;
        for (sim_id, results) in par_data.iter().enumerate() {
            // Export metrics
            let metrics = &results.metrics;
            if write_headers {
                metrics.append_header_to_csv(&mut metric_wtr, Some("Simulation".to_string()))?;
            }
            metrics.append_row_to_csv(&mut metric_wtr, Some(format!("#{}", sim_id + 1)))?;
            write_headers = false;

            // Export parameters
            match &results.catchment {
                CatchmentType::OneCatchment(data) => {
                    let wtr = &mut params_wtrs[0];
                    wtr.write_record([
                        format!("#{}", sim_id),
                        data.x1.value().to_string(),
                        data.x2.value().to_string(),
                        data.x3.value().to_string(),
                        data.x4.value().to_string(),
                        data.x5.value().to_string(),
                        data.x6.value().to_string(),
                    ])?;
                    wtr.flush()?;
                }
                CatchmentType::SubCatchments(data) => {
                    for (uh_id, sub_data) in data.iter().enumerate() {
                        let wtr = &mut params_wtrs[uh_id];
                        wtr.write_record([
                            format!("#{}", sim_id),
                            sub_data.x1.value().to_string(),
                            sub_data.x2.value().to_string(),
                            sub_data.x3.value().to_string(),
                            sub_data.x4.value().to_string(),
                            sub_data.x5.value().to_string(),
                            sub_data.x6.value().to_string(),
                        ])?;
                        wtr.flush()?;
                    }
                }
            }
        }

        info!("Exported metric file as '{}'", metric_dest_string);
        for file_name in param_files.iter() {
            info!("Exported parameter file as '{}'", file_name);
        }

        // TODO charts

        let run_off = par_data.iter_mut().map(|d| mem::take(d.run_off.as_mut())).collect();
        let catchment = par_data.iter_mut().map(|d| d.catchment.clone()).collect();
        let metrics = par_data.iter_mut().map(|d| d.metrics.clone()).collect();

        Ok(CalibrationOutputs {
            time,
            run_off,
            catchment,
            metrics,
        })
    }

    /// Create a sample with combinations of model parameters using the Latin Hypercube sampling.
    ///
    /// # Arguments
    ///
    /// * `data`: The data for one catchment.
    /// * `sample_size`: The sam[le size.
    ///
    /// returns: `Array2<f64>`
    /// ```
    fn sample(data: &CalibrationCatchmentData, sample_size: usize) -> Array2<f64> {
        debug!("Generating {} samples", sample_size);
        let limits = arr2(&[
            [data.x1.lower_bound, data.x1.upper_bound],
            [data.x2.lower_bound, data.x2.upper_bound],
            [data.x3.lower_bound, data.x3.upper_bound],
            [data.x4.lower_bound, data.x4.upper_bound],
            [data.x5.lower_bound, data.x5.upper_bound],
            [data.x6.lower_bound, data.x6.upper_bound],
        ]);
        Lhs::new(&limits).kind(LhsKind::Optimized).sample(sample_size)
    }
}
