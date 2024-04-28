# GR6J rainfall-runoff model for Rust

![Logo](../logo.png)

This repository contains the GR6J rainfall-runoff model implemented in the Rust language.

The model replicates the code from the official [GR6J model](https://gitlab.irstea.fr/HYCAR-Hydro/airgr)
and was fully against the airGR R module. This tool adds much more features such as:

- ability to split a catchment into sub-catchments or hydrological units (for example based on surface use).
- ability to specify a run and warm-up period.
- export CSV files of [simulated run-off](gr6j-python/examples/20240428_0713/Run-off.csv),
  [parameters](gr6j-python/examples/20240428_0713/Parameters.csv), [metrics](gr6j-python/examples/20240428_0713/Metrics.csv)
  and
  [flow duration curve](gr6j-python/examples/20240428_0713/Metrics.csv).
- generate charts with the [simulated run-off](gr6j-python/examples/20240428_0713/Summary.png) and
  [flow duration curve](gr6j-python/examples/20240428_0713/FDC.png).
- calibrate a model using the Latin Hypercube approach by running models in threads
- assisted calibration by
  generating [parameter vs. metric charts](gr6j-python/examples/calibration_20240427_1253/X1_vs_metrics.png),
  [observed vs. simulated flow charts](gr6j-python/examples/calibration_20240427_1253/Flows_model98.png)
  and diagnostic files of [parameters](gr6j-python/examples/calibration_20240427_1253/Parameters.csv) and
  [metrics](gr6j-python/examples/calibration_20240427_1253/Metrics.csv).

## Getting started

Add the crate to your Rust project first:

```toml,ignore
[dependencies]
gr6j = "1.0.0"
```

The project contains three Rust examples in the `examples` folder. You can run
a [simple model](gr6j-core/examples/simple_model.rs)
or a model with [two sub-models or sub-catchments](gr6j-core/examples/two_hydrological_unit_model.rs) or
[calibrate](gr6j-core/examples/calibration.rs) the parameters using a gauged flow.

To run an example with Cargo use:

    cargo run --example simple_model
