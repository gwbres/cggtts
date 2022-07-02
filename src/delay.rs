//! Delay 
//! Homepage: <https://github.com/gwbres/cggtts>

/// Different types of delay known,
/// refer to documentation to truely understand what they
/// represent. <!> Delays are always specified in nanoseconds <!> 
/// Use these value to increase system definition
/// and overall accuracy and take automate compensations
use rinex::constellation::Constellation;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Delay {
    /// Delay defined as `internal`
    Internal(f64),
    /// `System` delay
    System(f64),
}

impl Default for Delay {
    fn default() -> Delay {
        Delay::System(0.0_f64)
    }
}

impl Delay {
    /// Returns (`unwraps`) delay value in nanoseconds
    pub fn value (&self) -> f64 {
        match self {
            Delay::Internal(d) => *d,
            Delay::System(d) => *d,
        }
    }
    /// Returns (`unwraps`) itself in seconds
    pub fn value_seconds (&self) -> f64 {
        self.value() * 1.0E-9
    }
    /// Adds value to self
    pub fn add_value (&self, rhs: f64) -> Self {
        match self {
            Delay::System(d) => Delay::System(*d + rhs),
            Delay::Internal(d) => Delay::Internal(*d + rhs),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// `CalibratedDelay` are basically [Delay] values 
/// that were calibrated for a specific carrier frequency,
/// hence a specific GNSS constellation.
/// Some extra information regarding calibration process
/// might be available
pub struct CalibratedDelay {
    /// GNSS constellation
    pub constellation: Constellation,
    /// Actualy delay value
    pub delay: Delay,
    /// Calibration process information,
    /// usually laboraty code, ie., who performed that calibration
    pub info: Option<String>,
}

impl Into<f64> for CalibratedDelay {
    fn into(self) -> f64 {
        self.delay.value()
    }
}

impl std::ops::Add<f64> for CalibratedDelay {
    type Output = CalibratedDelay;
    fn add (self, rhs: f64) -> Self {
        Self {
            constellation: self.constellation.clone(),
            delay: self.delay.add_value(rhs),
            info: self.info.clone(),
        }
    }
}

impl std::ops::Add<CalibratedDelay> for CalibratedDelay {
    type Output = CalibratedDelay;
    fn add (self, rhs: Self) -> Self {
        let mut allowed = self.constellation == rhs.constellation;
        allowed |= self.constellation == Constellation::Mixed;
        allowed |= rhs.constellation == Constellation::Mixed;
        let delay : Delay = match allowed {
            true => {
                match self.delay {
                    Delay::Internal(delay) => {
                        match rhs.delay {
                            Delay::Internal(d) => {
                                Delay::Internal(delay+d)
                            },
                            _ => Delay::Internal(delay), // unchanged: mismatch
                        }
                    },
                    Delay::System(delay) => {
                        match rhs.delay {
                            Delay::System(d) => {
                                Delay::System(delay+d)
                            },
                            _ => Delay::System(delay), // unchanged: mismatch
                        }
                    },
                }
            },
            false => self.delay.clone(),
        };
        Self {
            constellation: {
                if self.constellation == Constellation::Mixed {
                    Constellation::Mixed
                } else {
                    if rhs.constellation == Constellation::Mixed {
                        Constellation::Mixed
                    } else {
                        self.constellation.clone()
                    }
                }
            },
            delay,
            info: {
                if let Some(info) = self.info {
                    Some(info)
                } else {
                    if let Some(info) = rhs.info {
                        Some(info)
                    } else {
                        None
                    }
                }
            },
        }
    }
}

impl Default for CalibratedDelay {
    /// Builds a default `CalibratedDelay` with Null value and
    /// `Mixed` constellation, to mark this delay as non really specified
    fn default() -> CalibratedDelay {
        CalibratedDelay {
            constellation: Constellation::Mixed, 
            delay: Delay::default(),
            info: None,
        }
    }
}

/*
impl std::fmt::Display for CalibratedDelay {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.values.len() == 1 {
            fmt.write_str(&format!("{:.1} ns ({} {})", self.values[0] * 1E9, self.constellation, self.codes[0]))?
        } else {
            // CSV
            for i in 0..self.values.len()-1 {
                fmt.write_str(&format!("{:.1} ns ({} {}), ", 
                    self.values[i] *1E9, self.constellation, self.codes[i]))? 
            }
            fmt.write_str(&format!("{:.1} ns ({} {})", 
                self.values[self.values.len()-1] *1E9, self.constellation, 
                    self.codes[self.values.len()-1]))?
        }
        fmt.write_str(&format!("     CAL_ID = {}", self.report))?;
        Ok(())
    }
}
*/

impl CalibratedDelay {
    /// Builds a new `CalibratedDelay`
    pub fn new (delay: Delay, info: Option<&str>) -> Self {
        Self {
            delay,
            constellation: Constellation::Mixed,
            info: {
                if let Some(info) = info {
                    Some(info.to_string())
                } else {
                    None
                }
            }
        }
    }

    /// Returns delay value in nanoseconds 
    pub fn value (&self) -> f64 {
        self.delay.value()
    }

    /// Returns delay value in seconds 
    pub fn value_seconds (&self) -> f64 {
        self.delay.value_seconds()
    }

    /// Adds constellation against which this delay was calibrated.
    /// If constellation is not `rinex::Constellation::Mixed`,
    /// then Self becomes a trusted delay value
    pub fn with_constellation (&self, c: Constellation) -> Self {
        Self {
            delay: self.delay,
            constellation: c,
            info: self.info.clone(),
        }
    }

    /// Returns true if this calibrated delay can be trusted,
    /// ie., was calibrated against a specific `GNSS` constellation.
    pub fn trusted (&self) -> bool {
        self.constellation != Constellation::Mixed
    }

    /// Non trusted calibration, means this delay was not calibrated
    /// against a specific GNSS constellation
    pub fn non_trusted (&self) -> bool {
        !self.trusted()
    }
}

/*
/// Identifies carrier dependant informations
/// from a string shaped like '53.9 ns (GLO C1)'
fn carrier_dependant_delay_parsing (string: &str) 
        -> Result<(f64,track::Constellation,String),Error> 
{
    let (delay, const_str, code) : (f64, String, String) = match scan_fmt!(string, "{f} ns ({} {})", f64, String, String) {
        (Some(delay),Some(constellation),Some(code)) => (delay,constellation,code),
        _ => return Err(Error::FrequencyDependentDelayParsingError(String::from(string)))
    };
    let mut constellation: track::Constellation = track::Constellation::default();
    if const_str.eq("GPS") {
        constellation = track::Constellation::GPS
    } else if const_str.eq("GLO") {
        constellation = track::Constellation::Glonass
    } else if const_str.eq("BDS") {
        constellation = track::Constellation::Beidou
    } else if const_str.eq("GAL") {
        constellation = track::Constellation::Galileo
    } else if const_str.eq("QZS") {
        constellation = track::Constellation::QZSS
    }
    Ok((delay,constellation,code))
}
*/

/// System Delay describe the total measurement systems delay
/// to be used in `Cggtts`
#[derive(Clone, PartialEq, Debug)]
pub struct SystemDelay {
    /// RF/cable delay
    pub rf_cable_delay: f64,
    /// reference delay 
    pub ref_delay: f64,
    /// carrier dependend delay 
    pub calib_delay: CalibratedDelay,
}

impl SystemDelay {
    /// Builds a new system delay description,
    /// with empty fields. Use `add_delay()` to customize.
    pub fn new () -> Self {
        Self {
            rf_cable_delay: 0.0_f64,
            ref_delay: 0.0_f64,
            calib_delay: CalibratedDelay::default(),
        }
    }

    /// Returns true if Self was calibrated against
    /// a specific GNSS constellation
    pub fn trusted (&self) -> bool {
        self.calib_delay.trusted()
    }

    /// Returns true if Self was not calibrated against
    /// a specific GNSS constellation
    pub fn non_trusted (&self) -> bool {
        self.calib_delay.non_trusted()
    }

    /// Returns measurement systems total delay in nanoseconds
    pub fn value (&self) -> f64 {
        self.calib_delay.value() 
            + self.rf_cable_delay
            + self.ref_delay
    }
}

impl std::ops::Add<CalibratedDelay> for SystemDelay {
    type Output = SystemDelay;
    fn add (self, rhs: CalibratedDelay) -> Self {
        Self {
            rf_cable_delay: self.rf_cable_delay,
            ref_delay: self.ref_delay,
            calib_delay: self.calib_delay + rhs,
        }
    }
}
