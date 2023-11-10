use crate::prelude::{Duration, Epoch, TimeScale, TrackData};
use gnss::prelude::SV;
use hifitime::SECONDS_PER_DAY_I64;
use linreg::{linear_regression as linreg, Error as LinregError};
use std::collections::BTreeMap;
use thiserror::Error;

/// CGGTTS track formation errors
#[derive(Debug, Clone, Error)]
pub enum FitError {
    /// CGGTTS track fitting requires the track midpoint
    /// to be evaluated. For that, you need at least three measurements
    /// to be latched, {t_0, t_k, t_n} where t_k is the measurement
    /// at trk_duration /2
    #[error("failed to determine track midpoint")]
    UndeterminedTrackMidpoint,
    /// Linear regression failure
    #[error("linear regression failure")]
    LinearRegressionFailure,
}

/// CGGTTS track scheduler used to generate synchronous CGTTTS files.
#[derive(Debug, Clone)]
pub struct Scheduler {
    /// Tracking duration in use. Although our API allows it,
    /// you can only modify the tracking duration if you have
    /// complete access to both remote clocks, so they follow
    /// the same tracking procedure.
    pub trk_duration: Duration,
    /* next release */
    pub(crate) next_release: Epoch,
    /* internal buffer */
    buffer: BTreeMap<Epoch, FitData>,
}

/// CGGTTS track generation helper
#[derive(Debug, Default, Clone)]
pub struct FitData {
    /// REFSV [s]
    pub refsv: f64,
    /// REFSYS [s]
    pub refsys: f64,
    /// MDTR Modeled Tropospheric Delay [s]
    pub mdtr: f64,
    /// SV elevation [°]
    pub elevation: f64,
    /// SV azimuth [°]
    pub azimuth: f64,
    /// MDIO Modeled Ionospheric Delay [s]
    pub mdio: Option<f64>,
    /// MSIO Measured Ionospheric Delay [s]
    pub msio: Option<f64>,
}

impl Scheduler {
    /// Standard tracking duration [s]
    pub const BIPM_TRACKING_DURATION_SECONDS: u32 = 960;

    /// Initialize a Track Scheduler from a random (usually "now") datetime
    /// expressed as an Epoch.
    pub fn new(t0: Epoch, trk_duration: Duration) -> Self {
        (trk_duration.total_nanoseconds() / 1_000_000_000) as i64;
        let mut s = Self {
            trk_duration,
            buffer: BTreeMap::new(),
            next_release: Epoch::default(),
        };
        s.next_release = s.next_track_start(t0);
        s
    }

    /// Builds a CGGTTS scheduler with desired tracking duration
    pub fn tracking_duration(&self, trk_duration: Duration) -> Self {
        let mut s = self.clone();
        s.trk_duration = trk_duration;
        s
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

    /* returns midpoint Epoch */
    pub(crate) fn track_midpoint(&self) -> Option<Epoch> {
        let (t0, _) = self.buffer.first_key_value()?;
        for (t, data) in self.buffer.iter() {
            if *t >= *t0 + self.trk_duration / 2 {
                return Some(*t);
            }
        }
        None
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
    pub fn time_to_next_track(&self, now: Epoch) -> Duration {
        self.next_track_start(now) - now
    }

    /// Fit: Track generation procedure. You should prefer the
    /// "latch_measurement" procedure to generate synchronous CGGTTS.
    /// You must provide the ongoing Issue of Ephemeris when fitting your data.
    pub fn fit(&self, ioe: u16) -> Result<((f64, f64), TrackData), FitError> {
        let t_mid = self
            .track_midpoint()
            .ok_or(FitError::UndeterminedTrackMidpoint)?;

        let t_xs: Vec<_> = self
            .buffer
            .keys()
            .map(|t| t.to_duration().total_nanoseconds() as f64 * 1.0E-9)
            .collect();

        // let (srsv, srsv_b) = linreg(&t_xs, self.buffer.values().map(|f| f.refsv).as_slice())?;
        // let (srsys, srsys_b) = linreg(&t_xs, self.buffer.values().map(|f| f.refsys).as_slice())?;
        // let (smdt, smdt_b) = linreg(&t_xs, self.buffer.values().map(|f| f.mdtr).as_slice())?;
        // let (smdi, smdi_b) = linreg(&t_xs, self.buffer.values().map(|f| f.mdtr).as_slice())?;

        let elev = self
            .buffer
            .iter()
            .find(|(t, fitdata)| **t == t_mid)
            .unwrap() // unfaillible @ this point
            .1
            .elevation;

        let azi = self
            .buffer
            .iter()
            .find(|(t, fitdata)| **t == t_mid)
            .unwrap() // unfaillible @ this point
            .1
            .azimuth;

        //TODO
        // interpolate ax + b @ midpoint
        let refsv = 0.0_f64;
        let srsv = 0.0_f64;
        let refsys = 0.0_f64;
        let srsys = 0.0_f64;
        let dsg = 0.0_f64;
        let mdtr = 0.0_f64;
        let smdt = 0.0_f64;
        let mdio = 0.0_f64;
        let smdi = 0.0_f64;

        let trk_data = TrackData {
            refsv,
            srsv,
            refsys,
            srsys,
            dsg,
            ioe,
            mdtr,
            smdt,
            mdio,
            smdi,
        };

        Ok(((elev, azi), trk_data))
    }

    /// Latch new measurements and we may form a new track,
    /// if the new track generation Epoch has been reached.
    /// "ioe": ongoing Issue of Ephemeris.
    pub fn latch_measurements(
        &mut self,
        t: Epoch,
        data: FitData,
        ioe: u16,
    ) -> Result<Option<((f64, f64), TrackData)>, FitError> {
        if t >= self.next_release {
            let trk_data = self.fit(ioe)?;
            // reset buffer
            self.reset(t);
            // insert new data
            self.buffer.insert(t, data);
            Ok(Some(trk_data))
        } else {
            self.buffer.insert(t, data);
            Ok(None)
        }
    }

    /// Reset and flush previous measurements
    pub fn reset(&mut self, t: Epoch) {
        self.buffer.clear();
        self.next_release = self.next_track_start(t);
    }
}

#[cfg(test)]
mod test {
    use crate::track::Scheduler;
    use crate::{Duration, Epoch};
    #[test]
    fn t0_offset_minutes() {
        let duration = Duration::from_seconds(Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64);
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
            assert_eq!(Scheduler::t0_offset_nanos(mjd, duration), expected);
        }
    }
    #[test]
    fn next_track_scheduler() {
        let tracker = Scheduler::default();
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
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // two tracks + 10sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 130.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        3.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // two tracks + 950 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0 + 950.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        3.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
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
