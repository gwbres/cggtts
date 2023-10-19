//! scheduler: to schedule and perform CGGTTS measurements
//use crate::delay;
//use chrono::Datelike;

/// `BIPM` tracking duration recommendations.
/// `Cggtts` tracks must respect that duration
/// to be BIPM compliant, which is not mandatory
pub const BIPM_RECOMMENDED_TRACKING: std::time::Duration = std::time::Duration::from_secs(13 * 60);

/// Modified Julian Day #50722 is taken as reference
/// for scheduling algorithm. Day 50722 is chosen so scheduling
/// is aligned with GPS sideral period.
const REFERENCE_MJD: u32 = 50722;

/// Returns nth track offset, in <!>minutes<!> within
/// reference MJD
fn time_ref(nth: u32) -> u32 {
    2 * (nth - 1) * 16
}

/// Returns nth track offset, in <!>minutes<!> within given MJD
pub fn time_track(mjd: i32, nth: u32) -> u32 {
    time_ref(nth) - 4 * (mjd as u32 - REFERENCE_MJD)
}
