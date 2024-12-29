use cggtts::prelude::{Epoch, CGGTTS};
use itertools::Itertools;
use plotly::common::Mode;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::plot::{
    //build_timedomain_plot,
    build_chart_epoch_axis,
    PlotContext,
};

pub fn single_clock(cggtts: &CGGTTS, ctx: &mut PlotContext) {
    let sv: Vec<_> = cggtts.tracks_iter().map(|trk| trk.sv).unique().collect();
    let codes: Vec<_> = cggtts
        .tracks_iter()
        .map(|trk| trk.frc.clone())
        .unique()
        .collect();

    //REFSV/SRSV analysis
    ctx.add_timedomain_2y_plot(
        &format!("{} REFSV/SRSV", cggtts.header.station),
        "REFSV [s]",
        "SRSV [s/s]",
    );
    for sv in &sv {
        for code in &codes {
            let epochs: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                .collect();

            let refsv: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| {
                    if trk.sv == *sv {
                        Some(trk.data.refsv)
                    } else {
                        None
                    }
                })
                .collect();

            let srsv: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| {
                    if trk.sv == *sv {
                        Some(trk.data.srsv)
                    } else {
                        None
                    }
                })
                .collect();

            let chart = build_chart_epoch_axis(
                &format!("REFSV({};{})", sv, code),
                Mode::Markers,
                epochs.clone(),
                refsv,
            );
            ctx.add_trace(chart);

            let chart = build_chart_epoch_axis(
                &format!("SRSV({},{})", sv, code),
                Mode::Markers,
                epochs.clone(),
                srsv,
            )
            .y_axis("y2");
            ctx.add_trace(chart);
        }
    }

    //REFSYS/SRSYS analysis
    ctx.add_timedomain_2y_plot(
        &format!("{} REFSYS/SRSYS", cggtts.header.station),
        "REFSYS [s]",
        "SRSYS [s/s]",
    );
    for sv in &sv {
        for code in &codes {
            let epochs: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                .collect();

            let refsys: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| {
                    if trk.sv == *sv {
                        Some(trk.data.refsys)
                    } else {
                        None
                    }
                })
                .collect();

            let srsys: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| {
                    if trk.sv == *sv {
                        Some(trk.data.srsys)
                    } else {
                        None
                    }
                })
                .collect();

            let chart = build_chart_epoch_axis(
                &format!("REFSYS({};{})", sv, code),
                Mode::Markers,
                epochs.clone(),
                refsys,
            );
            ctx.add_trace(chart);

            let chart = build_chart_epoch_axis(
                &format!("SRSYS({},{})", sv, code),
                Mode::Markers,
                epochs.clone(),
                srsys,
            )
            .y_axis("y2");
            ctx.add_trace(chart);
        }
    }

    //TROPO
    ctx.add_timedomain_2y_plot(
        &format!("{} MDTR/SMDT", cggtts.header.station),
        "MDTR [s]",
        "SMDT [s/s]",
    );
    for sv in &sv {
        for code in &codes {
            let epochs: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                .collect();

            let mdtr: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| {
                    if trk.sv == *sv {
                        Some(trk.data.mdtr)
                    } else {
                        None
                    }
                })
                .collect();

            let smdt: Vec<_> = cggtts
                .tracks_iter()
                .filter_map(|trk| {
                    if trk.sv == *sv {
                        Some(trk.data.smdt)
                    } else {
                        None
                    }
                })
                .collect();

            let chart = build_chart_epoch_axis(
                &format!("MDTR({};{})", sv, code),
                Mode::Markers,
                epochs.clone(),
                mdtr,
            );
            ctx.add_trace(chart);

            let chart = build_chart_epoch_axis(
                &format!("SMDT({},{})", sv, code),
                Mode::Markers,
                epochs.clone(),
                smdt,
            )
            .y_axis("y2");
            ctx.add_trace(chart);
        }
    }
}

pub fn clock_comparison(workspace: &Path, pool: &Vec<CGGTTS>, ctx: &mut PlotContext) {
    let ref_clock = &pool[0];
    info!("{} is considered reference clock", ref_clock.header.station);

    let ref_sv: Vec<_> = ref_clock.tracks_iter().map(|trk| trk.sv).unique().collect();
    let ref_codes: Vec<_> = ref_clock
        .tracks_iter()
        .map(|trk| trk.frc.clone())
        .unique()
        .collect();
    let refsys: HashMap<Epoch, f64> = ref_clock
        .tracks_iter()
        .map(|trk| (trk.epoch, trk.data.refsys))
        .collect();

    for i in 1..pool.len() {
        ctx.add_timedomain_plot(
            &format!("{}-{}", ref_clock.header.station, pool[i].header.station),
            "Delta [s]",
        );
        for sv in &ref_sv {
            for code in &ref_codes {
                let x_err: Vec<_> = ref_clock
                    .tracks_iter()
                    .filter_map(|trk| {
                        if trk.sv == *sv && &trk.frc == code {
                            if refsys.get(&trk.epoch).is_some() {
                                Some(trk.epoch)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                let t_err: Vec<_> = ref_clock
                    .tracks_iter()
                    .filter_map(|trk| {
                        if trk.sv == *sv && &trk.frc == code {
                            refsys
                                .get(&trk.epoch)
                                .map(|refsys| trk.data.refsys - refsys)
                        } else {
                            None
                        }
                    })
                    .collect();

                let chart = build_chart_epoch_axis(
                    &format!("({};{})", sv, code),
                    Mode::Markers,
                    x_err,
                    t_err,
                );
                ctx.add_trace(chart);
            }
        }
    }

    let mut fd = File::create(workspace.join(&pool[0].header.station))
        .expect("failed to create textfile: permission denied");

    writeln!(fd, "t, CLOCK(A), CLOCK(B), SV, (elev[°], azi[°]) @REF, (elev[°], azi[°]) @CLOCK, SIGNAL, CLOCK(A) - CLOCK(B) [s]")
        .expect("failed to generate textfile");

    for trk in ref_clock.tracks_iter() {
        let ref_t = trk.epoch;
        let ref_sv = trk.sv;
        let (ref_elev, ref_azim) = (trk.elevation_deg, trk.azimuth_deg);
        let ref_frc = &trk.frc;
        for i in 1..pool.len() {
            let track = pool[i]
                .tracks_iter()
                .filter(|trk| trk.epoch == ref_t && trk.sv == ref_sv && trk.frc == *ref_frc)
                .reduce(|trk, _| trk);
            if let Some(b_trk) = track {
                let (b_elev, b_azim) = (b_trk.elevation_deg, b_trk.azimuth_deg);
                let dt = b_trk.data.refsys - trk.data.refsys;
                writeln!(
                    fd,
                    "{:?}, {}, {}, {}, ({:.2E}, {:.2E}), ({:.2E}, {:.2E}), {}, {:.3E}",
                    ref_t,
                    pool[i].header.station,
                    pool[0].header.station,
                    ref_sv,
                    ref_elev,
                    ref_azim,
                    b_elev,
                    b_azim,
                    ref_frc,
                    dt
                )
                .expect("failed to generate textfile");
            }
        }
    }
}
