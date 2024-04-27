from datetime import date

import pandas as pd
from gr6j import (
    CalibrationCatchmentData,
    CalibrationInputs,
    ModelPeriod,
    RunOffUnit,
    X1Range,
    X2Range,
    X3Range,
    X4Range,
    X5Range,
    X6Range,
    Calibration,
)

# Read the input data
data = pd.read_csv(
    r"../../gr6j-core/src/test_data/airGR_L0123001_dataset.csv",
    index_col=[0],
    parse_dates=True,
    dayfirst=True,
)

# Set the parameter ranges to generate in the Latin Hyper-cube sampling
catchment_data = CalibrationCatchmentData(
    1,
    X1Range(None, 300),  # use None to set the bound to the min or max
    X2Range(0, 0),
    X3Range(None, None),
    X4Range(2, 7),
    X5Range(0, 0),
    X6Range(None, 3),
)

# Configure the calibration range
start = date(1990, 1, 1)
end = date(1994, 12, 31)

inputs = CalibrationInputs(
    time=data.index.tolist(),
    precipitation=data["P"].tolist(),
    evapotranspiration=data["E"].tolist(),
    observed_runoff=data["Qmm"].tolist(),
    catchment=catchment_data,
    calibration_period=ModelPeriod(start=start, end=end),
    destination=".",
    run_off_unit=RunOffUnit.NO_CONVERSION,
    sample_size=50,
)

# run the calibration
c = Calibration(inputs)

# get the vector of X1 parameter values generated for the first catchment
print(c.x1_vec(catchment_index=0))

# get the list of Nash-Sutcliffe coefficients calculated by comparing the simulated and
# observed flow
print(c.nash_sutcliffe)
