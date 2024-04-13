import pandas as pd
from datetime import date
from gr6j import GR6JModelInputs, CatchmentData, ModelPeriod, RunOffUnit, GR6JModel

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
    # create two hydrological units of sub-catchments
    catchment=[
        CatchmentData(area=2.0, x1=31, x2=3.47, x3=32, x4=2.1, x5=0.55, x6=5.3),
        CatchmentData(area=0.4, x1=1000, x2=1, x3=3, x4=1.2, x5=3, x6=1.3),
    ],
    run_period=ModelPeriod(start=start, end=end),
    # simulated run off and FDC will be exported as CSV files and figures
    destination=".",
    observed_runoff=data["Qmm"].tolist(),
    run_off_unit=RunOffUnit.NO_CONVERSION,
)

# Load the model and run it
model = GR6JModel(inputs)
results = model.run()

# Get the time and run-off vector as Pandas DataFrame use:
df = pd.DataFrame(zip(results.time, results.run_off), columns=["Time", "Run off"])
df.set_index("Time", inplace=True)
print(df)

# Get the exchange from routing store" for the only model and third time step use:
print(results.catchment_outputs[0][2].exchange_from_routing_store)
