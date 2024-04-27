use crate::chart::{save_flow_comparison_chart, save_metric_vs_parameter_chart};
use crate::error::{LoadModelError, RunModelError};
use crate::inputs::{CalibrationCatchmentData, CalibrationInputs, CatchmentData, GR6JModelInputs, RunOffUnit};
use crate::metric::CalibrationMetric;
use crate::model::GR6JModel;
use crate::outputs::{
    CalibrationMetricVector, CalibrationOutputs, CalibrationParameterValueVector, CalibrationParameterValues,
};
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
    run_inputs: Vec<GR6JModelInputs<'a>>,
    /// The destination where to save the charts and diagnostic data.
    destination: PathBuf,
    /// The flow unit
    run_off_unit: RunOffUnit,
    /// Whether to export the comparison of the observed and simulated run-off time series and
    /// flow duration curves for each model.
    generate_comparison_charts: bool,
}

/// The data collected by the parallel loop from each GR6J models.
struct ParData {
    /// The time vector
    time: Vec<NaiveDate>,
    /// The data of all hydrological units.
    catchment: Vec<CatchmentData>,
    /// The simulated run-off.
    run_off: Vec<f64>,
    /// The metrics to use to assess the model performance.
    metrics: CalibrationMetric,
    /// The observed run-off
    observed: Option<Vec<f64>>,
}

const PARAMETER_HEADER: [&str; 7] = ["Simulation", "X1", "X2", "X3", "X4", "X5", "X6"];

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
        info!("Generating {} parameter sub-samples with Latin-Hypercube", sample_size);

        // Generate the samples for each sub-catchment model
        let all_samples: Vec<Array2<f64>> = inputs
            .catchment
            .iter()
            .map(|data| Self::sample(data, sample_size, None))
            .collect();

        for sample_idx in 0..all_samples[0].nrows() {
            // Collect the data for all catchments
            let mut catchment: Vec<CatchmentData> = vec![];
            for (uh_idx, data) in inputs.catchment.iter().enumerate() {
                let sample = all_samples[uh_idx].slice(s![sample_idx, ..]);
                catchment.push(CatchmentData {
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
                catchment,
                run_period: inputs.calibration_period,
                warmup_period: None,
                destination: None,
                observed_runoff: Some(inputs.observed_runoff),
                run_off_unit: inputs.run_off_unit.clone(),
            });
        }

        info!("Created {:?} models", run_inputs.len());

        Ok(Self {
            run_inputs,
            run_off_unit: inputs.run_off_unit,
            destination,
            generate_comparison_charts: inputs.generate_comparison_charts,
        })
    }

    /// Run the calibration. This will run the GR6J models using threads; the parallel loop will
    /// stop if [`GR6JModel`] throws an error.
    ///
    /// returns: `Result<CalibrationOutputs, RunModelError>`
    pub fn run(&mut self) -> Result<CalibrationOutputs, RunModelError> {
        let run_inputs = mem::take(&mut self.run_inputs);

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
                    time: results.time,
                    catchment: data.to_vec(),
                    run_off: results.run_off,
                    metrics: results.metrics.unwrap(),
                    observed: model.observed,
                })
            })
            .collect();

        // Create the destination folder
        if !self.destination.exists() {
            create_dir(&self.destination)
                .map_err(|_| RunModelError::DestinationNotWritable(self.destination.to_str().unwrap().to_string()))?;
        }

        let mut par_data = par_data?;
        let observed = par_data[0].observed.clone().unwrap();
        let time: Vec<NaiveDate> = par_data[0].time.to_vec();

        // Group the catchment data by sub-catchment
        let first_catchment_data = par_data.first().expect("Cannot find any results").catchment.clone();
        let total_uh = first_catchment_data.len();

        let mut parameters_by_uh: Vec<CalibrationParameterValueVector> = vec![];
        for uh_id in 0..total_uh {
            let mut sub_uh_data: Vec<CalibrationParameterValues> = vec![];

            // write header to csv writer
            let file_name = match total_uh {
                1 => self.destination.join("Parameters.csv"),
                _ => self.destination.join(format!("Parameters_HU{}.csv", uh_id + 1)),
            };
            let mut wtr = Writer::from_path(&file_name)?;
            wtr.write_record(PARAMETER_HEADER)?;
            wtr.flush()?;

            // collect and write CSV lines
            for (sim_id, c) in par_data.iter().enumerate() {
                let data: CalibrationParameterValues = c.catchment[uh_id].clone().into();
                wtr.write_record([
                    format!("#{}", sim_id + 1),
                    data.x1.to_string(),
                    data.x2.to_string(),
                    data.x3.to_string(),
                    data.x4.to_string(),
                    data.x5.to_string(),
                    data.x6.to_string(),
                ])?;
                wtr.flush()?;

                sub_uh_data.push(data);
            }
            info!(
                "Exported parameter file as '{}'",
                file_name.to_str().unwrap().to_string()
            );

            parameters_by_uh.push(CalibrationParameterValueVector(sub_uh_data));
        }

        // Export metrics
        let metric_dest = self.destination.join("Metrics.csv");
        let metric_dest_string = metric_dest.to_str().unwrap().to_string();
        let mut metric_wtr = Writer::from_path(metric_dest)?;

        let mut write_headers = true;
        for (sim_id, results) in par_data.iter().enumerate() {
            let metrics = &results.metrics;
            if write_headers {
                metrics.append_header_to_csv(&mut metric_wtr, Some("Simulation".to_string()))?;
            }
            metrics.append_row_to_csv(&mut metric_wtr, Some(format!("#{}", sim_id + 1)))?;
            write_headers = false;
        }
        info!("Exported metric file as '{}'", metric_dest_string);

        let run_off: Vec<Vec<f64>> = par_data.iter_mut().map(|d| mem::take(d.run_off.as_mut())).collect();
        let metrics = CalibrationMetricVector(par_data.iter_mut().map(|d| d.metrics.clone()).collect());

        // Generate the parameter vs metric charts
        for (hu_id, parameters) in parameters_by_uh.iter().enumerate() {
            let file_prefix = match parameters_by_uh.len() {
                1 => "".to_string(),
                _ => format!("Sub-catchment{}_", hu_id + 1),
            };
            for (p_id, parameter_values) in parameters.to_vec().iter().enumerate() {
                let dest = self
                    .destination
                    .join(format!("{}X{}_vs_metrics.png", file_prefix, p_id + 1));
                let title = format!("{}Parameter X{}", file_prefix.replace('_', " / "), p_id + 1);
                save_metric_vs_parameter_chart(parameter_values, &metrics, title, &dest).map_err(|e| {
                    RunModelError::CannotGenerateChart(dest.to_str().unwrap().to_string(), e.to_string())
                })?;
                match parameters_by_uh.len() {
                    1 => info!("Saved chart for parameter X{}", p_id + 1),
                    _ => info!("Saved chart for sub-catchment {} - parameter X{}", hu_id + 1, p_id + 1),
                };
            }
        }

        // Generate the comparison charts for the simulated vs. observed flow and FDC
        if self.generate_comparison_charts {
            (0..par_data.len()).into_par_iter().try_for_each(|model_id| {
                info!("Generating run-off chart for model #{}", model_id + 1);
                let dest = self.destination.join(format!("Flows_model{}.png", model_id + 1));

                save_flow_comparison_chart(
                    &time,
                    &run_off[model_id],
                    &observed,
                    format!("Simulated vs. observed - Model #{}", model_id + 1),
                    &dest,
                    &self.run_off_unit,
                )
                .map_err(|e| RunModelError::CannotGenerateChart(dest.to_str().unwrap().to_string(), e.to_string()))?;

                Ok::<(), RunModelError>(())
            })?;
        }

        Ok(CalibrationOutputs {
            time,
            run_off,
            parameters: parameters_by_uh,
            metrics,
        })
    }

    /// Create a sample with combinations of model parameters using the Latin Hypercube sampling.
    ///
    /// # Arguments
    ///
    /// * `data`: The data for one catchment.
    /// * `sample_size`: The sample size.
    /// * `method`: The method to generate the random data.
    ///
    /// returns: `Array2<f64>`
    /// ```
    fn sample(data: &CalibrationCatchmentData, sample_size: usize, method: Option<LhsKind>) -> Array2<f64> {
        debug!("Generating {} samples", sample_size);
        let limits = arr2(&[
            [data.x1.lower_bound, data.x1.upper_bound],
            [data.x2.lower_bound, data.x2.upper_bound],
            [data.x3.lower_bound, data.x3.upper_bound],
            [data.x4.lower_bound, data.x4.upper_bound],
            [data.x5.lower_bound, data.x5.upper_bound],
            [data.x6.lower_bound, data.x6.upper_bound],
        ]);
        Lhs::new(&limits)
            .kind(method.unwrap_or(LhsKind::Classic))
            .sample(sample_size)
    }
}
