use crate::model::GR6JModel;
use crate::outputs::GR6JOutputs;
use crate::utils::series_max;
use chrono::{Datelike, NaiveDate};
use plotters::prelude::full_palette::GREY_A400;
use plotters::prelude::*;
use std::path::Path;

const FONT: &str = "sans-serif";
const AXIS_STYLE: (&str, i32, &RGBColor) = (FONT, 22, &BLACK);
const LABEL_STYLE: (&str, i32, &RGBColor) = (FONT, 20, &BLACK);

/// Generate the chart with the input data and the simulated run off.
///
/// # Arguments
///
/// * `model`: The GR6JModel struct.
/// * `results`: The GR6JOutputs struct.
/// * `destination`: The folder where to save the chart file.
///
/// returns: Result<(), Box<dyn std::error::Error>>
pub(crate) fn generate_summary_chart(
    model: &GR6JModel,
    results: &GR6JOutputs,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
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

        let mut p_max = series_max(series);
        if has_observed {
            let observed = model.observed.as_ref().unwrap();
            p_max = p_max.max(series_max(observed));
        }
        if p_max > 1.0 {
            p_max = p_max.ceil();
        };

        let mut cc = ChartBuilder::on(panel)
            .x_label_area_size(65)
            .y_label_area_size(95)
            .set_label_area_size(LabelAreaPosition::Left, 90)
            .margin_top(5)
            .margin_left(20)
            .margin_right(30)
            .build_cartesian_2d(t_range.clone(), 0.0..p_max)?;

        cc.configure_mesh()
            .y_desc(axis_labels[idx])
            .axis_desc_style(AXIS_STYLE)
            .label_style(LABEL_STYLE)
            .x_label_formatter(&|v| v.year().to_string())
            .draw()?;

        cc.draw_series(LineSeries::new(
            time.iter().zip(series).map(|(t, p)| (*t, *p)),
            ShapeStyle {
                color: colours[idx].to_rgba(),
                filled: false,
                stroke_width: 1,
            },
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], colours[idx].to_rgba()))
        .label(labels[idx]);

        if idx == 2 && model.observed.is_some() {
            let observed = model.observed.as_ref().unwrap();
            cc.draw_series(LineSeries::new(
                time.iter().zip(observed).map(|(t, p)| (*t, *p)),
                ShapeStyle {
                    color: BLACK.into(),
                    filled: false,
                    stroke_width: 1,
                },
            ))?
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK))
            .label("Observed");

            cc.configure_series_labels()
                .border_style(GREY_A400)
                .background_style(WHITE)
                .label_font((FONT, 20))
                .position(SeriesLabelPosition::Coordinate(2, 2))
                .draw()?;
        }
    }

    root_area.present()?;
    Ok(())
}

/// Data for a flow duration curve.
pub struct FDCData {
    /// The probability of flow exceedence.
    pub exceedence: Vec<f64>,
    /// The run-off corresponding to the exceedence probability.
    pub run_off: Vec<f64>,
}

/// Generate the chart with the FDCs.
///
/// # Arguments
///
/// * `model`: The GR6JModel struct.
/// * `simulated`: The FDCData struct for the FDC of the simulated run-off.
/// * `observed`: The FDCData struct for the FDC of the observed run-off.
/// * `destination`: The folder where to save the chart file.
///
/// returns: Result<(), Box<dyn std::error::Error>>
pub(crate) fn generate_fdc_chart(
    model: &GR6JModel,
    simulated: FDCData,
    observed: Option<FDCData>,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_file = destination.join("FDC.png");
    let root_area = BitMapBackend::new(&full_file, (1500 / 2, 1500 / 2)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let root_area = root_area.titled("Flow duration curve", (FONT, 30))?;
    let panels = root_area.split_evenly((2, 1));

    let has_observed = observed.as_ref().is_some();
    let mut q_max = series_max(&simulated.run_off);
    if has_observed {
        let observed = &observed.as_ref().unwrap().run_off;
        q_max = q_max.max(series_max(observed));
    }
    if q_max > 1.0 {
        q_max = q_max.ceil();
    };

    let sim_style = ShapeStyle {
        color: Palette99::pick(10).to_rgba(),
        filled: false,
        stroke_width: 1,
    };
    let obs_style = ShapeStyle {
        color: Palette99::pick(12).to_rgba(),
        filled: false,
        stroke_width: 1,
    };

    // First panel
    let mut cc1 = ChartBuilder::on(&panels[0])
        .x_label_area_size(65)
        .y_label_area_size(95)
        .set_label_area_size(LabelAreaPosition::Left, 90)
        .margin_top(5)
        .margin_left(20)
        .margin_right(30)
        .build_cartesian_2d(0.0..100.0, 0.0..q_max)?;

    cc1.configure_mesh()
        .y_desc(format!("Run off ({})", model.run_off_unit.unit_label()).as_str())
        .axis_desc_style(AXIS_STYLE)
        .label_style(LABEL_STYLE)
        .draw()?;

    cc1.draw_series(LineSeries::new(
        simulated
            .exceedence
            .iter()
            .zip(simulated.run_off.clone())
            .map(|(t, p)| (*t, p)),
        sim_style,
    ))?
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], sim_style.color.to_rgba()))
    .label("Simulated");

    // Second panel
    let mut cc2 = ChartBuilder::on(&panels[1])
        .x_label_area_size(65)
        .y_label_area_size(95)
        .set_label_area_size(LabelAreaPosition::Left, 90)
        .margin_top(5)
        .margin_left(20)
        .margin_right(30)
        .build_cartesian_2d(0.0..100.0, (0.0..q_max).log_scale())?;

    cc2.configure_mesh()
        .x_desc("Probability of exceedence (%)")
        .y_desc(format!("Log Run-off ({})", model.run_off_unit.unit_label()).as_str())
        .axis_desc_style(AXIS_STYLE)
        .label_style(LABEL_STYLE)
        .draw()?;

    cc2.draw_series(LineSeries::new(
        simulated.exceedence.iter().zip(simulated.run_off).map(|(t, p)| (*t, p)),
        sim_style,
    ))?
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], sim_style.color.to_rgba()))
    .label("Simulated");

    if has_observed {
        let fdc = observed.unwrap();
        cc1.draw_series(LineSeries::new(
            fdc.exceedence.iter().zip(&fdc.run_off).map(|(t, p)| (*t, *p)),
            obs_style,
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], obs_style.color.to_rgba()))
        .label("Observed");

        cc2.draw_series(LineSeries::new(
            fdc.exceedence.iter().zip(fdc.run_off).map(|(t, p)| (*t, p)),
            obs_style,
        ))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], obs_style.color.to_rgba()))
        .label("Observed");
    }

    Ok(())
}
