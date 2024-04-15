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
use std::fs::create_dir;
use std::path::{Path, PathBuf};

/// Perform the model calibration to pick the best calibration parameters using comparison charts
/// for the flow and flow duration curves and calibration metrics (such as Nash-Sutcliffe). For
/// a list of the metrics that are calculated see [`gr6j::metric::CalibrationMetricType`].
///
/// The calibration steps are as follows:
///   1) Generate [`CalibrationInputs::sample_size`] samples using the Latin hypercube sampling technique.
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
    /// The size of the sample
    sample_size: usize,
}

impl<'a> Calibration<'a> {
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
            sample_size,
        })
    }

    /// Run the calibration. Error data checking is performed by [`GR6JModelInputs`] and
    /// [`GR6JModel`].
    ///
    /// returns: Result<CalibrationOutputs, RunModelError>
    pub fn run(&mut self) -> Result<CalibrationOutputs, RunModelError> {
        let run_inputs = self.run_inputs.take().unwrap();
        let mut catchment_data: Vec<CatchmentType> = vec![];
        let mut calibration_metrics: Vec<CalibrationMetric> = vec![];
        let mut run_off: Vec<Vec<f64>> = vec![];
        let time: Vec<NaiveDate> = run_inputs[0].time.to_vec();

        // TODO move this to another function and return run off
        // TODO start threads here
        for (model, model_inputs) in run_inputs.into_iter().enumerate() {
            info!("Running model #{}", model + 1);
            let data = model_inputs.catchment.clone();

            let mut model =
                GR6JModel::new(model_inputs).map_err(|e| RunModelError::CalibrationError(model, e.to_string()))?;
            let results = model.run()?;

            catchment_data.push(data);
            run_off.push(results.run_off);
            calibration_metrics.push(results.metrics.unwrap());
        }

        // Create the destination folder
        if !self.destination.exists() {
            create_dir(&self.destination)
                .map_err(|_| RunModelError::DestinationNotWritable(self.destination.to_str().unwrap().to_string()))?;
        }

        // Export metrics
        // TODO this is wrong! -> structure not ok - metrics must be in columns
        let metric_dest = self.destination.join("Metrics.csv");
        let metric_dest_string = metric_dest.to_str().unwrap().to_string();
        debug!("Exporting metric file {}", metric_dest_string);
        self.write_metric_file(&calibration_metrics, &metric_dest)
            .map_err(|e| RunModelError::CannotExportCsv(metric_dest_string, e.to_string()))?;

        Ok(CalibrationOutputs {
            time,
            run_off,
            parameters: catchment_data,
            metrics: calibration_metrics,
        })

        // TODO export parameters -> write separate files for UH
    }

    /// Write the metric file.
    ///
    /// # Arguments
    ///
    /// * `metrics`: The vector with the metrics for each simulation.
    /// * `destination`: The destination CSV file.
    ///
    /// returns: Result<(), csv::Error>
    fn write_metric_file(&self, metrics: &[CalibrationMetric], destination: &PathBuf) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_path(destination)?;
        let mut write_header = true;

        for (sim_id, metric) in metrics.iter().enumerate() {
            if write_header {
                metric.append_header_to_csv(&mut wtr, Some("Simulation".to_string()))?;
            }
            metric.append_row_to_csv(&mut wtr, Some(format!("#{}", sim_id + 1)))?;
            write_header = false;
        }
        Ok(())
    }

    /// Export a CSV file with the parameter combinations for one catchment.
    fn write_parameter_combinations(samples: Array2<f64>, destination: &Path) {
        todo!()
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
        // We generate five samples using centered Latin Hypercube sampling.
        Lhs::new(&limits).kind(LhsKind::Centered).sample(sample_size)
    }
}
