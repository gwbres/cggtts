use crate::plot::{build_chart_epoch_axis, PlotContext};
use cggtts::prelude::CGGTTS;
use plotly::common::Mode;

pub fn single_clock(cggtts: &CGGTTS, ctx: &mut PlotContext) {
    let refsys: Vec<_> = cggtts.tracks().map(|trk| trk.data.refsys).collect();

    let srsys: Vec<_> = cggtts.tracks().map(|trk| trk.data.refsys).collect();

    let epochs: Vec<_> = cggtts.tracks().map(|trk| trk.epoch).collect();

    ctx.add_cartesian2d_2y_plot("REFSYS/SRSYS", "REFSYS", "SRSYS");

    let chart = build_chart_epoch_axis("REFSYS", Mode::LinesMarkers, epochs.clone(), refsys);
    ctx.add_trace(chart);

    let chart = build_chart_epoch_axis("SRSYS", Mode::Markers, epochs.clone(), srsys).y_axis("y2");
    ctx.add_trace(chart);
}

pub fn clock_comparison(pool: &Vec<CGGTTS>, ctx: &mut PlotContext) {}
