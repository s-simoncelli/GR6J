from datetime import date
from enum import Enum

class StoreLevels:
    """
    Class to define custom store levels to use at the beginning of
    the simulation.
    """

    def __init__(
        self, production_store: float, routing_store: float, exponential_store: float
    ):
        """
        Initialise the class to define custom store levels to use at the beginning of
        the simulation. For example:

        ```
            levels = StoreLevels(production_store=0.4, routing_store=0.4, exponential_store=0.1)
        ```
        :param production_store: The production store level (mm)
        :param routing_store: The routing store level (mm)
        :param exponential_store: The exponential store level (mm)
        """

class CatchmentData:
    """
    The data for the catchment or hydrological unit. Use this class to define the
    catchment area, GR6J parameters and optional store levels.
    """

    area: float
    """ The area in km2 """
    x1: float
    """  Maximum capacity of the production store (mm/day). """
    x2: float
    """ Inter-catchment (or groundwater) exchange coefficient (mm/day). """
    x3: float
    """ One-day-ahead maximum capacity of the routing store (mm/day). """
    x4: float
    """ Time base of unit hydrograph (days). """
    x5: float
    """  Inter-catchment exchange threshold. This is a dimensionless threshold
    parameter that allows a change in the direction of the groundwater exchange
    depending on the capacity of the routing store level. """
    x6: float
    """ Time constant of exponential store (mm). """
    store_levels: StoreLevels | None = None
    """ The initial in the store levels. """

    def __init__(
        self,
        area: float,
        x1: float,
        x2: float,
        x3: float,
        x4: float,
        x5: float,
        x6: float,
        store_levels: StoreLevels | None = None,
    ):
        """
        Initialise the class to define the data (catchment area, GR6J parameters and
        optional store levels.) for the catchment or hydrological unit.

        :param area: The catchment os sub-catchment area (km2).
        :param x1: Maximum capacity of the production store (mm/day). This must be in
        the [0, 2500] range.
        :param x2: Inter-catchment (or groundwater) exchange coefficient (mm/day). X2
        can be positive or negative to simulate imports or exports of water with deep
        aquifers or surrounding catchments. This must be in the [-5, 5] range.
        :param x3: One-day-ahead maximum capacity of the routing store (mm/day). This
        must be in the [0, 1000] range.
        :param x4: Time base of unit hydrograph `UH1` (days). This must be in the
        [0.5, 10] range.
        :param x5: Inter-catchment exchange threshold. This is a dimensionless threshold
        parameter that allows a change in the direction of the groundwater exchange
        depending on the capacity of the routing store level. This must be in the
        [-4, 4] range.
        :param x6: Time constant of exponential store (mm). This must be in the [0, 20]
        range.
        :param store_levels: Specify the initial in the store levels. Optional to use
        the GR6J default initial  conditions.
        """

class ModelPeriod:
    """
    Class used to define a model time range.
    """

    start: date
    """  The period start date. """
    end: date
    """ The period end date. """

    def __init__(self, start: date, end: date):
        """
        Initialise a model time range.
        :param start: The period start date.
        :param end: The period end date.
        """

class RunOffUnit(Enum):
    """
    Enumerator used to specify the unit of measurements of the simulated run-off. It
    supports the following enumerations:
     - NO_CONVERSION: keep the run-off in mm*km2/d
     - CUBIC_METRE_PER_DAY: convert the run-off to m³/d
     - ML_PER_DAY: convert the run-off to Ml/d
     - CUBIC_METRE_PER_SECOND: convert the run-off to m³/s
    """

    NO_CONVERSION = None
    CUBIC_METRE_PER_DAY = None
    ML_PER_DAY = None
    CUBIC_METRE_PER_SECOND = None

class GR6JModelInputs:
    """
    The class used to define the inputs to the GR6J model. For example:
    ```
        import datetime
        from pathlib import Path

        import gr6j
        from gr6j.inputs import (
            StoreLevels,
            RunOffUnit,
            GR6JModelInputs,
            CatchmentData,
            ModelPeriod,
        )

        base = datetime.date.today()
        time = [base + datetime.timedelta(days=x) for x in range(7)]

        inputs = GR6JModelInputs(
            time=time,
            precipitation=[0.1, 0.4, 0.7, 1, 24, 12],
            evapotranspiration=[0.1, 0.4, 0.7, 0.1, 0.8, 0.1],
            catchment=CatchmentData(area=31, x1=31, x2=3.47, x3=32, x4=2.1, x5=0.55, x6=5.3),
            run_period=ModelPeriod(start=time[0], end=time[-2]),
            warmup_period=None,
            destination=Path("./results"),
            run_off_unit=RunOffUnit.CUBIC_METRE_PER_DAY,
        )
    ```
    """

    time: list[date]
    """ The time vector as a list of datetime objects """
    precipitation: list[float]
    """ List of total precipitation values (mm/day) """
    evapotranspiration: list[float]
    """ List of potential evapotranspiration (PE) values (mm/day) """
    catchment: list[CatchmentData] | CatchmentData
    """ Area and GR6J parameters for the catchment or a list of areas and parameters if
    you would like to divide the catchment into sub-catchments or hydrological units
    (for example based on surface type). This can be an instance of `CatchmentData`
    (for a single hydrological unit) or a list of `CatchmentData` instances (for
    multiple hydrological units) """
    run_period: ModelPeriod
    """" The start and end date of the model. The model can be run on a shorter time
    period compared to `time` """
    warmup_period: ModelPeriod | None = None
    """ The start and end date of the warm-up period. If `None` and `run_period.start`
    allows, the one-year period preceding the `run_period.start` is used. """
    destination: str | None = None
    """" Whether to export charts, the simulated run-off and other diagnostic file into
    a sub-folder inside the given destination folder. The sub-folder will be named with
    the run timestamp. """
    observed_runoff: list[float] | None = None
    """ The time series of the observed run-off. The time-series and its FDC will be
     plotted against the simulated run-off if `self.destination` is provided. """
    run_off_unit: RunOffUnit | None = None
    """ Convert the run-off to the desired unit of measurement. """

    def __init__(
        self,
        time: list[date],
        precipitation: list[float],
        evapotranspiration: list[float],
        catchment: list[CatchmentData] | CatchmentData,
        run_period: ModelPeriod,
        warmup_period: ModelPeriod | None = None,
        destination: str | None = None,
        observed_runoff: list[float] | None = None,
        run_off_unit: RunOffUnit | None = None,
    ):
        """
        Initialise the inputs to the GR6J model.
        :param time: The time vector as a list of `date` objects.
        :param precipitation: List of total precipitation values (mm/day)
        :param evapotranspiration: List of potential evapotranspiration (PE) values
        (mm/day)
        :param catchment: Area and GR6J parameters for the catchment or a list of
        areas and parameters if you would like to divide the catchment into
        sub-catchments or hydrological units (for example based on surface type). This
        can be an instance of `CatchmentData` (for a single hydrological unit) or a
        list of `CatchmentData` instances (for multiple hydrological units).
        :param run_period: The start and end date of the model. The model can be run
        on a shorter time period compared to `time`.
        :param warmup_period: The start and end date of the warm-up period. If `None`
        and `run_period.start` allows, the one-year period preceding the
        `run_period.start` is used. Default to None.
        :param destination: Whether to export charts, the simulated run-off and other
        diagnostic file into a sub-folder inside the given destination folder. The
        sub-folder will be named with the run timestamp. Default to None.
        :param observed_runoff: The time series of the observed run-off. The
        time-series and its FDC will be plotted against the simulated run-off if
        `self.destination` is provided. Default to None.
        :param run_off_unit: Convert the run-off to the desired unit of measurement.
        Default to None.
        """

class ModelStepData:
    time: date
    """ The time """
    evapotranspiration: float
    """ The potential evapotranspiration (PE) (mm) """
    precipitation: float
    """ The total precipitation (mm) """
    net_rainfall: float
    """ Net rainfall (mm) """
    store_levels: StoreLevels
    """ The store levels """
    storage_p: float
    """ Part of the precipitation filling the production store (mm) """
    actual_evapotranspiration: float
    """ Actual evapotranspiration """
    percolation: float
    """ Catchment percolation (mm) """
    pr: float
    """ `self.net_rainfall` - `self.storage_p` + `self.percolation` (mm) """
    exchange: float
    """ Potential third-exchange between catchments (mm) """
    exchange_from_routing_store: float
    """ Actual exchange between catchments from routing store (mm) """
    exchange_from_direct_branch: float
    """ Actual exchange between catchments from direct branch (after UH2) (mm) """
    actual_exchange: float
    """ Actual total exchange between catchments [`self.exchange_from_routing_store`] +
     [`self.exchange_from_direct_branch`] + [`self.exchange`] (mm) """
    routing_store_outflow: float
    """ Outflow from routing store (mm) """
    exponential_store_outflow: float
    """ Outflow from exponential store (mm) """
    outflow_from_uh2_branch: float
    """ Outflow from UH2 branch after exchange (mm) """
    run_off: float
    """ Simulated outflow at catchment outlet (mm) """

class GR6JModelOutputs:
    """
    Fetch the results. To get the time and run-off vector as Pandas DataFrame use:

        results = model.run()
        print(pd.DataFrame(zip(results.time, results.run_off), columns=["Time", "Run off"]))

    `self.catchment_data` contains the results for each sub-catchment or hydrological
    unit (HU) and time step. For example if you have two HU and want to get the
    "exchange from routing store" for the second model and third time step use:

        print(results.catchment_outputs[1][2].exchange_from_routing_store)

    """

    time: list[date]
    """ The time vector as a list of `date` objects. """
    run_off: list[float]
    """ The simulated run off as a list of floats. """
    catchment_outputs: list[list[ModelStepData]]
    """ A list of the data at each simulation time step. Each list item contains the 
    results for each sub-catchment or hydrological unit (with one catchment there is 
    only one list). Each nested list contains the results (as instances of 
    `ModelStepData`) for each time steps for the sub-model. """

class GR6JModel:
    """
    Load and run a GR6J model.
    """

    def __init__(self, input: GR6JModelInputs):
        """
        Load the model with input data.
        :param input: The input data. See GR6JModelInputs how to build this class.
        """

    def run(self) -> GR6JModelOutputs:
        """
        Run the model and export charts and outputs.
        :return: A dictionary with the "time" and "run_off" keys containing the
        simulated time and run off values.
        """
