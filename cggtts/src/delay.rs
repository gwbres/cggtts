use strum_macros::{EnumString};
use rinex::constellation::Constellation;

#[derive(Debug, PartialEq, Clone, Copy)]
/// Different types of delay known,
/// refer to documentation to truly understand what they
/// represent. <!> Delays are always specified in nanoseconds <!> 
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

#[derive(Clone, Copy, PartialEq, Debug)]
#[derive(EnumString)]
pub enum Code {
    C1,
    C2,
    P1,
    P2,
    E1,
    E5a,
    B1,
    B2,
}

impl Default for Code {
    fn default() -> Code {
        Code::C1
    }
}

impl std::fmt::Display for Code {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Code::C1 => fmt.write_str("C1"),
            Code::C2 => fmt.write_str("C2"),
            Code::P1 => fmt.write_str("P1"),
            Code::P2 => fmt.write_str("P2"),
            Code::E1 => fmt.write_str("E1"),
            Code::E5a => fmt.write_str("E5a"),
            Code::B1 => fmt.write_str("B1"),
            Code::B2 => fmt.write_str("B2"),
        }
    }
}

/*
impl std::fmt::Display for CalibratedDelay {
fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.values.len() == 1 {
        fmt.write_str(&format!("{:.1} ns ({} {})", self.values[0] * 1E9, self.constellation, self.channels[0]))?
    } else {
        // CSV
        for i in 0..self.values.len()-1 {
            fmt.write_str(&format!("{:.1} ns ({} {}), ", 
                self.values[i] *1E9, self.constellation, self.channels[i]))? 
        }
        fmt.write_str(&format!("{:.1} ns ({} {})", 
            self.values[self.values.len()-1] *1E9, self.constellation, 
                self.channels[self.values.len()-1]))?
    }
    fmt.write_str(&format!("     CAL_ID = {}", self.report))?;
    Ok(())
}
}
*/

/*
/// Identifies carrier dependant informations
/// from a string shaped like '53.9 ns (GLO C1)'
fn carrier_dependant_delay_parsing (string: &str) 
        -> Result<(f64,track::Constellation,String),Error> 
{
    let (delay, const_str, channel) : (f64, String, String) = match scan_fmt!(string, "{f} ns ({} {})", f64, String, String) {
        (Some(delay),Some(constellation),Some(channel)) => (delay,constellation,channel),
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
    Ok((delay,constellation,channel))
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
    /// carrier dependend delays
    pub delays: Vec<(Code, Vec<Delay>)>,
    /// Calibration ID
    pub cal_id: Option<String>,
}

impl SystemDelay {
    /// Builds a new system delay description,
    /// with empty fields. Use `add_delay()` to customize.
    pub fn new () -> Self {
        Self {
            rf_cable_delay: 0.0_f64,
            ref_delay: 0.0_f64,
            delays: Vec::new(),
            cal_id: None,
        }
    }
    /// Returns total system delay for given carrier code
    pub fn total_delay (&self, code: Code) -> Option<Vec<f64>> {
        for (k, v) in self.delays.iter() {
            if *k == code {
                let mut data : Vec<f64> = Vec::new();
                for d in v.iter() {
                    data.push(d.value() 
                        + self.rf_cable_delay
                        + self.ref_delay
                    )
                }
                return Some(data)
            }
        }
        None
    }
}
