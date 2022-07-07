//! scheduler: to schedule and perform CGGTTS measurements
use crate::delay;
use chrono::Datelike;

/// Modified Julian Day #50722 is taken as reference
/// for scheduling algorithm. Day 50722 is chosen so scheduling
/// is aligned with GPS sideral period.
const REFERENCE_MJD : i32 = 50722;

/// `BIPM` tracking duration recommendations.
/// `Cggtts` tracks must respect that duration
/// to be BIPM compliant, which is not mandatory 
pub const BIPM_RECOMMENDED_TRACKING: std::time::Duration = std::time::Duration::from_secs(13*60); 
