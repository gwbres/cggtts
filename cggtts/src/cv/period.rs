//! Common View observation and gather period

use crate::prelude::{Duration, Epoch, TimeScale};
use hifitime::Unit;

/// Standard setup duration (in seconds), as per BIPM specifications.
pub const BIPM_SETUP_DURATION_SECONDS: u32 = 180;

/// Standard tracking duration (in seconds), as per BIPM specifications
pub const BIPM_TRACKING_DURATION_SECONDS: u32 = 780;

/// Reference MJD used in Common View tracking
const REFERENCE_MJD: u32 = 50_722;

/// [CommonViewPeriod] describes the period of satellite
/// tracking and common view realizations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommonViewPeriod {
    /// Setup duration, is a [Duration] at the beginning
    /// of each common view period where data is not collected.
    /// This is historically a 3' duration yet still used by strict CGGGTTS 2E collection (arbitrary).
    pub setup_duration: Duration,
    /// Tracking duration is the active tracking [Duration].
    /// This is historically a 13' duration yet still used by strict CGGGTTS 2E collection (arbitrary).
    pub tracking_duration: Duration,
}

impl CommonViewPeriod {
    /// Creates a [CommonViewPeriod] as per historical
    /// BIPM Common View specifications.
    pub fn bipm_common_view_period() -> Self {
        Self {
            setup_duration: Duration::from_seconds(BIPM_SETUP_DURATION_SECONDS as f64),
            tracking_duration: Duration::from_seconds(BIPM_TRACKING_DURATION_SECONDS as f64),
        }
    }

    /// Returns total period of this [CommonViewPeriod],
    /// expressed as [Duration].
    /// ```
    /// use cggtts::prelude::CommonViewPeriod;
    ///
    /// let bipm = CommonViewPeriod::bipm_common_view_period();
    /// assert_eq!(bipm.total_period().to_seconds(), 960.0);
    /// ```
    pub fn total_period(&self) -> Duration {
        self.setup_duration + self.tracking_duration
    }

    /// Offset of first track for any given MJD, expressed in nanoseconds
    /// within that day.
    fn first_track_offset_nanos(&self, mjd: u32) -> i128 {
        // if not bimp period, return 0
        if self.setup_duration != Duration::from_seconds(BIPM_SETUP_DURATION_SECONDS as f64)
            || self.tracking_duration
            != Duration::from_seconds(BIPM_TRACKING_DURATION_SECONDS as f64)
        {
            return 0i128;
        }

        let tracking_nanos = self.total_period().total_nanoseconds();

        let mjd_difference = REFERENCE_MJD as i128 - mjd as i128;

        let offset_nanos = (mjd_difference
            // this is the shift per day
            * 4 * 1_000_000_000 * 60
            // this was the offset on MJD reference
            + 2 * 1_000_000_000 * 60)
            % tracking_nanos;

        if offset_nanos < 0 {
            offset_nanos + tracking_nanos
        } else {
            offset_nanos
        }
    }

    /// Returns date and time of next [CommonViewPeriod]
    /// expressed as an [Epoch]. `now` may be any [Epoch]
    /// but is usually `now()` when actively tracking.
    /// Although CGGTTS uses UTC strictly, we accept any timescale here.
    pub fn next_period_start(&self, now: Epoch) -> Epoch {
        let total_period = self.total_period();
        let total_period_nanos = total_period.total_nanoseconds();

        let now_utc = match now.time_scale {
            TimeScale::UTC => now,
            _ => Epoch::from_utc_duration(now.to_utc_duration()),
        };

        let mjd_utc = (now_utc.to_mjd_utc_days()).floor() as u32;
        let today_midnight_utc = Epoch::from_mjd_utc(mjd_utc as f64);

        let today_t0_offset_nanos = self.first_track_offset_nanos(mjd_utc);
        let today_offset_nanos = (now_utc - today_midnight_utc).total_nanoseconds();

        let today_t0_utc = today_midnight_utc + (today_t0_offset_nanos as f64) * Unit::Nanosecond;

        if today_offset_nanos < today_t0_offset_nanos {
            // still within first track
            today_t0_utc
        } else {
            let ith_period = (((now_utc - today_t0_utc).total_nanoseconds() as f64)
                / total_period_nanos as f64)
                .ceil() as i128;

            let number_periods_per_day = (24 * 3600 * 1_000_000_000) / total_period_nanos;

            if ith_period >= number_periods_per_day {
                let tomorrow_t0_offset_nanos = self.first_track_offset_nanos(mjd_utc + 1);

                Epoch::from_mjd_utc((mjd_utc + 1) as f64)
                    + tomorrow_t0_offset_nanos as f64 * Unit::Nanosecond
            } else {
                today_midnight_utc
                    + today_t0_offset_nanos as f64 * Unit::Nanosecond
                    + (ith_period * total_period_nanos) as f64 * Unit::Nanosecond
            }
        }
    }

    /// Returns remaining [Duration] before beginning of next
    /// [CommonViewPeriod]. `now` may be any [Epoch]
    /// but is usually `now()` when actively tracking.
    /// Although CGGTTS uses UTC strictly, we accept any timescale here.
    pub fn time_to_next_period(&self, now: Epoch) -> Duration {
        self.next_period_start(now) - now
    }

    /// Returns a new [CommonViewPeriod] with desired setup [Duration]
    /// for which data should not be collected (at the beginning of each period)
    pub fn with_setup_duration(&self, setup_duration: Duration) -> Self {
        let mut s = self.clone();
        s.setup_duration = setup_duration;
        s
    }

    /// Returns a new [CommonViewPeriod] with desired tracking [Duration]
    /// for which data should be collected (at the end of each period, after possible
    /// setup [Duration]).
    pub fn with_tracking_duration(&self, tracking_duration: Duration) -> Self {
        let mut s = self.clone();
        s.tracking_duration = tracking_duration;
        s
    }
}

#[cfg(test)]
mod test {

    use super::{BIPM_SETUP_DURATION_SECONDS, BIPM_TRACKING_DURATION_SECONDS};
    use crate::prelude::{CommonViewPeriod, Duration, Epoch};
    use hifitime::Unit;

    const BIPM_PERIOD_SPECIFICATION_SECONDS: u32 =
        BIPM_SETUP_DURATION_SECONDS + BIPM_TRACKING_DURATION_SECONDS;

    #[test]
    fn bipm_specifications() {
        let cv = CommonViewPeriod::bipm_common_view_period();
        assert_eq!(cv.total_period().to_seconds(), 960.0);
        assert_eq!(cv.setup_duration.to_seconds(), 180.0);
        assert_eq!(cv.tracking_duration.to_seconds(), 780.0);
    }

    #[test]
    fn mjd_first_day_track() {
        let cv = CommonViewPeriod::bipm_common_view_period();

        for (mjd, expected) in vec![
            (50720, 10 * 60 * 1_000_000_000),
            (50721, 6 * 60 * 1_000_000_000),
            (50722, 2 * 60 * 1_000_000_000),
            (50723, 14 * 60 * 1_000_000_000),
            (50724, 10 * 60 * 1_000_000_000),
            (59507, 14 * 60 * 1_000_000_000),
            (59508, 10 * 60 * 1_000_000_000),
            (59509, 6 * 60 * 1_000_000_000),
            (59510, 2 * 60 * 1_000_000_000),
            (59511, 14 * 60 * 1_000_000_000),
            (59026, 2 * 60 * 1_000_000_000),
        ] {
            let t0_offset = cv.first_track_offset_nanos(mjd);
            assert_eq!(t0_offset, expected);
        }
    }

    #[test]
    fn cv_next_period_start() {
        let cv = CommonViewPeriod::bipm_common_view_period();

        for (now, expected) in vec![
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
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(10.0),
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
            // 1 period into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 120.0 * Unit::Second,
                Epoch::from_mjd_utc(50722.0)
                    + BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 120.0 * Unit::Second,
            ),
            // two periods into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 120.0 * Unit::Second,
                Epoch::from_mjd_utc(50722.0)
                    + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 120.0 * Unit::Second,
            ),
            // two periods + 10s into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 10.0 * Unit::Second
                    + 120.0 * Unit::Second,
                Epoch::from_mjd_utc(50722.0)
                    + 3.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 120.0 * Unit::Second,
            ),
            // two periods + 90% into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 950.0 * Unit::Second
                    + 120.0 * Unit::Second,
                Epoch::from_mjd_utc(50722.0)
                    + 3.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
                    + 120.0 * Unit::Second,
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
            // MJD = 59_025 N-1
            (
                Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (34.0 * 60.0) * Unit::Second,
                Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (34.0 * 60.0) * Unit::Second,
            ),
            // MJD = 59_025 N-2 +10s
            (
                Epoch::from_mjd_utc(59025.0)
                    + 23.0 * Unit::Hour
                    + (34.0 * 60.0) * Unit::Second
                    + 10.0 * Unit::Second,
                Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
            ),
            // MJD = 59_025 N-2 +950s
            (
                Epoch::from_mjd_utc(59025.0)
                    + 23.0 * Unit::Hour
                    + (34.0 * 60.0) * Unit::Second
                    + 950.0 * Unit::Second,
                Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
            ),
            // // MJD = 59_025 N-1 => MJD 59_026 N-1
            (
                Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
                Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
            ),
            // // MJD = 59_025 N-1 +10s => MJD 59_026 T0
            (
                Epoch::from_mjd_utc(59025.0)
                    + 23.0 * Unit::Hour
                    + (50.0 * 60.0 + 10.0) * Unit::Second,
                CommonViewPeriod::bipm_common_view_period()
                    .next_period_start(Epoch::from_mjd_utc(59026.0)),
            ),
            // // MJD = 59_025 N-1 +10s => MJD 59_026 T0
            (
                Epoch::from_mjd_utc(59025.0)
                    + 23.0 * Unit::Hour
                    + (50.0 * 60.0 + 10.0) * Unit::Second,
                Epoch::from_mjd_utc(59026.0) + (2.0 * 60.0) * Unit::Second,
            ),
        ] {
            let next_start = cv.next_period_start(now);
            let error_nanos = (next_start - expected).abs().total_nanoseconds();
            assert!(
                error_nanos < 1,
                "test failed for now={:?} got {:?} (error is {}ns)",
                now,
                next_start,
                error_nanos
            );
        }
    }
}
