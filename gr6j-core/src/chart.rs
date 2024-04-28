use crate::inputs::RunOffUnit;
use crate::model::GR6JModel;
use crate::outputs::{CalibrationMetricVector, GR6JOutputs};
use crate::utils::{series_max, series_min, Fdc};
use chrono::{Datelike, NaiveDate};
use plotters::coord::ranged1d::ValueFormatter;
use plotters::coord::types::{RangedCoordf64, Yearly};
use plotters::coord::Shift;
use plotters::prelude::full_palette::GREY_A400;
use plotters::prelude::*;
use std::path::{Path, PathBuf};

const FONT: &str = "sans-serif";
const AXIS_STYLE: (&str, i32, &RGBColor) = (FONT, 22, &BLACK);
const LABEL_STYLE: (&str, i32, &RGBColor) = (FONT, 20, &BLACK);
const X_LABEL_AREA_SIZE: i32 = 65;
const Y_LABEL_AREA_SIZE: i32 = 95;

/// The chart context for a flow time-series
type FlowChartContext<'a, DB> = ChartContext<'a, DB, Cartesian2d<Yearly<NaiveDate>, RangedCoordf64>>;

/// The type return by render_time_series_panel()
type TimeSeriesDataOutput<'a, DB> = Result<FlowChartContext<'a, DB>, Box<dyn std::error::Error>>;

/// The return type of a chart function
type ChartResult = Result<(), Box<dyn std::error::Error>>;

/// The line style for the simulated data.
fn sim_style() -> ShapeStyle {
    ShapeStyle {
        color: Palette99::pick(10).to_rgba(),
        filled: false,
        stroke_width: 1,
    }
}
/// The line style for the observed data.
fn obs_style() -> ShapeStyle {
    ShapeStyle {
        color: Palette99::pick(12).to_rgba(),
        filled: false,
        stroke_width: 1,
    }
}

/// Render the legend box.
///
/// # Arguments
///
/// * `context`: The chart context
///
/// returns: `()`
fn render_legend_box<'a, DB: DrawingBackend, X, Y, XType>(
    context: &mut ChartContext<'a, DB, Cartesian2d<X, Y>>,
) -> ChartResult
where
    DB: 'a,
    DB::ErrorType: 'static,
    X: Ranged<ValueType = XType> + ValueFormatter<XType>,
    Y: Ranged<ValueType = f64> + ValueFormatter<f64>,
{
    context
        .configure_series_labels()
        .border_style(GREY_A400)
        .background_style(WHITE)
        .label_font((FONT, 20))
        .position(SeriesLabelPosition::Coordinate(2, 2))
        .draw()?;
    Ok(())
}

/// Generate the chart with the input data and the simulated run-off of a GR6J model.
///
/// # Arguments
///
/// * `model`: The GR6JModel struct.
/// * `results`: The GR6JOutputs struct.
/// * `destination`: The folder where to save the chart file.
///
/// returns: `ChartResult`
pub(crate) fn generate_summary_chart(model: &GR6JModel, results: &GR6JOutputs, destination: &Path) -> ChartResult {
    let time: Vec<NaiveDate> = results.time.clone();
    let all_series = [
        model.precipitation.clone(),
        model.evapotranspiration.clone(),
        results.run_off.clone(),
    ];

    let run_off_label = format!("Run off ({})", model.run_off_unit.unit_label());
    let axis_labels = ["Rainfall (mm)", "Evapotranspiration (mm)", &run_off_label];
    let labels = ["Rainfall", "Evapotranspiration", "Simulated"];

    let full_file = destination.join("Summary.png");
    let root_area = BitMapBackend::new(&full_file, (2100 / 2, 2970 / 2)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let root_area = root_area.titled("Inputs & simulated run-off", (FONT, 30))?;
    let panels = root_area.split_evenly((3, 1));
    let colours = [Palette99::pick(10), Palette99::pick(5), Palette99::pick(4)];

    let t_range = (*time.first().unwrap()..*time.last().unwrap()).yearly();
    for (idx, panel) in panels.iter().enumerate() {
        let series = &all_series[idx];
        let has_observed = idx == 2 && model.observed.is_some();

        let mut y_max = series_max(series);
        if has_observed {
            let observed = model.observed.as_ref().unwrap();
            y_max = y_max.max(series_max(observed));
        }
        if y_max > 1.0 {
            y_max = y_max.ceil();
        };

        let mut cc = render_time_series_panel(TimeSeriesData {
            panel,
            time: &time,
            series,
            t_range: &t_range,
            y_max,
            axis_label: axis_labels[idx],
            series_name: labels[idx],
            colour: colours[idx].to_rgba(),
        })?;

        if idx == 2 && model.observed.is_some() {
            add_obs_flow_to_context(&mut cc, &time, model.observed.as_ref().unwrap())?;
            render_legend_box(&mut cc)?;
        }
    }

    root_area.present()?;
    Ok(())
}

struct TimeSeriesData<'a, DB: DrawingBackend> {
    /// The panel where to render the series
    panel: &'a DrawingArea<DB, Shift>,
    /// The time vector
    time: &'a [NaiveDate],
    /// The value vector
    series: &'a [f64],
    /// The range to use for the time axis
    t_range: &'a Yearly<NaiveDate>,
    /// THe maximum y value
    y_max: f64,
    /// The name for the y-axis
    axis_label: &'a str,
    /// The name of the series
    series_name: &'a str,
    /// The series colour
    colour: RGBAColor,
}

/// Render a panel with a time-based series
fn render_time_series_panel<DB: DrawingBackend>(inputs: TimeSeriesData<DB>) -> TimeSeriesDataOutput<DB>
where
    DB::ErrorType: 'static,
{
    let mut cc = ChartBuilder::on(inputs.panel)
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .y_label_area_size(Y_LABEL_AREA_SIZE)
        .set_label_area_size(LabelAreaPosition::Left, 90)
        .margin_top(5)
        .margin_left(20)
        .margin_right(30)
        .build_cartesian_2d(inputs.t_range.clone(), 0.0..inputs.y_max)?;

    cc.configure_mesh()
        .y_desc(inputs.axis_label)
        .axis_desc_style(AXIS_STYLE)
        .label_style(LABEL_STYLE)
        .x_label_formatter(&|v| v.year().to_string())
        .draw()?;

    cc.draw_series(LineSeries::new(
        inputs.time.iter().zip(inputs.series).map(|(t, p)| (*t, *p)),
        ShapeStyle {
            color: inputs.colour,
            filled: false,
            stroke_width: 1,
        },
    ))?
    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], inputs.colour))
    .label(inputs.series_name);

    Ok(cc)
}

/// Render the chart line of the observed flow to an existing chart context.
///
/// # Arguments
///
/// * `context`: The chart context.
/// * `time`: The time vector.
/// * `observed`: The observed flow vector.
///
/// returns: `Result<(), Box<dyn Error>>`
fn add_obs_flow_to_context<DB: DrawingBackend>(
    context: &mut FlowChartContext<DB>,
    time: &[NaiveDate],
    observed: &[f64],
) -> ChartResult
where
    DB::ErrorType: 'static,
{
    context
        .draw_series(LineSeries::new(
            time.iter().zip(observed).map(|(t, p)| (*t, *p)),
            obs_style(),
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK))
        .label("Observed");

    Ok(())
}

/// Render a panel with the FDC.
///
/// # Arguments
///
/// * `panel`: The panel reference.
/// * `simulated_fdc`: The simulated FDC.
/// * `flow_unit`: The unit to show on the flow axis.
/// * `observed`: The observed FDC.
/// * `y_range`: The range to use on the y-axis. Use a log scale to plot the log FDC.
///
/// returns: `Result<(), Box<dyn Error>>`
fn render_fdc_panel<DB, Y>(
    panel: &DrawingArea<DB, Shift>,
    simulated_fdc: Fdc,
    flow_unit: &RunOffUnit,
    observed: Option<Fdc>,
    y_range: Y,
) -> ChartResult
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
    Y: Ranged<ValueType = f64> + ValueFormatter<f64>,
{
    let mut cc = ChartBuilder::on(panel)
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .y_label_area_size(Y_LABEL_AREA_SIZE)
        .set_label_area_size(LabelAreaPosition::Left, 90)
        .margin_top(5)
        .margin_left(20)
        .margin_right(30)
        .build_cartesian_2d(0.0..100.0, y_range)?;

    cc.configure_mesh()
        .x_desc("Probability of exceedence (%)")
        .y_desc(format!("Run-off ({})", flow_unit.unit_label()))
        .axis_desc_style(AXIS_STYLE)
        .label_style(LABEL_STYLE)
        .draw()?;

    cc.draw_series(LineSeries::new(
        simulated_fdc
            .exceedence
            .iter()
            .zip(simulated_fdc.sorted_run_off)
            .map(|(t, p)| (*t, p)),
        sim_style(),
    ))?
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], sim_style().color.to_rgba()))
    .label("Simulated");

    if let Some(fdc) = observed {
        cc.draw_series(LineSeries::new(
            fdc.exceedence.iter().zip(fdc.sorted_run_off).map(|(t, p)| (*t, p)),
            obs_style(),
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], obs_style().color))
        .label("Observed");
        render_legend_box(&mut cc)?;
    }

    Ok(())
}

/// Generate a chart with two panels; the top panel containing the simulated and observed FDCs
/// using a normal scale for the y-axis, the second panel with the same FDCs but the y-axis is
/// logarithmic.
///
/// # Arguments
///
/// * `model`: The GR6JModel struct.
/// * `simulated`: The FDC struct for the FDC of the simulated run-off.
/// * `observed`: The FDC struct for the FDC of the observed run-off.
/// * `destination`: The folder where to save the chart file.
///
/// returns: `ChartResult`
pub(crate) fn save_fdc_chart(
    model: &GR6JModel,
    simulated: Fdc,
    observed: Option<Fdc>,
    destination: &Path,
) -> ChartResult {
    let full_file = destination.join("FDC.png");
    let root_area = BitMapBackend::new(&full_file, (1500 / 2, 1500 / 2)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let root_area = root_area.titled("Flow duration curve", (FONT, 30))?;
    let panels = root_area.split_evenly((2, 1));

    let has_observed = observed.as_ref().is_some();
    let mut q_max = series_max(&simulated.sorted_run_off);
    if has_observed {
        let observed = &observed.as_ref().unwrap().sorted_run_off;
        q_max = q_max.max(series_max(observed));
    }
    if q_max > 1.0 {
        q_max = q_max.ceil();
    };

    render_fdc_panel::<BitMapBackend<'_>, RangedCoordf64>(
        &panels[0],
        simulated.clone(),
        &model.run_off_unit,
        observed.clone(),
        (0.0..q_max).into(),
    )?;

    render_fdc_panel::<BitMapBackend<'_>, LogCoord<f64>>(
        &panels[1],
        simulated,
        &model.run_off_unit,
        observed,
        (0.0..q_max).log_scale().into(),
    )?;

    Ok(())
}

/// Render a chart panel comparing parameter values against a metric values.
///
/// # Arguments
///
/// * `x`: The parameter values.
/// * `y`: The metric values.
/// * `panel`: The drawing area reference.
/// * `metric_name`: The name of the metric.
/// * `metric_ideal_value`: The ideal value the metric should reach.
///
/// returns: `Result<(), Box<dyn Error>>`
fn render_metric_vs_parameter_panel<DB: DrawingBackend>(
    x: &[f64],
    y: &[f64],
    panel: &DrawingArea<DB, Shift>,
    metric_name: &str,
    metric_ideal_value: f64,
) -> ChartResult
where
    DB::ErrorType: 'static,
{
    let x_min = series_min(x).floor();
    let x_max = series_max(x).ceil();

    let mut cc = ChartBuilder::on(panel)
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .y_label_area_size(45)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .margin_top(5)
        .margin_left(5)
        .margin_right(10)
        .build_cartesian_2d(x_min..x_max, series_min(y).floor()..series_max(y).ceil())?;

    cc.configure_mesh()
        .y_desc(metric_name)
        .axis_desc_style(AXIS_STYLE)
        .label_style(LABEL_STYLE)
        .draw()?;

    cc.draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
        x.iter().zip(y).map(|(xx, yy)| (*xx, *yy)).collect::<Vec<(f64, f64)>>(),
        3,
        BLACK.filled(),
    ))?;

    cc.draw_series(DashedLineSeries::new(
        [(x_min, metric_ideal_value), (x_max, metric_ideal_value)],
        6,
        6,
        ShapeStyle {
            color: Palette99::pick(10).to_rgba(),
            filled: false,
            stroke_width: 2,
        },
    ))?;

    Ok(())
}

/// Plot a chart for a parameter catchment (or sub-catchment) to compare the parameter values
/// against all calculated metrics.
///
/// # Arguments
///
/// * `x`: The vector with the parameter values.
/// * `metrics`: The vector with the metric values.
/// * `title`: The chart title.
/// * `destination`: The folder where to save the chart file.
///
/// returns: `Result<(), Box<dyn Error>>`
pub(crate) fn save_metric_vs_parameter_chart(
    x: &[f64],
    metrics: &CalibrationMetricVector,
    title: String,
    destination: &PathBuf,
) -> ChartResult {
    let first_metrics = &metrics.0.first().unwrap();

    let root_area = BitMapBackend::new(destination, (1800, 1200)).into_drawing_area();
    root_area.fill(&WHITE)?;
    let root_area = root_area.titled(&title, (FONT, 30))?;
    let panels = root_area.split_evenly((2, 3));

    // First panel
    render_metric_vs_parameter_panel(
        x,
        &metrics.nash_sutcliffe(),
        &panels[0],
        &first_metrics.nash_sutcliffe.name,
        first_metrics.nash_sutcliffe.ideal_value,
    )?;

    // Second panel
    render_metric_vs_parameter_panel(
        x,
        &metrics.log_nash_sutcliffe(),
        &panels[1],
        &first_metrics.log_nash_sutcliffe.name,
        first_metrics.log_nash_sutcliffe.ideal_value,
    )?;

    // Third panel
    render_metric_vs_parameter_panel(
        x,
        &metrics.non_parametric_kling_gupta(),
        &panels[2],
        &first_metrics.non_parametric_kling_gupta.name,
        first_metrics.non_parametric_kling_gupta.ideal_value,
    )?;

    // Fourth panel
    render_metric_vs_parameter_panel(
        x,
        &metrics.rmse(),
        &panels[3],
        &first_metrics.rmse.name,
        first_metrics.rmse.ideal_value,
    )?;

    // Fifth panel
    render_metric_vs_parameter_panel(
        x,
        &metrics.volume_error(),
        &panels[4],
        &first_metrics.volume_error.name,
        first_metrics.volume_error.ideal_value,
    )?;

    Ok(())
}

/// Plot a chart to compare the observed vs simulated flow and flow duration curve for one model.
///
/// # Arguments
///
/// * `time`: The time vector.
/// * `simulated`: The simulated flow time-series.
/// * `observed`: The observed flow time-series.
/// * `title`: The chart title.
/// * `destination`: The folder where to save the chart file.
/// * `flow_unit`: The unit of measurement for the flow.
///
/// returns: `Result<(), Box<dyn Error>>`
pub(crate) fn save_flow_comparison_chart(
    time: &[NaiveDate],
    simulated: &[f64],
    observed: &[f64],
    title: String,
    destination: &PathBuf,
    flow_unit: &RunOffUnit,
) -> ChartResult {
    let root_area = BitMapBackend::new(destination, (1800, 1200)).into_drawing_area();
    root_area.fill(&WHITE)?;
    let root_area = root_area.titled(&title, (FONT, 30))?;
    let panels = root_area.split_evenly((2, 1));

    // Flow panel
    let t_range = (*time.first().unwrap()..*time.last().unwrap()).yearly();
    let mut y_max = series_max(simulated);
    y_max = y_max.max(series_max(observed));

    let axis_label = format!("Run-off ({})", flow_unit.unit_label());
    let mut cc = render_time_series_panel(TimeSeriesData {
        panel: &panels[0],
        time,
        series: simulated,
        t_range: &t_range,
        y_max,
        axis_label: &axis_label,
        series_name: "Simulated",
        colour: sim_style().color.to_rgba(),
    })?;
    add_obs_flow_to_context(&mut cc, time, observed)?;
    render_legend_box(&mut cc)?;

    // Panel with log FDC
    render_fdc_panel::<BitMapBackend<'_>, LogCoord<f64>>(
        &panels[1],
        Fdc::new(simulated),
        flow_unit,
        Some(Fdc::new(observed)),
        (0.0..y_max).log_scale().into(),
    )?;

    Ok(())
}
