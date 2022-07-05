//! scheduler: to schedule CGGTTS measurements
use chrono::Datelike;
use julianday::ModifiedJulianDay;

/// Modified Julian Day #50722 is taken as reference
/// for scheduling algorithm. Day 50722 is chosen so scheduling
/// is aligned with GPS sideral period.
const REFERENCE_MJD : i32 = 50722;

/// `BIPM` tracking duration recommendations.
/// `Cggtts` tracks must respect that duration
/// to be BIPM compliant, which is not mandatory 
pub const BIPM_RECOMMENDED_TRACKING: std::time::Duration = std::time::Duration::from_secs(13*60); 

pub struct Scheduler {
    /// Scheduler start time
    t0: chrono::NaiveDateTime,
    /// Measurements counter (within 24h)
    n: u32,
    /// Scheduling duration,
    /// should match BIPM recommendations,
    /// but custom value can be applied.
    /// It is not recommended to use a duration
    /// that is not multiple of `BIPM_RECOMMENDED_TRACKING`
    pub duration: std::time::Duration,
}

impl Scheduler {
    /// Builds a new measurement scheduler,
    /// Inputs:
    ///   t0: optionnal "starting date", otherwise
    ///   this core uses `now()` (creation datetime)
    ///
    ///   trk_duration: optionnal custom tracking duration,
    ///   defaults to `BIPM_RECOMMENDED_TRACKING`
    pub fn new (t0: Option<chrono::NaiveDateTime>, trk_duration: Option<std::time::Duration>) -> Self {
        Self {
            t0: t0.unwrap_or(chrono::Utc::now().naive_utc()),
            n: 0,
            duration: trk_duration.unwrap_or(BIPM_RECOMMENDED_TRACKING),
        }
    }

    /// Returns scheduled measurements for given day,
    /// if date is not provided, we use now()
    pub fn scheduled_events (&self, date: Option<chrono::NaiveDate>) -> Vec<chrono::NaiveDateTime> {
        let mut res : Vec<chrono::NaiveDateTime> = Vec::new();
        let mjd_ref = ModifiedJulianDay::new(REFERENCE_MJD).inner();
        let date = date.unwrap_or(chrono::Utc::now().naive_utc().date());
        let mjd = ModifiedJulianDay::from(date).inner();
        for i in 1..self.tracks_in_24h()-1 {
            let offset = Scheduler::time_ref(self.n) as i32 - 4*(mjd_ref - mjd)/60;
            if offset > 0 {
                let h = offset / 3600;
                let m = (offset - h*3600)/60;
                let s = offset -h*3600 - m*60;
                res.push(
                    chrono::NaiveDate::from_ymd(date.year(), date.month(), date.day())
                        .and_hms(h as u32 ,m as u32,s as u32));
            }
        }
        res
    }

    /// Returns duration (time interval) between given date
    /// and next scheduled measurement
    pub fn next (&self, datetime: chrono::NaiveDateTime) -> std::time::Duration {
        let offset = Scheduler::time_ref(self.n); 
        //let mjd = ModifiedJulianDay::from(datetime.naive_date()); 
        //let reference = ModifiedJulianDay::new(REFERENCE_MJD);
        //let start = offset - 4*(mjd.inner() - reference.inner()) * 60; 
        std::time::Duration::from_secs(10)
    }

    /// Returns offset in seconds during the course of `MJD_REF` 
    /// reference Modified Julian Day (defined in standards), 
    /// for given nth observation within that day.
    ///
    /// Input: 
    ///  - observation: observation counter 
    fn time_ref (observation: u32) -> u32 {
        60 * 2 + (observation -1)*16*60
    }
    
    /// Returns number of measurements to perform within 24hours
    fn tracks_in_24h (&self) -> u64 {
        24 * 3600 / self.duration.as_secs()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};
    #[test]
    fn test_scheduler_basic() {
        let t0 = chrono::NaiveDate::from_ymd(2022, 07, 05)
            .and_hms(00, 00, 00);
        let scheduler = Scheduler::new(Some(t0), None);
    }
}
