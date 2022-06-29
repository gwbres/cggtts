//! Delay 
//! Homepage: <https://github.com/gwbres/cggtts>

/// Different types of delay known,
/// refer to documentation to truely understand what they
/// represent. <!> Delays are always specified in nanoseconds <!> 
/// Use these value to increase system definition
/// and overall accuracy and take automate compensations
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Delay {
    /// Delay defined as `internal`
    Internal(f64),
    /// Delay defined as `System` delay
    System(f64),
    /// Cable / RF delay
    RFCable(f64),
    /// `Reference` delay
    Reference(f64),
    /// `Total` delay
    Total(f64)
}

impl Default for Delay {
    /// Default Delay is a total delay of 0 nanoseconds,
    /// ie., completly unknown delay
    fn default() -> Delay {
        Delay::Total(0.0_f64)
    }
}

impl Delay {
    /// Returns (`unwraps`) delay value in nanoseconds
    pub fn value (&self) -> f64 {
        match self {
            Delay::Internal(d) => *d,
            Delay::System(d) => *d,
            Delay::RFCable(d) => *d,
            Delay::Reference(d) => *d,
            Delay::Total(d) => *d,
        }
    }

    /// Returns (`unwraps`) itself in seconds
    pub fn value_seconds (&self) -> f64 {
        self.value() * 1.0E-9
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
    constellation: rinex::Constellation,
    /// Actualy delay value
    delay: Delay,
    /// Calibration process information,
    /// usually laboraty code, ie., who performed that calibration
    info: Option<String>,
}

impl Default for CalibratedDelay {
    /// Builds a default `CalibratedDelay` with Null value and
    /// `Mixed` constellation, to mark this delay as non really specified
    fn default() -> CalibratedDelay {
        CalibratedDelay {
            constellation: rinex::Constellation::Mixed, 
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
            constellation: rinex::Constellation::Mixed,
            info: {
                if let Some(info) = info {
                    Some(info.to_string())
                } else {
                    None
                }
            }
        }
    }

    /// Adds constellation against which this delay was calibrated
    pub fn with_constellation (&self, c: rinex::Constellation) -> Self {
        Self {
            delay: self.delay,
            constellation: c,
            info: self.info.clone(),
        }
    }

    /// Returns true if this calibrated can be trusted,
    /// ie., was calibrated for a specific GNSS constellation,
    /// not `mixed` 
    pub fn trusted (&self) -> bool {
        self.constellation != rinex::Constellation::Mixed
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

