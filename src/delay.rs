//! Delay 
//! Homepage: <https://github.com/gwbres/cggtts>

/// Different types of delay known,
/// refer to documentation to truely understand what they
/// represent. <!> Delays are always specified in nanoseconds <!> 
/// Use these value to increase system definition
/// and overall accuracy and take automate compensations
use rinex::Constellation;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Delay {
    /// Delay defined as `internal`
    Internal(f64),
    /// Cable / RF delay
    RfCable(f64),
    /// `Reference` delay
    Reference(f64),
    /// `System` delay
    System(f64),
}

impl Default for Delay {
    /// Default Delay is a total delay of 0 nanoseconds,
    /// ie., completly unknown delay
    fn default() -> Delay {
        Delay::RfCable(0.0_f64)
    }
}

impl Delay {
    /// Returns (`unwraps`) delay value in nanoseconds
    pub fn value (&self) -> f64 {
        match self {
            Delay::Internal(d) => *d,
            Delay::RfCable(d) => *d,
            Delay::Reference(d) => *d,
            Delay::System(d) => *d,
        }
    }

    /// Returns (`unwraps`) itself in seconds
    pub fn value_seconds (&self) -> f64 {
        self.value() * 1.0E-9
    }

    /// Adds given value in nanoseconds, to Self, regardless of delay type
    pub fn add_value (&self, v: f64) -> Self {
        match self {
            Delay::Internal(d) => Delay::Internal(*d + v), 
            Delay::RfCable(d) => Delay::RfCable(*d + v), 
            Delay::Reference(d) => Delay::Reference(*d + v), 
            Delay::System(d) => Delay::System(*d + v), 
        }
    }

    /// Converts self to calibrated delay,
    /// by associating a constellation and possible
    /// calibration process information
    pub fn calibrate (&self, constellation: Constellation, info: Option<String>) -> CalibratedDelay {
        CalibratedDelay {
            delay: self.clone(),
            constellation,
            info,
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

    /// Adds constellation against which this delay was calibrated
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

    /// Adds given delay value, in nanoseconds, to self
    pub fn add_value (&mut self, value: f64) {
        self.delay = self.delay
            .add_value(value)
    }

    /// Returns delay value in nanoseconds
    pub fn value (&self) -> f64 {
        self.delay.value()
    }

    /// Returns delay value in seconds
    pub fn value_seconds (&self) -> f64 {
        self.delay.value_seconds()
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
    /// Internal group of delays
    pub delays: Vec<CalibratedDelay>,
}

impl SystemDelay {
    /// Builds a new system delay description,
    /// with empty fields. Use `add_delay()` to customize.
    pub fn new () -> Self {
        Self {
            delays: Vec::with_capacity(3),
        }
    }

    /// Returns true if System delay can be trusted,
    /// meaning, all specified values are specified
    /// against a unique `GNSS` constellation
    pub fn trusted (&self) -> bool {
        for i in 0..self.delays.len() {
            if !self.delays[i].trusted() {
                return false
            }
        }
        true
    }

    /// Adds new calibrated delay to Self.
    /// User should avoid declaring Calibrated Delay against unspecified constellation
    /// (= `Constellation::Mixed`).
    /// * (1) If user already specified same kind of delay, it is accumulated to existing value.
    /// It is possible to add negative delay value. In case of (1), 
    /// it will decrease the contribution of this kind of delay
    /// * (2) If other kinds of delay were previously specified,
    /// GNSS system used in calibration must match already declared systems,
    /// except in the two following scenarios
    /// * (2a) if we're introducing a Delay value calibrated against `Constellation::Mixed` 
    /// and previous values were specified against a specific GNSS system,
    /// we redefine all calibration values to `Constellation::Mixed`, calibration is
    /// `untrusted()`: avoid this scenario.
    /// * (2b) if we're introducing a Delay value calibrated against a specific 
    /// Constellation while other values were declared as `Constellation::Mixed`,
    /// we introduce the given delay as `Constellation::Mixed`.
    pub fn add_delay (&mut self, new: CalibratedDelay) {
        if self.delays.len() == 0 {
            self.delays.push(new);
            return;
        }
        let mut rework = false;
        let mut matched = false;
        for i in 0..self.delays.len() {
            let delay = self.delays[i].delay;
            let constell = self.delays[i].constellation;
            match delay { 
                Delay::Internal(_) => {
                    if let Delay::Internal(v) = new.delay {
                        // Already exists, but kinds do match
                        if new.constellation == constell {
                            // perfect constellation match
                            self.delays[i] 
                                .add_value(v);
                        } else if new.constellation == Constellation::Mixed {
                            self.delays[i] 
                                .add_value(v);
                            rework = true
                        }
                        matched = true;
                        break;
                    } else {
                        continue
                    }
                },
                Delay::RfCable(_) => {
                    if let Delay::RfCable(v) = new.delay {
                        // Already exists, but kinds do match
                        if new.constellation == constell {
                            self.delays[i] 
                                .add_value(v);
                        } else if new.constellation == Constellation::Mixed {
                            self.delays[i] 
                                .add_value(v);
                            rework = true
                        }
                        matched = true;
                        break
                    } else {
                        continue
                    }
                },
                Delay::Reference(_) => {
                    if let Delay::Reference(v) = new.delay {
                        // Already exists, but kinds do match
                        if new.constellation == constell {
                            self.delays[i] 
                                .add_value(v);
                        } else if new.constellation == Constellation::Mixed {
                            self.delays[i] 
                                .add_value(v);
                            rework = true
                        }
                        matched = true;
                        break
                    } else {
                        continue
                    }
                },
                Delay::System(_) => {
                    if let Delay::System(v) = new.delay {
                        // Already exists, but kinds do match
                        if new.constellation == constell {
                            self.delays[i]
                                .add_value(v);
                        } else if new.constellation == Constellation::Mixed {
                            self.delays[i]
                                .add_value(v);
                            rework = true
                        }
                        matched = true;
                        break
                    } else {
                        continue
                    }
                },
            }
        }
        if !matched {
            self.delays.push(new);
        }
        if rework {
            for i in 0..self.delays.len() {
                self.delays[i].constellation = Constellation::Mixed 
            }
        }
    }

    /// Returns measurement systems total delay in nanoseconds
    pub fn value (&self) -> f64 {
        let mut dly: f64 = 0.0;
        for i in 0..self.delays.len() {
            dly += self.delays[i].value()
        }
        dly
    }

    /// Returns measurement systems total delay in seconds
    pub fn value_seconds (&self) -> f64 {
        self.value() * 1E-9
    }
}
