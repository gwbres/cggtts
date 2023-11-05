// use thiserror::Error;
//
// #[derive(Debug, Error)]
// pub enum Error {
//     #[error("bad operation: not synchronized yet")]
//     NotSynchronized,
// }

use crate::{Duration, Epoch, TimeScale};
use gnss::prelude::SV;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Default)]
/// CGGTTS tracking mode : either focused on a single SV
/// or a combination of SV
pub enum TrackingMode {
    #[default]
    Single,
    MeltingPot,
}

// Single SV tracker
#[derive(Debug, Clone, Default)]
pub(crate) struct SVTracker {
    /// Averaged pseudo range
    pub(crate) pseudo_range: f64,
    /// number of averages
    pub(crate) n_avg: u32,
}

impl SVTracker {
    fn new_data(&mut self, pr: f64) {
        let n = (self.n_avg + 1) as f64;
        self.pseudo_range += (pr - self.pseudo_range) / n;
        self.n_avg += 1;
    }
    fn reset(&mut self) {
        self.n_avg = 0_u32;
        self.pseudo_range = 0.0_f64;
    }
}

/// Synchronous Sky Tracker. The tracker
/// will not considerate data unless it is synchronized.
#[derive(Debug, Clone)]
pub struct SkyTracker {
    // internal trackers (real time updated)
    trackers: HashMap<SV, SVTracker>,
    // tracking duration: fixed to BIPM specs at the moment
    trk_duration: Duration,
    /// Tracking starting point: until then, nothing to be produced.
    /// First production is t0 + tracking_duration.
    pub t0: Epoch,
    /// true when we're aligned to the first track (ever)
    /// that means we're in position to produce data
    pub synchronized: bool,
    /// tracking mode
    pub mode: TrackingMode,
}

impl Default for SkyTracker {
    fn default() -> Self {
        Self {
            synchronized: false,
            t0: Epoch::default(),
            trackers: HashMap::new(),
            trk_duration: Duration::from_seconds(Self::BIPM_TRACK_DURATION_SECS.into()),
            mode: TrackingMode::default(),
        }
    }
}

impl SkyTracker {
    const BIPM_TRACK_DURATION_SECS: u32 = 960;
    const BIPM_TRACK_DURATION_MINUTES: u32 = 16;

    /* t0 offset (in minutes) within MJD=50722 reference day */
    fn mjd50722_t0_offset(i: u32) -> u32 {
        2 + (i - 1) * Self::BIPM_TRACK_DURATION_MINUTES * 60
    }

    /* t0 offset (in minutes), within any MJD */
    fn t0_offset_minutes(mjd: u32) -> u32 {
        ((mjd - 50722) * 4 + 2) % Self::BIPM_TRACK_DURATION_MINUTES
    }

    /* Next track start time, compared to current "t" */
    pub(crate) fn next_track_start(t: Epoch, trk_duration: Duration) -> Epoch {
        let mjd = t.to_mjd_utc_days();
        let mjd_u = mjd.floor() as u32;

        let mjd_next = Epoch::from_mjd_utc((mjd_u + 1) as f64);
        let time_to_midnight = mjd_next - t;

        match time_to_midnight < trk_duration {
            true => {
                /*
                 * if we're in the last track of the day,
                 * we need to consider next day (MJD+1)
                 */
                let offset_seconds = Self::t0_offset_minutes(mjd_u) * 60;
                Epoch::from_mjd_utc((mjd_u + 1) as f64)
                    + Duration::from_seconds(offset_seconds as f64)
            },
            false => {
                // determine which track within that day, "t" contributes to
                let day_offset_nanos = (t - Epoch::from_mjd_utc(mjd_u as f64)).total_nanoseconds();
                println!("{:?} : day offset nanos: {}", t, day_offset_nanos);
                let i = day_offset_nanos / trk_duration.total_nanoseconds();
                println!("{:?} : i(th) track: {}", t, i);
                // compute for i+1
                let offset_seconds = Self::t0_offset_minutes(mjd_u) * 60;
                Epoch::from_mjd_utc(mjd as f64)
                    + Duration::from_seconds(offset_seconds as f64)
                    + Duration::from_nanoseconds(
                        ((i + 1) * trk_duration.total_nanoseconds()) as f64,
                    )
            },
        }
    }

    /* Time to next track and potential next data production */
    pub fn time_to_next_track(&self, t: Epoch) -> Duration {
        Self::next_track_start(t, self.trk_duration) - t
    }

    /// Synchronization method. This should be called until Self.synchronized is true.
    pub fn synchronize(&mut self, t: Epoch) {
        self.t0 = Self::next_track_start(t, self.trk_duration);
        self.synchronized = t >= self.t0;
    }

    fn new_data(&mut self, t: Epoch, sv: SV, pr: f64) {
        // synchronize, if need be
        if !self.synchronized {
            self.synchronize(t);
        }

        let mut found = false;
        for (svnn, tracker) in self.trackers.iter_mut() {
            if *svnn == sv {
                found = true;
                tracker.new_data(pr);
                break;
            }
        }
        if !found {
            self.trackers.insert(
                sv,
                SVTracker {
                    n_avg: 0_u32,
                    pseudo_range: pr,
                },
            );
        }
    }
}
