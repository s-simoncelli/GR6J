/// The unit hydrpgraph type
#[derive(Debug)]
pub enum UnitHydrographType {
    T1,
    T2,
}

impl UnitHydrographType {
    pub fn size(&self) -> usize {
        match self {
            UnitHydrographType::T1 => 20,
            UnitHydrographType::T2 => 40,
        }
    }
}

/// The unit hydrpgraph inputs (i.e. the direct runoff hydrograph resulting from one unit of constant
/// intensity uniform rainfall occurring over the catchment).
#[derive(Debug)]
pub struct UnitHydrographInputs {
    /// The unit hydrograph type
    pub uh_type: UnitHydrographType,
    /// The hydropgrah time constant
    pub time_constant: f64,
    /// The hydropgrah exponent
    pub exponent: f64,
}

/// The unit hydrpgraph (i.e. the direct runoff hydrograph resulting from one unit of constant
/// intensity uniform rainfall occurring over the catchment)
#[derive(Debug)]
pub struct UnitHydrograph {
    /// The unit hydrograph type
    pub uh_type: UnitHydrographType,
    /// The hydropgrah time constant
    pub time_constant: f64,
    /// The hydropgrah exponent
    pub exponent: f64,
    /// The unit hydrograph values (-)
    pub values: Vec<f64>,
    /// The ordinates of the Unit Hydrograph (mm) to convert UH to surface run-off.
    pub ordinates: Vec<f64>,
}

impl UnitHydrograph {
    pub fn new(inputs: UnitHydrographInputs) -> Self {
        let values = match inputs.uh_type {
            UnitHydrographType::T1 => vec![0.0; 20],
            UnitHydrographType::T2 => vec![0.0; 40],
        };

        // Calculate the unit hydrograph using successive differences on the S-curve.
        let ordinates = {
            let size = inputs.uh_type.size();
            let mut uh = vec![0.0; size];
            for i in 1..=size {
                uh[i - 1] = Self::s_curve(&inputs, i as i64) - Self::s_curve(&inputs, (i as i64) - 1);
            }
            uh
        };

        UnitHydrograph {
            uh_type: inputs.uh_type,
            time_constant: inputs.time_constant,
            exponent: inputs.exponent,
            values,
            ordinates,
        }
    }

    /// Calculate the S-curve for the unit hydrograph (i.e. the direct runoff due to the effective
    /// rainfall applied over an infinite time).
    ///
    /// # Arguments
    ///
    /// * `inputs`: The `UnitHydrographInputs` struct with the hydrograph data.
    /// * `time_step`: The time-step where the S curve is calculated at.
    ///
    /// returns: The S-curve value.
    ///
    fn s_curve(inputs: &UnitHydrographInputs, time_step: i64) -> f64 {
        let i_time_step = time_step as f64;
        if time_step < 0 {
            return 0.0;
        }

        match inputs.uh_type {
            UnitHydrographType::T1 => {
                if i_time_step < inputs.time_constant {
                    (i_time_step / inputs.time_constant).powf(inputs.exponent)
                } else {
                    1.0
                }
            }
            UnitHydrographType::T2 => {
                if (time_step as f64) < inputs.time_constant {
                    0.5 * (i_time_step / inputs.time_constant).powf(inputs.exponent)
                } else if i_time_step < 2.0 * inputs.time_constant {
                    1.0 - 0.5 * (2.0 - i_time_step / inputs.time_constant).powf(inputs.exponent)
                } else {
                    1.0
                }
            }
        }
    }

    /// Perform the hydrograph convolution using excess rainfall.
    ///
    /// # Arguments
    ///
    /// * `max_index`: The maximum index to use to multiply the rainfall amount.
    /// * `precipitation`: The precipitation amount (mm).
    ///
    /// returns: ()
    pub fn convolution(&mut self, max_index: i32, precipitation: f64) {
        let total_uh = self.uh_type.size() as i32;
        let last_index = total_uh as usize - 1;
        match self.uh_type {
            UnitHydrographType::T1 => {
                for i in 0..(total_uh - 1).min(max_index + 1).max(1) {
                    let k = i as usize;
                    self.values[k] = self.values[k + 1] + self.ordinates[k] * precipitation;
                }
                self.values[last_index] = self.ordinates[last_index] * precipitation;
            }
            UnitHydrographType::T2 => {
                for i in 0..(2 * total_uh - 1).min(2 * (max_index + 1)).max(1) {
                    let k = i as usize;
                    self.values[k] = self.values[k + 1] + self.ordinates[k] * precipitation;
                }
                self.values[last_index] = self.ordinates[last_index] * precipitation;
            }
        }
    }
}
