//! CGTTS Track scheduler
#![cfg_attr(docrs, feature(doc_cfg))]

#[derive(Debug, Copy, Clone, Default)]
/// CGGTTS tracking mode : either focused on a single SV
/// or a combination of SV
pub enum TrackingMode {
    #[default]
    Single,
    MeltingPot,
}

use hifitime::{
    Duration,
    Epoch,
    Unit,
};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum Error {
    // GNSSSolverError(#[from] solver::Error),
}

use gnss::prelude::SV;
use crate::prelude::Track;
use rinex::prelude::Observable;
use rtk::solver as Solver;

use std::collections::HashMap;

use rinex::prelude::RnxContext;

/// TrackScheduler is a structure to generate CGGTTS Tracks by tracking one or several SV
#[derive(Debug, Clone)]
pub struct TrackScheduler {
    /* internal data */
    buffer: HashMap<(Epoch, SV, Observable), (u16, f64)>,
    /* gnss solver */
    solver: Solver,
    /// Tracking mode
    pub mode: TrackingMode,
    /// Tracking duration, by default we use TrackScheduler::BIPM_TRACKING_DURATION
    pub mode: Duration, 
}

impl TrackScheduler {
    /// Builds a new track Scheduler to resolve CGGTTS Tracks from given RINEX context.
    /// "trk_duration": Tracking duration typically set to Self::BIPM_TRACKING_DURATION but that can be customized.
    /// "mode": Tracking Mode, either focusing on a single SV or using a combination of SV to form the track.
    pub fn new(mode: TrackingMode, trk_duration: Duration, context: &RnxContext) -> Result<Self, Error> {
        Ok(Self {
            buffer: HashMap::with_capacity(32), 
            solver: Solver::from(context)?,
            mode,
            trk_duration,
        })
    }

    #[cfg_attr(docrs, doc(cfg(feature = "rinex")))]
    pub fn init(&mut self, ctx: &mut RnxContext) -> Result<(), Error> {
        self.solver.init(ctx)?
    }

    /// BIPM Tracking duration specifications, is the prefered tracking duration 
    pub const BIPM_TRACKING_DURATION : Duration  = Duration {
        centuries: 0,     
        nanoseconds: 780_000_000_000,
    };

    /*
     * Modified Julian Day #50722 is taken as reference
     * for scheduling algorithm. Day 50722 is chosen so scheduling
     * is aligned to GPS sideral period
     */
    const REF_MJD: u32 = 50722; // used in calc

    /*
     * Returns Nth track offset, expressed in minutes
     */
    const fn time_ref(nth: u32) -> u32 {
        2 * (nth - 1) * (780 / 60 + 3) // 3'(warmup/lock?) +13' track
    }

    /*
     * Returns currently tracked data for given SV and CODE
     */
    fn tracking(&mut self, sv: &SV, code: &Observable) -> Option<((Epoch, SV, Observable), (u16, f64))> {
        let key = self.buffer
            .keys()
            .filter_map(|(t, svnn, codenn)| {
                if svnn == sv && code == codenn {
                    Some((t, svnn, codenn))
                } else {
                    None
                }
            })
            .reduce(|data, _| data);
        let key = key?;
        let value = self.buffer.remove(key)?;
        Some((key, value))
    }
    
    /// Track provided Pseudo range (raw value) from given SV at "t" Epoch 
    /// and try a resolve a CGGTTS Track. Prior running this method,
    /// self.solver must be initialized first.
    /// Returns new Track if a new track can now be formed.
    /// A new CGGTTSTrack can be formed if SV was tracked for BIPM_TRACKING_DURATION without interruption.
    pub fn track(&mut self, t: Epoch, sv: SV, code: &Observable, pr: f64) -> Option<Track> {
        if !self.solver.initialized() {
            error!("cggtts bad op: solver should be initliazed first");
            return None;
        }

        if let Some((k, v)) = self.tracking(sv, code) {
            let (t_first, _, _) = k;
            let (n_avg, buff) = v;
            let tracking_duration = t - t_first;
            let sum += 
            if tracking_duration >= Self::BIPM_TRACKING_DURATION {
                /* forward this value to the solver */
                if let Ok(estimate) = self.solver.resolve();
                self.buffer.insert((t_first, sv, code), (pr, 0));
            } else {
                self.buffer.insert((t_first, sv, code), (pr, n_avg +1));
            }
        } else {
            self.buffer.insert(t, sv, code.clone(), (pr, 1));
            None
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let mut scheduler = TrackScheduler::new();
    }
}
