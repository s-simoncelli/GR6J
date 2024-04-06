use crate::chart::{generate_fdc_chart, generate_summary_chart, FDCData};
use crate::error::{LoadModelError, RunModelError};
use crate::inputs::{CatchmentType, GR6JModelInputs, ModelPeriod, RunOffUnit, StoreLevels};
use crate::outputs::{GR6JOutputs, ModelStepData, ModelStepDataVector};
use crate::parameter::Parameter;
use crate::unit_hydrograph::{UnitHydrograph, UnitHydrographInputs, UnitHydrographType};
use crate::utils::calculate_fdc;
use chrono::{Local, NaiveDate, TimeDelta};
use csv::Writer;
use log::{debug, info, warn};
use std::fmt::Debug;
use std::fs::create_dir;
use std::path::{Path, PathBuf};

/// Internal state variables
#[derive(Debug)]
struct InternalState {
    // The current time step index
    step: usize,
    // The store levels
    store_levels: StoreLevels,
    /// The first unit hydrograph
    unit_hydrograph1: UnitHydrograph,
    /// The second unit hydrograph
    unit_hydrograph2: UnitHydrograph,
}

/// The struct containing the state and GR6J parameters for one model.
#[derive(Debug)]
struct ModelData {
    /// The catchment os sub-catchment area (km2).
    area: f64,
    /// Parameter X1
    x1: Parameter,
    /// Parameter X2
    x2: Parameter,
    /// Parameter X3
    x3: Parameter,
    /// Parameter X4
    x4: Parameter,
    /// Parameter X5
    x5: Parameter,
    /// Parameter X6
    x6: Parameter,
    /// The current internal state of the model
    state: InternalState,
}

/// The GR6J model
#[derive(Debug)]
pub struct GR6JModel {
    /// Vector of time.
    pub time: Vec<NaiveDate>,
    /// Input vector of total precipitation (mm/day)
    pub precipitation: Vec<f64>,
    /// input vector of potential evapotranspiration (PE) (mm/day)
    pub evapotranspiration: Vec<f64>,
    /// The data of each model
    models: Vec<ModelData>,
    /// The first day of the simulation (without the warm-up period).
    collect_data_from: NaiveDate,
    /// The path where to save the files
    destination: Option<PathBuf>,
    /// The observed run=off time-series.
    pub observed: Option<Vec<f64>>,
    /// Conversion to apply to the run-off data.
    pub run_off_unit: RunOffUnit,
}

impl GR6JModel {
    /// Check the value of a parameter is within the allowed bound.
    ///
    /// # Arguments
    ///
    /// * `value`: The parameter value.
    /// * `param_type`: The parameter type.
    fn check_parameter_range(parameter: Parameter) -> Result<(), LoadModelError> {
        // check min
        if parameter.value() < parameter.min_threshold() {
            return Err(LoadModelError::LoadModel(format!(
                "The {} must be larger than its minimum threshold ({})",
                parameter,
                parameter.min_threshold(),
            )));
        }
        // check max
        if parameter.value() > parameter.max_threshold() {
            return Err(LoadModelError::LoadModel(format!(
                "The {} must be smaller than its maximum threshold ({})",
                parameter,
                parameter.max_threshold(),
            )));
        }
        Ok(())
    }

    /// Create a new instance(s) of the GR6J model(s). More instances are created if more than
    /// one hydrological unit is provided.
    ///
    /// # Arguments
    ///
    /// * `inputs`: The `GR6JModelInputs` struct containing the model input data.
    ///
    /// returns: Vec<GR6JModel>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn new(inputs: GR6JModelInputs) -> Result<Self, LoadModelError> {
        // Check hydrological data
        if inputs.time.len() != inputs.precipitation.len() {
            return Err(LoadModelError::MismatchedLength("precipitation".to_string()));
        }
        if inputs.time.len() != inputs.evapotranspiration.len() {
            return Err(LoadModelError::MismatchedLength("evapotranspiration".to_string()));
        }
        if let Some(observed) = &inputs.observed_runoff {
            if inputs.time.len() != observed.len() {
                return Err(LoadModelError::MismatchedLength("observed run-off".to_string()));
            }
        }

        // Check time
        if inputs
            .time
            .windows(2)
            .map(|ts| (ts[1] - ts[0]).num_days())
            .max()
            .unwrap()
            != 1
        {
            return Err(LoadModelError::NotContinuousDates());
        }
        if inputs.run_period.start < *inputs.time.first().unwrap() {
            return Err(LoadModelError::DateOutsideTVector(
                "run start date".to_string(),
                "first date".to_string(),
            ));
        }
        if inputs.run_period.end > *inputs.time.last().unwrap() {
            return Err(LoadModelError::DateOutsideTVector(
                "run end date".to_string(),
                "last date".to_string(),
            ));
        }

        // initialise the warm-up period
        let warmup_period = match inputs.warmup_period {
            None => {
                let warmup_end = inputs.run_period.start - TimeDelta::try_days(1).unwrap();
                let mut warmup_start = warmup_end - TimeDelta::try_days(364).unwrap();
                // check leap year
                if warmup_end + TimeDelta::try_days(1).unwrap() != inputs.run_period.start {
                    warmup_start -= TimeDelta::try_days(1).unwrap();
                }
                warn!(
                    "Model warm-up period not defined. Using default period {}-{}",
                    warmup_start, warmup_end
                );

                if warmup_start >= inputs.time[0] {
                    // one year is available
                    Some(ModelPeriod {
                        start: warmup_start,
                        end: warmup_end,
                    })
                } else if inputs.run_period.start > inputs.time[0] {
                    // reduced warm-up period
                    warn!(
                        "The input data is too short to define a one-year warm-up period. Period \
                        will start from {} which is the first date in the time vector",
                        inputs.time[0]
                    );
                    Some(ModelPeriod {
                        start: inputs.time[0],
                        end: warmup_end,
                    })
                } else {
                    // disregard warm-up period if there is no enough data
                    warn!("The input data is too short to define a warm-up period");
                    None
                }
            }
            Some(period) => {
                // check date validity
                if period.start >= inputs.run_period.start {
                    return Err(LoadModelError::DateTooSmall("warm-up start".to_string()));
                }
                if period.end > inputs.run_period.start {
                    return Err(LoadModelError::DateTooSmall("warm-up end".to_string()));
                }
                if (inputs.run_period.start - period.end).num_days() != 1 {
                    return Err(LoadModelError::TooFarWarmUpPeriod(
                        period.end.to_string(),
                        inputs.run_period.start.to_string(),
                    ));
                }
                Some(period)
            }
        };
        if warmup_period.is_some() {
            info!("Model warm-up period set to: {:?}", warmup_period.as_ref().unwrap());
        }
        info!(
            "Model run period set to {}-{}",
            inputs.run_period.start, inputs.run_period.end
        );

        // create the destination folder
        let destination: Option<PathBuf> = if let Some(dest) = inputs.destination {
            if !dest.exists() {
                return Err(LoadModelError::DestinationNotFound(dest.to_str().unwrap().to_string()));
            }
            let destination = dest.join(Local::now().format("%Y%m%d_%H%M").to_string());
            if !destination.exists() {
                create_dir(&destination)
                    .map_err(|_| LoadModelError::DestinationNotWritable(destination.to_str().unwrap().to_string()))?;
            }
            Some(destination)
        } else {
            None
        };

        // truncate the data based on the warm-up and run periods
        let start_date = match warmup_period {
            None => inputs.run_period.start,
            Some(p) => p.start,
        };
        let start_index = inputs.time.iter().position(|&r| r == start_date).unwrap();
        let end_index = inputs.time.iter().position(|&r| r == inputs.run_period.end).unwrap();

        let time = inputs.time[start_index..end_index].to_owned();
        let precipitation = inputs.precipitation[start_index..end_index].to_owned();
        let evapotranspiration = inputs.evapotranspiration[start_index..end_index].to_owned();
        let observed = inputs.observed_runoff.map(|q| q[start_index..end_index].to_owned());

        // Convert data for one catchment
        let catchments_data = match inputs.catchment {
            CatchmentType::OneCatchment(data) => vec![data],
            CatchmentType::SubCatchments(data_vec) => data_vec,
        };

        let mut models: Vec<ModelData> = vec![];
        for catchment_data in catchments_data.iter() {
            // check parameters
            Self::check_parameter_range(catchment_data.x1)?;
            Self::check_parameter_range(catchment_data.x2)?;
            Self::check_parameter_range(catchment_data.x3)?;
            Self::check_parameter_range(catchment_data.x4)?;
            Self::check_parameter_range(catchment_data.x5)?;
            Self::check_parameter_range(catchment_data.x6)?;

            // initialise the reservoir levels
            let mut int_store_levels = catchment_data.store_levels.unwrap_or_default();

            // scale the levels
            int_store_levels = StoreLevels {
                production_store: int_store_levels.production_store * catchment_data.x1.value(),
                routing_store: int_store_levels.routing_store * catchment_data.x3.value(),
                exponential_store: int_store_levels.exponential_store,
            };

            // initialise the unit hydrographs
            let unit_hydrograph1 = UnitHydrograph::new(UnitHydrographInputs {
                uh_type: UnitHydrographType::T1,
                time_constant: catchment_data.x4.value(),
                exponent: 2.5,
            });
            let unit_hydrograph2 = UnitHydrograph::new(UnitHydrographInputs {
                uh_type: UnitHydrographType::T2,
                time_constant: catchment_data.x4.value(),
                exponent: 2.5,
            });

            let internal_state = InternalState {
                step: 0,
                store_levels: int_store_levels,
                unit_hydrograph1,
                unit_hydrograph2,
            };

            models.push(ModelData {
                area: catchment_data.area,
                x1: catchment_data.x1,
                x2: catchment_data.x2,
                x3: catchment_data.x3,
                x4: catchment_data.x4,
                x5: catchment_data.x5,
                x6: catchment_data.x6,
                state: internal_state,
            })
        }
        Ok(GR6JModel {
            time,
            precipitation,
            evapotranspiration,
            collect_data_from: inputs.run_period.start,
            models,
            destination,
            observed,
            run_off_unit: inputs.run_off_unit,
        })
    }

    pub fn run(&mut self) -> Result<GR6JOutputs, RunModelError> {
        let mut catchment_outputs: Vec<ModelStepDataVector> = vec![];
        for model_index in 0..self.models.len() {
            let mut outputs: Vec<ModelStepData> = vec![];
            debug!("Started run for hydrological unit {model_index}");

            catchment_outputs.push({
                loop {
                    let out = self.step(model_index);
                    if out.is_ok() {
                        let step_data = out.unwrap();
                        if step_data.time < self.collect_data_from {
                            continue;
                        }
                        outputs.push(step_data);
                    } else {
                        break;
                    }
                }
                ModelStepDataVector(outputs)
            });
        }

        info!("Simulation is completed :)");

        let time = catchment_outputs[0].time();

        // get the run off for each hydrological unit and scale it by area to get the volume
        debug!("Collecting run-off data");
        let mut run_offs: Vec<Vec<f64>> = vec![];
        for (model_index, data) in catchment_outputs.iter().enumerate() {
            run_offs.push(data.run_off(Some(self.models[model_index].area)));
        }

        let conv_factor = self.run_off_unit.conv_factor();
        if conv_factor <= 0.0 {
            return Err(RunModelError::WrongConversion());
        }

        // get the combined run off components for all hydrological units
        let mut total_run_off: Vec<f64> = vec![];
        for step_index in 0..run_offs[0].len() {
            let mut q = 0.0;
            for q_t in run_offs.iter() {
                // convert from mm*km2/day to m3/day
                q += q_t[step_index] * conv_factor;
            }
            total_run_off.push(q);
        }

        let fdc = calculate_fdc(&total_run_off);
        let results = GR6JOutputs {
            catchment_outputs,
            time,
            run_off: total_run_off,
        };

        if let Some(destination) = &self.destination {
            // Export run-off CSV file
            let runoff_dest = destination.join("Run-off.csv");
            self.write_run_off_file(
                results.time.as_ref(),
                results.run_off.as_ref(),
                self.run_off_unit.unit_label(),
                &runoff_dest,
            )
            .map_err(|e| RunModelError::CannotExportCsv(runoff_dest.to_str().unwrap().to_string(), e.to_string()))?;
            debug!("Exported run-off CSV file");

            // Export parameters
            match results.catchment_outputs.len() {
                1 => {
                    let dest = destination.join("Parameters.csv");
                    self.write_parameter_file(&self.models[0], &dest).map_err(|e| {
                        RunModelError::CannotExportCsv(dest.to_str().unwrap().to_string(), e.to_string())
                    })?;
                    debug!(
                        "Exported parameter CSV files to '{}'",
                        dest.to_str().unwrap().to_string()
                    );
                }
                _ => {
                    for (uh, model) in self.models.iter().enumerate() {
                        let dest = destination.join(format!("Parameters_HU{}.csv", uh + 1));
                        self.write_parameter_file(model, &dest).map_err(|e| {
                            RunModelError::CannotExportCsv(dest.to_str().unwrap().to_string(), e.to_string())
                        })?;
                        debug!(
                            "Exported parameter CSV files to '{}'",
                            dest.to_str().unwrap().to_string()
                        );
                    }
                }
            }

            // Export FDC
            let fdc_dest = destination.join("FDC.csv");
            self.write_fdc_file(&fdc.0, &fdc.1, self.run_off_unit.unit_label(), &fdc_dest)
                .map_err(|e| {
                    RunModelError::CannotExportCsv(runoff_dest.to_str().unwrap().to_string(), e.to_string())
                })?;
            debug!("Exported FDC CSV file");

            // Generate charts
            generate_summary_chart(self, &results, destination)
                .map_err(|e| RunModelError::CannotGenerateChart("summary".to_string(), e.to_string()))?;

            let sim_fdc = FDCData {
                exceedence: fdc.0,
                run_off: fdc.1,
            };
            let obs_fdc = match &self.observed {
                None => None,
                Some(q) => {
                    let fdc = calculate_fdc(q);
                    Some(FDCData {
                        exceedence: fdc.0,
                        run_off: fdc.1,
                    })
                }
            };
            generate_fdc_chart(self, sim_fdc, obs_fdc, destination)
                .map_err(|e| RunModelError::CannotGenerateChart("fdc".to_string(), e.to_string()))?;
        }

        Ok(results)
    }

    /// Advance time for one model.
    ///
    /// # Arguments
    ///
    /// * `model_index`: The index model to step.
    ///
    /// returns: Result<ModelStepData, RunModelError>
    pub fn step(&mut self, model_index: usize) -> Result<ModelStepData, RunModelError> {
        let b = 0.9;
        let c = 0.4;
        let x1 = self.models[model_index].x1.value();
        let x3 = self.models[model_index].x3.value();
        let x6 = self.models[model_index].x6.value();

        let step = self.models[model_index].state.step;
        if step == self.precipitation.len() {
            return Err(RunModelError::ReachedSimulationEnd());
        }

        debug!("Running step #{} - {}", step, self.time[step]);

        let p = self.precipitation[step];
        let e = self.evapotranspiration[step];
        let storage_ratio = self.models[model_index].state.store_levels.production_store / x1;

        // update production store level
        let mut net_p = 0.0;
        let mut pr = 0.0;
        let mut storage_p = 0.0;
        let mut actual_e = 0.0;
        if p < e {
            let net_e = e - p;
            let scaled_e = (net_e / x1).min(13.0);
            let exp_scaled_e = (2.0 * scaled_e).exp();

            let tws = (exp_scaled_e - 1.0) / (exp_scaled_e + 1.0);
            let storage_e = self.models[model_index].state.store_levels.production_store * (2.0 - storage_ratio) * tws
                / (1.0 + (1.0 - storage_ratio) * tws);

            actual_e = storage_e + p;
            self.models[model_index].state.store_levels.production_store -= storage_e;
        } else {
            actual_e = e;
            net_p = p - e;
            let scaled_p = (net_p / x1).min(13.0);
            let exp_scaled_p = (2.0 * scaled_p).exp();

            let tws = (exp_scaled_p - 1.0) / (exp_scaled_p + 1.0);
            storage_p = x1 * (1.0 - storage_ratio.powi(2)) * tws / (1.0 + storage_ratio * tws);
            pr = net_p - storage_p;
            self.models[model_index].state.store_levels.production_store += storage_p;
        }

        if self.models[model_index].state.store_levels.production_store < 0.0 {
            self.models[model_index].state.store_levels.production_store = 0.0;
        }

        // update percolation in production store
        let percolation = self.models[model_index].state.store_levels.production_store
            * (1.
                - (1. + (self.models[model_index].state.store_levels.production_store / (9. / 4. * x1)).powi(4))
                    .powf(-0.25));
        self.models[model_index].state.store_levels.production_store -= percolation;
        pr += percolation;

        // split the effective rainfall into the two routing components and generate the two new hydrographs
        let precipitation_uh1 = pr * b;
        let precipitation_uh2 = pr * (1.0 - b);

        // Combine the two hydrographs
        let x4 = self.models[model_index].x4.value() as i32;
        self.models[model_index]
            .state
            .unit_hydrograph1
            .convolution(x4, precipitation_uh1);
        self.models[model_index]
            .state
            .unit_hydrograph2
            .convolution(x4, precipitation_uh2);

        // potential inter-catchment semi-exchange
        let exchange = self.models[model_index].x2.value()
            * (self.models[model_index].state.store_levels.routing_store / x3 - self.models[model_index].x5.value());

        // routing store
        let new_routing_store = self.models[model_index].state.store_levels.routing_store
            + (1.0 - c) * self.models[model_index].state.unit_hydrograph1.values[0]
            + exchange;
        let exchange_from_routing_store = {
            if new_routing_store < 0.0 {
                -(new_routing_store - exchange)
            } else {
                exchange
            }
        };
        self.models[model_index].state.store_levels.routing_store = new_routing_store;
        if self.models[model_index].state.store_levels.routing_store < 0.0 {
            self.models[model_index].state.store_levels.routing_store = 0.0;
        }

        let scaled_routing_store = (self.models[model_index].state.store_levels.routing_store / x3).powi(4);
        let routing_store_outflow = self.models[model_index].state.store_levels.routing_store
            * (1. - 1. / f64::sqrt(f64::sqrt(1. + scaled_routing_store)));
        self.models[model_index].state.store_levels.routing_store -= routing_store_outflow;

        // exponential store
        self.models[model_index].state.store_levels.exponential_store +=
            c * self.models[model_index].state.unit_hydrograph1.values[0] + exchange;
        let scaled_exp_store = self.models[model_index].state.store_levels.exponential_store / x6.max(-33.0).min(33.0);
        let exponential_store_outflow = {
            if scaled_exp_store > 7.0 {
                self.models[model_index].state.store_levels.exponential_store + x6 / scaled_exp_store.exp()
            } else if scaled_exp_store < -7.0 {
                x6 * scaled_exp_store.exp()
            } else {
                x6 * (scaled_exp_store.exp() + 1.0).ln()
            }
        };
        self.models[model_index].state.store_levels.exponential_store -= exponential_store_outflow;

        // run-off from outflow from UH2 branch after exchange
        let exchange_from_direct_branch = {
            if self.models[model_index].state.unit_hydrograph2.values[0] + exchange < 0.0 {
                -self.models[model_index].state.unit_hydrograph2.values[0]
            } else {
                exchange
            }
        };
        let outflow_from_uh2_branch = (self.models[model_index].state.unit_hydrograph2.values[0] + exchange).max(0.0);

        // total run-off
        let run_off = routing_store_outflow + outflow_from_uh2_branch + exponential_store_outflow;

        // update the step index
        self.models[model_index].state.step += 1;

        Ok(ModelStepData {
            time: self.time[self.models[model_index].state.step - 1],
            evapotranspiration: e,
            precipitation: p,
            net_rainfall: net_p,
            store_levels: self.models[model_index].state.store_levels,
            storage_p,
            actual_evapotranspiration: actual_e,
            percolation,
            pr,
            exchange,
            exchange_from_routing_store,
            exchange_from_direct_branch,
            actual_exchange: exchange_from_routing_store + exchange_from_direct_branch + exchange,
            routing_store_outflow,
            exponential_store_outflow,
            outflow_from_uh2_branch,
            run_off,
        })
    }

    /// Export the run-off data to a CSV file.
    ///
    /// # Arguments
    ///
    /// * `time`: The vector with the date.
    /// * `total_run_off`: The vector with the run-off values.
    /// * `run_off_unit`: The run-off unit of measurement.
    /// * `destination`: The path to the CSV file.
    ///
    /// returns: Result<(), csv::Error>
    fn write_run_off_file(
        &self,
        time: &[NaiveDate],
        total_run_off: &[f64],
        run_off_unit: &str,
        destination: &Path,
    ) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_path(destination)?;
        wtr.write_record(["Date", format!("Run-off ({})", run_off_unit).as_str()])?;

        for (step_index, q) in total_run_off.iter().enumerate() {
            wtr.write_record(&[time[step_index].to_string(), q.to_string()])?;
            wtr.flush()?;
        }
        Ok(())
    }

    /// Export the FDC to a CSV file.
    ///
    /// # Arguments
    ///
    /// * `percentiles`: The vector of percentiles.
    /// * `flow_rate`: The flow rate.
    /// * `run_off_unit`: The run-off unit of measurement.
    /// * `destination`: The path to the CSV file.
    ///
    /// returns: Result<(), csv::Error>
    fn write_fdc_file(
        &self,
        percentiles: &[f64],
        flow_rate: &[f64],
        run_off_unit: &str,
        destination: &Path,
    ) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_path(destination)?;
        wtr.write_record(["Percentage exceedance", format!("Run-off ({})", run_off_unit).as_str()])?;

        for (pct, q) in percentiles.iter().zip(flow_rate) {
            wtr.write_record([pct.to_string(), q.to_string()])?;
            wtr.flush()?;
        }
        Ok(())
    }

    /// Export the list of parameters for one hydrological unit.
    ///
    /// # Arguments
    ///
    /// * `data`: The model data.
    /// * `destination`: The path to the CSV file.
    ///
    /// returns: Result<(), csv::Error>
    fn write_parameter_file(&self, data: &ModelData, destination: &Path) -> Result<(), csv::Error> {
        let mut wtr = Writer::from_path(destination)?;
        wtr.write_record(["Parameter", "Value", "Unit", "Description"])?;

        wtr.write_record(["Area", data.area.to_string().as_str(), "Km2", "Catchment area"])?;
        wtr.write_record([
            "X1",
            data.x1.value().to_string().as_str(),
            data.x1.unit(),
            data.x1.description(),
        ])?;
        wtr.write_record([
            "X2",
            data.x2.value().to_string().as_str(),
            data.x2.unit(),
            data.x2.description(),
        ])?;
        wtr.write_record([
            "X3",
            data.x3.value().to_string().as_str(),
            data.x3.unit(),
            data.x3.description(),
        ])?;
        wtr.write_record([
            "X4",
            data.x4.value().to_string().as_str(),
            data.x4.unit(),
            data.x4.description(),
        ])?;
        wtr.write_record([
            "X5",
            data.x5.value().to_string().as_str(),
            data.x5.unit(),
            data.x5.description(),
        ])?;
        wtr.write_record([
            "X6",
            data.x6.value().to_string().as_str(),
            data.x6.unit(),
            data.x6.description(),
        ])?;
        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::inputs::{CatchmentData, RunOffUnit, StoreLevels};
    use crate::model::{CatchmentType, GR6JModel, GR6JModelInputs, ModelPeriod, Parameter};
    use crate::outputs::{ModelStepData, ModelStepDataVector};
    use crate::utils::assert_approx_array_eq;
    use chrono::{Datelike, NaiveDate, TimeDelta};
    use std::env;
    use std::fs::File;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    /// Parse the result file for a GR6J run from R
    fn parse_r_file(file: &Path) -> ModelStepDataVector {
        let file = File::open(file).expect("Failed to read CSV file");
        let mut data: Vec<ModelStepData> = vec![];

        for result in csv::Reader::from_reader(file).records() {
            let record = result.unwrap();
            data.push(ModelStepData {
                time: NaiveDate::from_str(record.get(0).unwrap()).unwrap(),
                evapotranspiration: record.get(2).unwrap().parse::<f64>().unwrap(),
                precipitation: record.get(1).unwrap().parse::<f64>().unwrap(),
                net_rainfall: record.get(3).unwrap().parse::<f64>().unwrap(),
                store_levels: StoreLevels {
                    production_store: record.get(11).unwrap().parse::<f64>().unwrap(),
                    routing_store: record.get(12).unwrap().parse::<f64>().unwrap(),
                    exponential_store: record.get(13).unwrap().parse::<f64>().unwrap(),
                },
                storage_p: record.get(4).unwrap().parse::<f64>().unwrap(),
                actual_evapotranspiration: record.get(16).unwrap().parse::<f64>().unwrap(),
                percolation: record.get(6).unwrap().parse::<f64>().unwrap(),
                pr: record.get(5).unwrap().parse::<f64>().unwrap(),
                exchange: record.get(7).unwrap().parse::<f64>().unwrap(),
                exchange_from_routing_store: record.get(8).unwrap().parse::<f64>().unwrap(),
                exchange_from_direct_branch: record.get(9).unwrap().parse::<f64>().unwrap(),
                actual_exchange: record.get(15).unwrap().parse::<f64>().unwrap(),
                routing_store_outflow: record.get(10).unwrap().parse::<f64>().unwrap(),
                exponential_store_outflow: record.get(17).unwrap().parse::<f64>().unwrap(),
                outflow_from_uh2_branch: record.get(14).unwrap().parse::<f64>().unwrap(),
                run_off: record.get(18).unwrap().parse::<f64>().unwrap(),
            });
        }
        ModelStepDataVector(data)
    }

    /// Get the test path
    fn test_path() -> PathBuf {
        Path::new(&env::current_dir().unwrap()).join("src").join("test_data")
    }

    /// Run the model and compare the results against data generate for the airGR R package.
    ///
    /// # Arguments
    ///
    /// * `r_csv_file`: The CSV file name with the R-exported data.
    /// * `start_year`: Start collecting input data when this year is reached.
    /// * `stop_year`: Stop collecting input data when this year is reached.
    /// * `start`: Model start date. Default to first day in the input data.
    /// * `end`: Model end date. Default to last day in the input data.
    /// * `parameters`: The list of model parameters.
    ///
    /// returns: ()
    fn compare_against_r_data(
        r_csv_file: &str,
        start_year: i32,
        stop_year: i32,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        parameters: [Parameter; 6],
    ) {
        let expected_data = parse_r_file(test_path().join(r_csv_file).as_ref());
        let file = File::open(test_path().join("airGR_L0123001_dataset.csv")).expect("Failed to read CSV file");
        let mut rdr = csv::Reader::from_reader(file);

        let mut time: Vec<NaiveDate> = vec![];
        let mut precipitation: Vec<f64> = vec![];
        let mut evapotranspiration: Vec<f64> = vec![];
        for result in rdr.records() {
            let record = result.unwrap();
            let date = NaiveDate::parse_from_str(record.get(0).unwrap(), "%d/%m/%Y").unwrap();

            if date.year() < start_year {
                continue;
            }
            if date.year() > stop_year {
                break;
            }

            time.push(date);
            precipitation.push(record.get(1).unwrap().parse::<f64>().unwrap());
            evapotranspiration.push(record.get(2).unwrap().parse::<f64>().unwrap());
        }

        let start = match start {
            None => *time.first().unwrap(),
            Some(s) => s,
        };
        let end = match end {
            None => *time.last().unwrap(),
            Some(e) => e,
        };

        let catchment_data = CatchmentData {
            area: 1.0,
            x1: parameters[0],
            x2: parameters[1],
            x3: parameters[2],
            x4: parameters[3],
            x5: parameters[4],
            x6: parameters[5],
            store_levels: None,
        };
        let inputs = GR6JModelInputs {
            time,
            precipitation,
            evapotranspiration,
            catchment: CatchmentType::OneCatchment(catchment_data),
            run_period: ModelPeriod { start, end },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };

        let mut model = GR6JModel::new(inputs).unwrap();
        let results = model.run().expect("Cannot fetch results");

        // compare all data
        assert_approx_array_eq(
            results.catchment_outputs[0].production_store().as_ref(),
            &expected_data.production_store(),
        );
        assert_approx_array_eq(
            results.catchment_outputs[0].routing_store().as_ref(),
            &expected_data.routing_store(),
        );
        assert_approx_array_eq(
            results.catchment_outputs[0].exponential_store().as_ref(),
            &expected_data.exponential_store(),
        );
        assert_approx_array_eq(
            results.run_off.as_ref(),
            &expected_data.run_off(Some(catchment_data.area)),
        );
    }

    fn build_t_vector() -> Vec<NaiveDate> {
        let t0 = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let mut t: Vec<NaiveDate> = vec![t0; 366];
        for (d, date) in t.iter_mut().enumerate() {
            *date += TimeDelta::try_days(d as i64).unwrap();
        }
        t
    }

    #[test]
    fn test_too_small_x1() {
        let t = build_t_vector();
        let catchment_data = CatchmentData {
            area: 1.0,
            x1: Parameter::X1(0.0),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(0.4),
            x4: Parameter::X4(0.6),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(0.4),
            store_levels: None,
        };
        let inputs = GR6JModelInputs {
            time: t.clone(),
            precipitation: vec![0.0; t.len()],
            evapotranspiration: vec![0.0; t.len()],
            catchment: CatchmentType::OneCatchment(catchment_data),
            run_period: ModelPeriod {
                start: t[0],
                end: t[365],
            },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };

        let x1 = catchment_data.x1;
        let model = GR6JModel::new(inputs);
        assert_eq!(
            model.unwrap_err().to_string(),
            format!(
                "The {} must be larger than its minimum threshold ({})",
                x1,
                x1.min_threshold()
            )
        )
    }

    #[test]
    fn test_invalid_precipitation_length() {
        let t = build_t_vector();
        let catchment_data = CatchmentData {
            area: 1.0,
            x1: Parameter::X1(0.4),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(0.4),
            x4: Parameter::X4(0.6),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(0.4),
            store_levels: None,
        };
        let inputs = GR6JModelInputs {
            time: t.clone(),
            precipitation: vec![0.0; t.len() - 10],
            evapotranspiration: vec![0.0; t.len()],
            catchment: CatchmentType::OneCatchment(catchment_data),
            run_period: ModelPeriod {
                start: t[0],
                end: t[365],
            },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };

        let model = GR6JModel::new(inputs);
        assert_eq!(
            model.unwrap_err().to_string(),
            "The time and precipitation vectors must have the same length".to_string()
        )
    }

    #[test]
    fn test_invalid_evapotranspiration_length() {
        let t = build_t_vector();
        let catchment_data = CatchmentData {
            area: 1.0,
            x1: Parameter::X1(0.4),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(0.4),
            x4: Parameter::X4(0.6),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(0.4),
            store_levels: None,
        };
        let inputs = GR6JModelInputs {
            time: t.clone(),
            precipitation: vec![0.0; t.len()],
            evapotranspiration: vec![0.0; t.len() - 10],
            catchment: CatchmentType::OneCatchment(catchment_data),
            run_period: ModelPeriod {
                start: t[0],
                end: t[365],
            },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };
        let model = GR6JModel::new(inputs);
        assert_eq!(
            model.unwrap_err().to_string(),
            "The time and evapotranspiration vectors must have the same length".to_string()
        )
    }

    #[test]
    fn test_non_continuous_dates() {
        let mut t = build_t_vector();
        t[365] += TimeDelta::try_days(3).unwrap();
        let catchment_data = CatchmentData {
            area: 1.0,
            x1: Parameter::X1(0.4),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(0.4),
            x4: Parameter::X4(0.6),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(0.4),
            store_levels: None,
        };
        let inputs = GR6JModelInputs {
            time: t.clone(),
            precipitation: vec![0.0; t.len()],
            evapotranspiration: vec![0.0; t.len()],
            catchment: CatchmentType::OneCatchment(catchment_data),
            run_period: ModelPeriod {
                start: t[0],
                end: t[365],
            },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };
        let model = GR6JModel::new(inputs);
        assert_eq!(
            model.unwrap_err().to_string(),
            "The time vector must have continuous dates".to_string()
        )
    }

    #[test]
    fn test_small_start_date() {
        let t = build_t_vector();
        let catchment_data = CatchmentData {
            area: 1.0,
            x1: Parameter::X1(0.4),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(0.4),
            x4: Parameter::X4(0.6),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(0.4),
            store_levels: None,
        };
        let inputs = GR6JModelInputs {
            time: t.clone(),
            precipitation: vec![0.0; t.len()],
            evapotranspiration: vec![0.0; t.len()],
            catchment: CatchmentType::OneCatchment(catchment_data),
            run_period: ModelPeriod {
                start: NaiveDate::from_ymd_opt(1999, 1, 1).unwrap(),
                end: t[365],
            },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };
        let model = GR6JModel::new(inputs);
        assert_eq!(
            model.unwrap_err().to_string(),
            "The run start date must be larger or equal to the first date in the time vector".to_string()
        )
    }

    #[test]
    /// Test simulation with L0123001 dataset from 1994-01-01 to 1998-12-31 w/o warmup period.
    fn test_gr6j_l0123001_no_warm_up() {
        compare_against_r_data(
            "airGR_results_L0123001_no_warmup.csv",
            1984,
            1998,
            Some(NaiveDate::from_ymd_opt(1984, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(1994, 12, 31).unwrap()),
            [
                Parameter::X1(1250.0),
                Parameter::X2(0.3),
                Parameter::X3(500.0),
                Parameter::X4(5.2),
                Parameter::X5(2.0),
                Parameter::X6(10.0),
            ],
        )
    }

    #[test]
    /// Test simulation with L0123001 dataset from 1994-01-01 to 1998-12-31 w warmup period.
    fn test_gr6j_l0123001_sc1() {
        compare_against_r_data(
            "airGR_results_L0123001_sc1.csv",
            1990,
            1998,
            Some(NaiveDate::from_ymd_opt(1994, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(1998, 12, 31).unwrap()),
            [
                Parameter::X1(1250.0),
                Parameter::X2(0.3),
                Parameter::X3(500.0),
                Parameter::X4(5.2),
                Parameter::X5(2.0),
                Parameter::X6(10.0),
            ],
        );
    }

    #[test]
    /// Test simulation with L0123001 dataset from 1994-01-01 to 1998-12-31 w warmup period and
    /// different parameters.
    fn test_gr6j_l0123001_sc2() {
        compare_against_r_data(
            "airGR_results_L0123001_sc2.csv",
            1990,
            1998,
            Some(NaiveDate::from_ymd_opt(1994, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(1998, 12, 31).unwrap()),
            [
                Parameter::X1(1000.0),
                Parameter::X2(0.0),
                Parameter::X3(200.0),
                Parameter::X4(1.0),
                Parameter::X5(0.0),
                Parameter::X6(20.0),
            ],
        );
    }

    #[test]
    /// Test simulation with L0123001 dataset from 1994-01-01 to 1998-12-31 w warmup period and
    /// different parameters.
    fn test_gr6j_l0123001_sc3() {
        compare_against_r_data(
            "airGR_results_L0123001_sc3.csv",
            1990,
            1998,
            Some(NaiveDate::from_ymd_opt(1994, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(1998, 12, 31).unwrap()),
            [
                Parameter::X1(31.0),
                Parameter::X2(3.47),
                Parameter::X3(32.0),
                Parameter::X4(2.1),
                Parameter::X5(0.55),
                Parameter::X6(5.3),
            ],
        );
    }

    #[test]
    /// Test the model with multiple hydrological units
    fn test_grg6_hu() {
        let file = File::open(test_path().join("airGR_L0123001_dataset.csv")).expect("Failed to read CSV file");
        let mut rdr = csv::Reader::from_reader(file);

        let mut time: Vec<NaiveDate> = vec![];
        let mut precipitation: Vec<f64> = vec![];
        let mut evapotranspiration: Vec<f64> = vec![];
        for result in rdr.records() {
            let record = result.unwrap();
            let date = NaiveDate::parse_from_str(record.get(0).unwrap(), "%d/%m/%Y").unwrap();
            time.push(date);
            precipitation.push(record.get(1).unwrap().parse::<f64>().unwrap());
            evapotranspiration.push(record.get(2).unwrap().parse::<f64>().unwrap());
        }

        let hu1 = CatchmentData {
            area: 10.0,
            x1: Parameter::X1(1000.0),
            x2: Parameter::X2(0.0),
            x3: Parameter::X3(200.0),
            x4: Parameter::X4(1.0),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(20.0),
            store_levels: None,
        };
        let hu2 = CatchmentData {
            area: 5.0,
            x1: Parameter::X1(2000.0),
            x2: Parameter::X2(2.0),
            x3: Parameter::X3(500.0),
            x4: Parameter::X4(3.2),
            x5: Parameter::X5(0.0),
            x6: Parameter::X6(15.0),
            store_levels: None,
        };
        let start = *time.first().unwrap();
        let end = *time.last().unwrap();
        let inputs = GR6JModelInputs {
            time,
            precipitation,
            evapotranspiration,
            catchment: CatchmentType::SubCatchments(vec![hu1, hu2]),
            run_period: ModelPeriod { start, end },
            warmup_period: None,
            destination: None,
            observed_runoff: None,
            run_off_unit: RunOffUnit::NoConversion,
        };

        let mut model = GR6JModel::new(inputs).unwrap();
        let results = model.run().expect("Cannot fetch results");
        assert_eq!(results.catchment_outputs.len(), 2);
    }
}
