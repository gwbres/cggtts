use plotly::{
    common::{
        AxisSide,
        //DashType,
        Font,
        HoverInfo,
        Mode,
        Side,
        Title,
    },
    layout::Axis,
    Layout, Plot, Scatter,
};

use serde::Serialize;

mod context;
pub use context::PlotContext;

use cggtts::prelude::Epoch;

/*
 * builds a standard 2D plot single Y scale,
 * ready to plot data against time (`Epoch`)
 */
pub fn build_default_plot(title: &str, y_title: &str) -> Plot {
    build_plot(
        title,
        Side::Top,
        Font::default(),
        "Epoch",
        y_title,
        (true, true), // y=0 lines
        true,         // show legend
        true,         // autosize
    )
}

/*
 * build a standard 2D plot dual Y axes,
 * to plot against `Epochs`
 */
pub fn build_default_2y_plot(title: &str, y1_title: &str, y2_title: &str) -> Plot {
    build_plot_2y(
        title,
        Side::Top,
        Font::default(),
        "Epoch",
        y1_title,
        y2_title,
        (false, false), // y=0 lines
        true,           // show legend
        true,           // autosize
    )
}

/*
 * Builds a Plot
 */
fn build_plot(
    title: &str,
    title_side: Side,
    title_font: Font,
    x_axis_title: &str,
    y_axis_title: &str,
    zero_line: (bool, bool), // plots a bold line @ (x=0,y=0)
    show_legend: bool,
    auto_size: bool,
) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title).font(title_font))
        .x_axis(
            Axis::new()
                .title(Title::new(x_axis_title).side(title_side))
                .zero_line(zero_line.0)
                .show_tick_labels(false),
        )
        .y_axis(
            Axis::new()
                .title(Title::new(y_axis_title))
                .zero_line(zero_line.0),
        )
        .show_legend(show_legend)
        .auto_size(auto_size);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

fn build_plot_2y(
    title: &str,
    title_side: Side,
    title_font: Font,
    x_title: &str,
    y1_title: &str,
    y2_title: &str,
    zero_line: (bool, bool), // plots a bold line @ (x=0,y=0)
    show_legend: bool,
    auto_size: bool,
) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title).font(title_font))
        .x_axis(
            Axis::new()
                .title(Title::new(x_title).side(title_side))
                .zero_line(zero_line.0)
                .show_tick_labels(false),
        )
        .y_axis(
            Axis::new()
                .title(Title::new(y1_title))
                .zero_line(zero_line.1),
        )
        .y_axis2(
            Axis::new()
                .title(Title::new(y2_title))
                .overlaying("y")
                .side(AxisSide::Right)
                .zero_line(zero_line.1),
        )
        .show_legend(show_legend)
        .auto_size(auto_size);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

/*
 * Builds a default chart, 2D, X = time axis
 */
pub fn build_chart_epoch_axis<T: Clone + Default + Serialize>(
    name: &str,
    mode: Mode,
    epochs: Vec<Epoch>,
    data_y: Vec<T>,
) -> Box<Scatter<f64, T>> {
    let txt: Vec<String> = epochs.iter().map(|e| e.to_string()).collect();
    Scatter::new(epochs.iter().map(|e| e.to_utc_seconds()).collect(), data_y)
        .mode(mode)
        //.web_gl_mode(true)
        .name(name)
        .hover_text_array(txt)
        .hover_info(HoverInfo::All)
}
