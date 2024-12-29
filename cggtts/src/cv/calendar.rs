//! Common View Planification table

use crate::prelude::{CommonViewPeriod, Duration, Epoch, TimeScale};
use hifitime::errors::HifitimeError;

/// [CommonViewCalendar] is a serie of evenly spaced [CommonViewPeriod]s.
pub struct CommonViewCalendar {
    /// Deploy time is the [Epoch] at which
    /// the first [CommonViewPeriod] was launched
    start_time: Epoch,
    /// [CommonViewPeriod] specification
    period: CommonViewPeriod,
}

impl CommonViewCalendar {
    /// Returns "now" (system time) expressed in [TimeScale::UTC].
    fn now_utc() -> Result<Epoch, HifitimeError> {
        Ok(Epoch::now()?.to_time_scale(TimeScale::UTC))
    }

    /// Design a new [CommonViewCalendar] (planification table)
    /// which is a serie of steady [CommonViewPeriod]s starting "right now",
    /// as opposed to [Self::new_postponed].
    /// Whether "right now" is already inside active measurement or not, is up
    /// to your [CommonViewPeriod] specifications.
    pub fn now(period: CommonViewPeriod) -> Result<Self, HifitimeError> {
        let now = Self::now_utc()?;
        Ok(Self {
            start_time: period.next_period_start(now),
            period,
        })
    }

    /// Design a new [CommonViewCalendar] (planification table)
    /// which is a serie of steady [CommonViewPeriod]s
    /// intended to start working later, as opposed to [Self::now].
    pub fn new_postponed(
        postponing: Duration,
        period: CommonViewPeriod,
    ) -> Result<Self, HifitimeError> {
        let now = Self::now_utc()?;
        Ok(Self {
            start_time: period.next_period_start(now + postponing),
            period,
        })
    }

    /// Returns true if this [CommonViewCalendar] is actively working.
    /// That means we're inside a [CommonViewPeriod]. Whether this is
    /// active measurement or not, depends on your [CommonViewPeriod] specifications.
    pub fn active(&self) -> Result<bool, HifitimeError> {
        let now = Self::now_utc()?;
        Ok(now > self.start_time)
    }

    /// Returns true if we're currently inside an observation period (active measurement).
    /// To respect this [CommonViewCalendar] table, your measurement system should be active!
    pub fn active_measurement(&self) -> Result<bool, HifitimeError> {
        let now = Self::now_utc()?;
        if now > self.start_time {
            // we're inside a cv-period
            Ok(false)
        } else {
            // not inside a cv-period
            Ok(false)
        }
    }
}
