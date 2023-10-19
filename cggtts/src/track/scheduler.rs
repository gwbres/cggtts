//! CGGTTS Track scheduler

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Hash)]
struct TrackScheduler {
    /*
     * time of previous realization
     */
    last: Epoch,
}

impl TrackScheduler {
    /*
     * Modified Julian Day #50722 is taken as reference
     * for scheduling algorithm. Day 50722 is chosen so scheduling
     * is aligned to GPS sideral period
     */
    const REF_MJD :u32 = 50722; // used in calc
    pub const BIPM_TRACK_DURATION_SECS: u32 = 780; /* 13' */
    pub const BIPM_TRACK_DURATION : Duration = Duration {
        centuries: 0,
        nanoseconds: BIPM_TRACK_DURATION_SECS * 1_000_000_000,
    };
    /*
     * Returns Nth track offset, expressed in minutes
     */
    const fn time_ref(nth: u32) -> u32 {
        2 * (nth -1) * (BIPM_TRACK_DURATION_SECS / 60 +3) // 3'(warmup/lock?) +13' track
    }
    
    /// Returns true if we should publish a new realization "now"
    pub fn schedule(&mut self, now: Epoch) -> bool {
        // Returns nth track offset, in <!>minutes<!> within given MJD
        // time_ref(nth) - 4 * (mjd as u32 - REFERENCE_MJD);
        self.last = now;
    }
    /// Returns Epoch of next realization
    pub fn next(&self) -> Epoch {
        self.epoch + BIPM_TRACK_DURATION
    }
    /// Returns Epoch of previous realization
    pub fn last(&self) -> Epoch {
        self.epoch
    }
}
