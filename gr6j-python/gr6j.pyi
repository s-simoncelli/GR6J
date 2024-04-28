from datetime import date
from enum import Enum

import pandas as pd


class X1:
    def __init__(self, value: float):
        """
        Set the maximum capacity of the production store (mm/day)
        :param value: The parameter value.
        """

    @staticmethod
    def unit() -> str:
        """
        The parameter unit of measurement.
        """

    @staticmethod
    def description() -> str:
        """
        The parameter description.
        """

    @staticmethod
    def min() -> str:
        """
        The parameter minimum value.
        """

    @staticmethod
    def max() -> str:
        """
        The parameter maximum value.
        """


class X2:
    def __init__(self, value: float):
        """
        Set the maximum capacity of the production store (mm/day)
        :param value: The parameter value.
        """

    @staticmethod
    def unit() -> str:
        """
        The parameter unit of measurement.
        """

    @staticmethod
    def description() -> str:
        """
        The parameter description.
        """

    @staticmethod
    def min() -> str:
        """
        The parameter minimum value.
        """

    @staticmethod
    def max() -> str:
        """
        The parameter maximum value.
        """


class X3:
    def __init__(self, value: float):
        """
        Set the maximum capacity of the production store (mm/day)
        :param value: The parameter value.
        """

    @staticmethod
    def unit() -> str:
        """
        The parameter unit of measurement.
        """

    @staticmethod
    def description() -> str:
        """
        The parameter description.
        """

    @staticmethod
    def min() -> str:
        """
        The parameter minimum value.
        """

    @staticmethod
    def max() -> str:
        """
        The parameter maximum value.
        """


class X4:
    def __init__(self, value: float):
        """
        Set the maximum capacity of the production store (mm/day)
        :param value: The parameter value.
        """

    @staticmethod
    def unit() -> str:
        """
        The parameter unit of measurement.
        """

    @staticmethod
    def description() -> str:
        """
        The parameter description.
        """

    @staticmethod
    def min() -> str:
        """
        The parameter minimum value.
        """

    @staticmethod
    def max() -> str:
        """
        The parameter maximum value.
        """


class X5:
    def __init__(self, value: float):
        """
        Set the maximum capacity of the production store (mm/day)
        :param value: The parameter value.
        """

    @staticmethod
    def unit() -> str:
        """
        The parameter unit of measurement.
        """

    @staticmethod
    def description() -> str:
        """
        The parameter description.
        """

    @staticmethod
    def min() -> str:
        """
        The parameter minimum value.
        """

    @staticmethod
    def max() -> str:
        """
        The parameter maximum value.
        """


class X6:
    def __init__(self, value: float):
        """
        Set the maximum capacity of the production store (mm/day)
        :param value: The parameter value.
        """

    @staticmethod
    def unit() -> str:
        """
        The parameter unit of measurement.
        """

    @staticmethod
    def description() -> str:
        """
        The parameter description.
        """

    @staticmethod
    def min() -> str:
        """
        The parameter minimum value.
        """

    @staticmethod
    def max() -> str:
        """
        The parameter maximum value.
        """


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
    x1: X1
    """  Maximum capacity of the production store (mm/day). """
    x2: X2
    """ Inter-catchment (or groundwater) exchange coefficient (mm/day). """
    x3: X3
    """ One-day-ahead maximum capacity of the routing store (mm/day). """
    x4: X4
    """ Time base of unit hydrograph (days). """
    x5: X5
    """  Inter-catchment exchange threshold. This is a dimensionless threshold
    parameter that allows a change in the direction of the groundwater exchange
    depending on the capacity of the routing store level. """
    x6: X6
    """ Time constant of exponential store (mm). """
    store_levels: StoreLevels | None = None
    """ The initial in the store levels. """

    def __init__(
            self,
            area: float,
            x1: X1,
            x2: X2,
            x3: X3,
            x4: X4,
            x5: X5,
            x6: X6,
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


class Metric:
    name: str
    """ The metric name """
    ideal_value: float
    """ The metric ideal value. """
    value: float
    """ The metric value from the difference between the observed and simulated data. """


class CalibrationMetric:
    nash_sutcliffe: Metric
    """ The Nash-Sutcliffe efficiency. An efficiency of 1 gives a perfect match of 
     simulated to observed data. An efficiency of 0 indicates that the model predictions 
     are as accurate as the mean of the observations, whereas an efficiency less than 
     zero occurs when the observed mean is a better predictor than the model.   """
    log_nash_sutcliffe: Metric
    """ The Nash-Sutcliffe efficiency but the logarithm is applied to flow data to give 
     more importance to low flow periods. An efficiency of 1 gives a perfect match of 
     simulated to observed data. """
    kling_gupta2009: Metric
    """ The 2009 Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
     of simulated to observed data. To calculate the alpha component the standard 
     deviation is used. """
    kling_gupta2012: Metric
    """  The 2012 Kling-Gupta efficiency metric. An efficiency of 1 gives a perfect match
     of simulated to observed data. To calculate the alpha component the ratio of the 
     standard deviation and the mean is used. """
    non_paramettric_kling_gupta: Metric
    """  The non-parametric Kling-Gupta efficiency metric. An efficiency of 1 gives a 
     perfect match of simulated to observed data. This differs from kling_gupta2012 and
     kling_gupta2012 because the alpha component is calculated using the flow percentile
     from the flow duration curve instead of using the standard deviation.
     See <https://www.tandfonline.com/doi/full/10.1080/02626667.2018.1552002> """


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
        print(results.to_dataframe())

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

    def to_dataframe(self) -> pd.DataFrame:
        """
        Get a DataFrame containing the time and the simulated run-off.
        :return: The DataFrame with the run-off.
        """


class GR6JModel:
    """
    Load and run a GR6J model.
    """

    def __init__(self, input: GR6JModelInputs):
        """
        Load the model with input data.
        :param input: The input data. See `GR6JModelInputs` how to build this class.
        """

    def run(self) -> GR6JModelOutputs:
        """
        Run the model and export charts and outputs.
        :return: A dictionary with the "time" and "run_off" keys containing the
        simulated time and run off values.
        """


class X1Range:
    lower_bound: float
    """ The parameter range lower bound. """
    upper_bound: float
    """ The parameter range upper bound. """

    def __init__(self, lower_bound: float | None, upper_bound: float | None):
        """
        Set the range for the maximum capacity of the production store (mm/day).
        :param lower_bound: The parameter lower bound. If None the X1 minimum value
        will be used.
        :param upper_bound: The parameter upper bound. If None the X1 maximum value
        will be used.
        """


class X2Range:
    lower_bound: float
    """ The parameter range lower bound. """
    upper_bound: float
    """ The parameter range upper bound. """

    def __init__(self, lower_bound: float | None, upper_bound: float | None):
        """
        Set the range for the inter-catchment (or groundwater) exchange coefficient
        (mm/day).
        :param lower_bound: The parameter lower bound. If None the X2 minimum value
        will be used.
        :param upper_bound: The parameter upper bound. If None the X2 maximum value
        will be used.
        """


class X3Range:
    lower_bound: float
    """ The parameter range lower bound. """
    upper_bound: float
    """ The parameter range upper bound. """

    def __init__(self, lower_bound: float | None, upper_bound: float | None):
        """
        Set the range for the one-day-ahead maximum capacity of the routing store (mm/day).
        :param lower_bound: The parameter lower bound. If None the X3 minimum value
        will be used.
        :param upper_bound: The parameter upper bound. If None the X3 maximum value
        will be used.
        """


class X4Range:
    lower_bound: float
    """ The parameter range lower bound. """
    upper_bound: float
    """ The parameter range upper bound. """

    def __init__(self, lower_bound: float | None, upper_bound: float | None):
        """
        Set the range for the time base of unit hydrograph (days).
        :param lower_bound: The parameter lower bound. If None the X4 minimum value
        will be used.
        :param upper_bound: The parameter upper bound. If None the X4 maximum value
        will be used.
        """


class X5Range:
    lower_bound: float
    """ The parameter range lower bound. """
    upper_bound: float
    """ The parameter range upper bound. """

    def __init__(self, lower_bound: float | None, upper_bound: float | None):
        """
        Set the range for the inter-catchment exchange threshold.
        :param lower_bound: The parameter lower bound. If None the X5 minimum value
        will be used.
        :param upper_bound: The parameter upper bound. If None the X5 maximum value
        will be used.
        """


class X6Range:
    lower_bound: float
    """ The parameter range lower bound. """
    upper_bound: float
    """ The parameter range upper bound. """

    def __init__(self, lower_bound: float | None, upper_bound: float | None):
        """
        Set the range for the time constant of exponential store (mm).
        :param lower_bound: The parameter lower bound. If None the X6 minimum value
        will be used.
        :param upper_bound: The parameter upper bound. If None the X6 maximum value
        will be used.
        """


class CalibrationCatchmentData:
    """
    The data for the catchment or hydrological unit to calibrate.
    """

    area: float
    """ The catchment os sub-catchment area (km2). """
    x1_range: X1Range
    """ Range for the maximum capacity of the production store (mm/day). """
    x2_range: X2Range
    """ Range for the inter-catchment (or groundwater) exchange coefficient (mm/day). """
    x3_range: X3Range
    """ Range for the one-day-ahead maximum capacity of the routing store (mm/day). """
    x4_range: X4Range
    """ Range for the time base of unit hydrograph (days). """
    x5_range: X5Range
    """ Range for the inter-catchment exchange threshold. """
    x6_range: X6Range
    """ Range for the time constant of exponential store (mm) """

    def __init__(
            self,
            area: float,
            x1_range: X1Range,
            x2_range: X2Range,
            x3_range: X3Range,
            x4_range: X4Range,
            x5_range: X5Range,
            x6_range: X6Range,
    ):
        """
        Define the data for the catchment or hydrological unit to calibrate. With this
        class you can specify the parameter ranges to use when random combinations of
        the parameters are generated to calibrate a model.
        :param area: The catchment os sub-catchment area (km2).
        :param x1_range: Range for the maximum capacity of the production store
        (mm/day). This is a two-item tuple with the lower an upper bound.
        :param x2_range: Range for the inter-catchment (or groundwater) exchange
        coefficient (mm/day). This is a two-item tuple with the lower an upper bound.
        This is a two-item tuple with the lower an upper bound.
        :param x3_range: Range for the one-day-ahead maximum capacity of the routing
        store (mm/day). This is a two-item tuple with the lower an upper bound.
        :param x4_range: Range for the time base of unit hydrograph (days). This is a
        two-item tuple with the lower an upper bound.
        :param x5_range: Range for the inter-catchment exchange threshold. This is a
        two-item tuple with the lower an upper bound.
        :param x6_range: Range for the time constant of exponential store (mm). This is
        a two-item tuple with the lower an upper bound.
        """


class CalibrationInputs:
    time: list[date]
    """ Vector of time. """
    precipitation: list[float]
    """ Input vector of total precipitation (mm/day). """
    evapotranspiration: list[float]
    """ Input vector of potential evapotranspiration (PE) (mm/day). """
    observed_runoff: list[float]
    """ The time series of the observed run-off. """
    catchment: CalibrationCatchmentData | list[CalibrationCatchmentData]
    """ Area and GR6J parameter ranges for one catchment or a list of sub-catchments 
    (in case you want to divide the catchment into independent hydrological units). """
    calibration_period: ModelPeriod
    """ The start and end date of the model run. """
    destination: str
    """ The path where to export (1) the comparison charts for the observed vs. 
    simulated flow, (2) the flow duration curves, (3) the scatter charts of the 
    calibration metrics to select the best calibration parameters and (4) a CSV file 
    with metric values. The files are exported to a sub-folder named with the run 
    timestamp. """
    run_off_unit: RunOffUnit
    """ The unit of measurement of the observed run-off. """
    sample_size: int
    """ The number of random combinations of the model parameters """
    generate_comparison_charts: bool
    """ Whether to export the comparison of the observed and simulated run-off time 
    series and flow duration curves for each model. """

    def __init__(
            self,
            time: list[date],
            precipitation: list[float],
            evapotranspiration: list[float],
            observed_runoff: list[float],
            catchment: CalibrationCatchmentData | list[CalibrationCatchmentData],
            calibration_period: ModelPeriod,
            destination: str,
            run_off_unit: RunOffUnit,
            sample_size: int | None = None,
            generate_comparison_charts: bool | None = None,
    ):
        """
        Define the input data to calibrate a GR6J model.
        :param time: Vector of time.
        :param precipitation: Input vector of total precipitation (mm/day)..
        :param evapotranspiration: Input vector of potential evapotranspiration (PE)
        (mm/day).
        :param observed_runoff: The time series of the observed run-off. This will be
        compared against the generated simulated run-off series to calculate the
        calibration metrics.
        :param catchment: Area and GR6J parameter ranges for one catchment or a list
        of sub-catchments (in case you want to divide the catchment into independent
        hydrological units). See `CalibrationCatchmentData`.
        :param calibration_period: The start and end date of the model run. The model
        can be run on a shorter time period compared to `time`.
        :param destination: The path where to export (1) the comparison charts for the
        observed vs. simulated flow, (2) the flow duration curves, (3) the scatter charts
        of the calibration metrics to select the best calibration parameters and (4) a
        CSV file with metric values. The files are exported to a sub-folder named with
        the run timestamp.
        :param run_off_unit: Convert the simulated run-off to the desired unit of
        measurement, so that it matches the unit of the observed run-off.
        :param sample_size: Generate the provided number of samples. Each sample
        contains a random combination of the model parameters based on the ranges
        given in the `catchment` argument. Default to `200` when `None`.
        :param generate_comparison_charts: Whether to export the comparison of the
        observed and simulated run-off time series and flow duration curves for each
         model. If `true`, the tool will generate as many as `self.sample_size` figures.
        """


class Calibration:
    time: list[date]
    """  The vector with the dates. """
    run_off: list[list[float]]
    """ The run-off for each simulated model. The size of the vector is 
    `CalibrationInputs.sample_size`. """
    nash_sutcliffe: list[float]
    """ The list of the Nash-Sutcliffe coefficients for all models. """
    log_nash_sutcliffe: list[float]
    """ The list of the log Nash-Sutcliffe coefficients for all models. """
    non_parametric_kling_gupta: list[float]
    """ The list of the non-parametric Kling-Gupta coefficients for all models. """
    rmse: list[float]
    """ The list of the oot-mean-square errors for all models. """
    volume_error: list[float]
    """ The list of the volume errors for all models. """

    def __init__(self, inputs: CalibrationInputs):
        """
        Run the calibration.
        :param inputs: The calibration inputs.
        """

    def x1_vec(self, catchment_index: int) -> list[float]:
        """
        Get the vector of X1 values for a catchment for all models.
        :param catchment_index: The index (0 based) of the catchment.
        :return: The parameter values.
        """

    def x2_vec(self, catchment_index: int) -> list[float]:
        """
        Get the vector of X2 values for a catchment for all models.
        :param catchment_index: The index (0 based) of the catchment.
        :return: The parameter values.
        """

    def x3_vec(self, catchment_index: int) -> list[float]:
        """
        Get the vector of X3 values for a catchment for all models.
        :param catchment_index: The index (0 based) of the catchment.
        :return: The parameter values.
        """

    def x4_vec(self, catchment_index: int) -> list[float]:
        """
        Get the vector of X4 values for a catchment for all models.
        :param catchment_index: The index (0 based) of the catchment.
        :return: The parameter values.
        """

    def x5_vec(self, catchment_index: int) -> list[float]:
        """
        Get the vector of X5 values for a catchment for all models.
        :param catchment_index: The index (0 based) of the catchment.
        :return: The parameter values.
        """

    def x6_vec(self, catchment_index: int) -> list[float]:
        """
        Get the vector of X6 values for a catchment for all models.
        :param catchment_index: The index (0 based) of the catchment.
        :return: The parameter values.
        """
