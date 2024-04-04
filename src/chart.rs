use crate::model::GR6JModel;
use crate::outputs::GR6JOutputs;
use chrono::{Datelike, NaiveDate};
use plotters::prelude::full_palette::GREY_A400;
use plotters::prelude::*;
use std::path::Path;

const FONT: &str = "sans-serif";

/// Get the series max value
fn series_max(series: &[f64]) -> f64 {
    *series.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

/// Generate the chart with the input data and the simulated run off.
///
/// # Arguments
///
/// * `model`: The GR6JModel struct.
/// * `results`: The GR6JOutputs struct.
/// * `destination`: The folder where to save the chart file.
///
/// returns: Result<(), Box<dyn std::error::Error>>
pub fn generate_summary_chart(
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
    let axis_labels = ["Rainfall (mm)", "Evapotranspiration (mm)", "Run off (m³/day)"];
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
            .axis_desc_style((FONT, 22, &BLACK))
            .label_style((FONT, 20, &BLACK))
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
