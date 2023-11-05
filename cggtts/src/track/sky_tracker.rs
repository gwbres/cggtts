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
pub struct SVTracker {
    /// Averaged pseudo range
    pub pseudo_range: f64,
    /// number of averages
    pub n_avg: u32,
}

impl SVTracker {
    fn latch_data(&mut self, pr: f64) {
        let n = (self.n_avg + 1) as f64;
        self.pseudo_range += (pr - self.pseudo_range) / n;
        self.n_avg += 1;
    }
    fn reset(&mut self) {
        self.n_avg = 0_u32;
        self.pseudo_range = 0.0_f64;
    }
}

/// Sky Tracker is used to track a Sky View
/// synchronously according to a predefined scheduling method.
#[derive(Debug, Clone)]
pub struct SkyTracker {
    /// internal trackers (real time updated)
    pub pool: HashMap<SV, SVTracker>,
    /// Tracking duration in use. It is not recommended
    /// to modify this duration when actively tracking.
    /// The CGGTTS data you compare should use identical tracking specifications.
    /// Therefore it is not recommended to modify this value unless
    /// you have means to adapt the two remote sites accordingly.
    pub trk_duration: Duration,
}

impl Default for SkyTracker {
    fn default() -> Self {
        Self {
            pool: HashMap::new(),
            trk_duration: Duration::from_seconds(Self::BIPM_TRACKING_DURATION_SECONDS as f64),
        }
    }
}

impl SkyTracker {
    pub const BIPM_TRACKING_DURATION_SECONDS: u32 = 960;

    /// Builds a Sky view tracker with specified tracking duration
    pub fn tracking_duration(&self, trk_duration: Duration) -> Self {
        let mut s = self.clone();
        s.trk_duration = trk_duration;
        s
    }

    /// Reset the sky tracker
    pub fn reset(&mut self) {
        for (_, trk) in self.pool.iter_mut() {
            trk.reset();
        }
    }

    /* track 0 offset within any MJD, expressed in nanos */
    pub(crate) fn t0_offset_nanos(mjd: u32, trk_duration: Duration) -> i128 {
        let tracking_nanos = trk_duration.total_nanoseconds();
        let offset_nanos = (
            (50_722 - mjd as i128)
            * 4 * 1_000_000_000 * 60  // shift per day
            + 2 * 1_000_000_000 * 60
            // offset on MJD=50722 reference
        ) % trk_duration.total_nanoseconds();
        if offset_nanos < 0 {
            offset_nanos + tracking_nanos
        } else {
            offset_nanos
        }
    }

    /// Next track start time, compared to current "t"
    pub fn next_track_start(&self, t: Epoch) -> Epoch {
        let trk_duration = self.trk_duration;
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
                let offset_nanos = Self::t0_offset_nanos(mjd_u + 1, trk_duration);
                Epoch::from_mjd_utc((mjd_u + 1) as f64)
                    + Duration::from_nanoseconds(offset_nanos as f64)
            },
            false => {
                let offset_nanos = Self::t0_offset_nanos(mjd_u, trk_duration);

                // determine track number this "t" contributes to
                let day_offset_nanos =
                    (t - Epoch::from_mjd_utc(mjd_u as f64)).total_nanoseconds() - offset_nanos;
                let i = (day_offset_nanos as f64 / trk_duration.total_nanoseconds() as f64).ceil();

                let mut e = Epoch::from_mjd_utc(mjd_u as f64)
                    + Duration::from_nanoseconds(offset_nanos as f64);

                // on first track of day: we only have the day nanos offset
                if i > 0.0 {
                    // add ith track offset
                    e += Duration::from_nanoseconds(i * trk_duration.total_nanoseconds() as f64);
                }
                e
            },
        }
    }

    /// Time remaining before next track production
    pub fn time_to_next_track(&self, t: Epoch) -> Duration {
        self.next_track_start(t) - t
    }

    /// Latch a new observation
    pub fn latch_data(&mut self, t: Epoch, sv: SV, pr: f64) {
        let mut found = false;
        for (svnn, tracker) in self.pool.iter_mut() {
            if *svnn == sv {
                found = true;
                tracker.latch_data(pr);
                break;
            }
        }
        if !found {
            self.pool.insert(
                sv,
                SVTracker {
                    n_avg: 0_u32,
                    pseudo_range: pr,
                },
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::track::SkyTracker;
    use crate::{Duration, Epoch};
    #[test]
    fn t0_offset_minutes() {
        let duration = Duration::from_seconds(SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64);
        for (mjd, expected) in vec![
            (50721, 6 * 60 * 1_000_000_000),
            (50722, 2 * 60 * 1_000_000_000),
            (50723, 14 * 60 * 1_000_000_000),
            (50724, 10 * 60 * 1_000_000_000),
            (59507, 14 * 60 * 1_000_000_000),
            (59508, 10 * 60 * 1_000_000_000),
            (59509, 6 * 60 * 1_000_000_000),
            (59510, 2 * 60 * 1_000_000_000),
        ] {
            assert_eq!(SkyTracker::t0_offset_nanos(mjd, duration), expected);
        }
    }
    #[test]
    fn next_track_scheduler() {
        let tracker = SkyTracker::default();
        for (t, expected) in vec![
            // reference MJD
            (
                Epoch::from_mjd_utc(50722.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 1 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 10 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 1 sec before reference MJD
            (
                Epoch::from_mjd_utc(50722.0) - Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 10 sec before reference MJD
            (
                Epoch::from_mjd_utc(50722.0) - Duration::from_seconds(10.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // two tracks into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // two tracks + 10sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64 + 130.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        3.0 * SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // two tracks + 950 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0 + 950.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        3.0 * SkyTracker::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // MJD = 59_506
            (
                Epoch::from_mjd_utc(59506.0),
                Epoch::from_mjd_utc(59506.0) + Duration::from_seconds(2.0 * 60.0),
            ),
            // MJD = 59_507
            (
                Epoch::from_mjd_utc(59507.0),
                Epoch::from_mjd_utc(59507.0) + Duration::from_seconds(14.0 * 60.0),
            ),
            // MJD = 59_508
            (
                Epoch::from_mjd_utc(59508.0),
                Epoch::from_mjd_utc(59508.0) + Duration::from_seconds(10.0 * 60.0),
            ),
            // MJD = 59_509
            (
                Epoch::from_mjd_utc(59509.0),
                Epoch::from_mjd_utc(59509.0) + Duration::from_seconds(6.0 * 60.0),
            ),
        ] {
            let next_track_start = tracker.next_track_start(t);
            println!("next track start: {:?}", next_track_start);
            let error_nanos = (next_track_start - expected).abs().total_nanoseconds();
            assert!(
                error_nanos < 10,
                "failed for {} with {} ns of error",
                t,
                error_nanos
            );
        }
    }
}
