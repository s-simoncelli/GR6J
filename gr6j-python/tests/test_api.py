import pandas as pd
from datetime import date

import pytest
from gr6j import (
    GR6JModelInputs,
    CatchmentData,
    ModelPeriod,
    RunOffUnit,
    GR6JModel,
    CalibrationCatchmentData,
    X2,
    X1,
    X6,
    X5,
    X4,
    X3,
)


def test_simple_model():
    # Read the input data
    data = pd.read_csv(
        r"../../gr6j-core/src/test_data/airGR_L0123001_dataset.csv",
        index_col=[0],
        parse_dates=True,
        dayfirst=True,
    )

    # Configure the model
    start = date(1990, 1, 1)
    end = date(1994, 12, 31)

    inputs = GR6JModelInputs(
        time=data.index.tolist(),
        precipitation=data["P"].tolist(),
        evapotranspiration=data["E"].tolist(),
        catchment=CatchmentData(
            area=1.0,
            x1=X1(31),
            x2=X2(3.47),
            x3=X3(32),
            x4=X4(2.1),
            x5=X5(0.55),
            x6=X6(5.3),
        ),
        run_period=ModelPeriod(start=start, end=end),
        observed_runoff=data["Qmm"].tolist(),
        run_off_unit=RunOffUnit.NO_CONVERSION,
    )

    # Load the model and run it
    model = GR6JModel(inputs)
    model.run()


def test_destination_exception():
    t = [date(1999, 1, 1), date(1999, 1, 2)]
    inputs = GR6JModelInputs(
        time=t,
        precipitation=[1.0],
        evapotranspiration=[0.2],
        catchment=CatchmentData(
            area=1.0,
            x1=X1(31),
            x2=X2(3.47),
            x3=X3(32),
            x4=X4(2.1),
            x5=X5(0.55),
            x6=X6(5.3),
        ),
        run_period=ModelPeriod(start=t[0], end=t[-1]),
        destination="non_existing folder",
    )
    with pytest.raises(ValueError):
        GR6JModel(inputs)


def test_parameter_exception():
    with pytest.raises(ValueError):
        CatchmentData(
            area=1.0,
            x1=X1(0.0),
            x2=X2(3.47),
            x3=X3(32),
            x4=X4(2.1),
            x5=X5(0.55),
            x6=X6(5.3),
        )


def test_model_period_exception():
    with pytest.raises(ValueError):
        ModelPeriod(start=date(1999, 12, 1), end=date(1999, 1, 1))


def test_calibration_values_out_of_range():
    with pytest.raises(ValueError):
        CalibrationCatchmentData(
            area=1,
            x1_range=(2, 3000),
            x2_range=(2, 3),
            x3_range=(2, 3),
            x4_range=(0, 3),
            x5_range=(2, 3),
            x6_range=(2, 3),
        )
