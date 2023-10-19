use chrono::Timelike;
use thiserror::Error;
use format_num::NumberFormat;
use crate::scheduler;
use crate::{CrcError, calc_crc};
use crate::ionospheric;

use hifitime::{Epoch, Duration};
use gnss::prelude::{Constellation, SV};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Describes whether this common view is based on a unique 
/// or a combination of SV
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum CommonViewClass {
    /// Single Channel
    SingleChannel,
    /// Multi Channel
    MultiChannel,
}

impl std::fmt::UpperHex for CommonViewClass {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommonViewClass::Single => write!(fmt, "99"),
            CommonViewClass::Multiple => write!(fmt, "FF"),
        }
    }
}

/// Describes Glonass Frequency channel,
/// in case this `Track` was estimated using Glonass
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GlonassChannel {
    /// Default value when not using Glonass constellation
    Unknown,
    /// Glonass Frequency channel number
    Channel(u8),
}

impl std::fmt::Display for GlonassChannel {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GlonassChannel::Unknown => write!(fmt, "00"),
            GlonassChannel::Channel(c) => write!(fmt, "{:02X}", c),
        }
    }
}

impl PartialEq for GlonassChannel {
    fn eq (&self, rhs: &Self) -> bool {
        match self {
            GlonassChannel::Unknown => {
                match rhs {
                    GlonassChannel::Unknown => true,
                    _ => false
                }
            },
            GlonassChannel::Channel(c0) => {
                match rhs {
                    GlonassChannel::Channel(c1) => {
                        c0 == c1
                    },
                    _ => false,
                }
            }
        }
    }
}

impl Default for GlonassChannel {
    /// Default Glonass Channel is `Unknown`
    fn default() -> Self {
        Self::Unknown
    }
}

const TRACK_WITH_IONOSPHERIC :usize = 24;
const TRACK_WITHOUT_IONOSPHERIC :usize = 21;

/// A `Track` is a `Cggtts` measurement
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    /// Common view class.
    /// Most of the time, `Tracks` are estimated
    /// using a combination of Spave Vehicules
    pub class: CommonViewClass,
    /// Epoch of this track 
    pub epoch: Epoch,
    /// Tracking duration
    pub duration: Duration,
    /// Space vehicule against which this 
    /// measurement / track was realized.
    /// Is only relevant, as a whole, 
    /// if `class` is set to CommonViewClass::Single.
    /// Refer to [class]
    pub sv: Sv,
    /// Elevation (angle) at Tracking midpoint [in degrees]
    pub elevation: f64, 
    /// Azimuth (angle) at Tracking midpoint in [degrees]
    pub azimuth: f64, 
    pub refsv: f64,
    pub srsv: f64,
    pub refsys: f64,
    pub srsys: f64,
    /// Data signma (`DSG`)
    /// Root-mean-square of the residuals to linear fit from solution B in section 2.3.3
    pub dsg: f64,
    /// Issue of Ephemeris (`IOE`),
    /// Three-digit decimal code indicating the ephemeris used for the computation.
    /// As no IOE is associated with the GLONASS navigation messages, the values 1-96 have to be
    /// used to indicate the date of the ephemeris used, given by the number of the quarter of an hour in
    /// the day, starting at 1=00h00m00s. For BeiDou, IOE will report the integer hour in the date of the
    /// ephemeris (Time of Clock).
    pub ioe: u16,
    /// Modeled tropospheric delay corresponding to the solution C in section 2.3.3
    pub mdtr: f64, 
    /// Slope of the modeled tropospheric delay corresponding to the solution C in section 2.3.3
    pub smdt: f64, 
    /// Modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    pub mdio: f64, 
    /// Slope of the modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    pub smdi: f64, 
    /// Optionnal Ionospheric Data.
    /// Technically, these require a dual carrier
    /// GNSS receiver for their evaluation
    pub ionospheric: Option<ionospheric::IonosphericData>,
    /// Glonass Channel Frequency [1:24], O for other GNSS
    pub fr: GlonassChannel, 
    /// Receiver Hardware Channel [0:99], 0 if Unknown
    pub hc: u8, 
    /// Carrier frequency standard 3 letter code,
    /// refer to RINEX specifications for meaning
    pub frc: String, 
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("track data format mismatch")]
    InvalidDataFormatError(String),
    #[error("failed to parse space vehicule")]
    SvError(#[from] rinex::sv::Error),
    #[error("failed to parse int number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse track time")]
    ChronoParseError(#[from] chrono::ParseError),
    #[error("crc calc() failed over non utf8 data: \"{0}\"")]
    NonAsciiData(#[from] CrcError),
    #[error("checksum error - expecting \"{0}\" - got \"{1}\"")]
    ChecksumError(u8, u8),
}

impl Track {
    /// Builds a new CGGTTS measurement, referred to as `Track`,
    /// without known Ionospheric parameters, and
    /// production date set to `Today()`.
    /// To customize, use `with_` methods later on,
    /// for example to provide ionospheric parameters or use a different date
    pub fn new (class: CommonViewClass,
        epoch: Epoch,
        duration: Duration,
        sv: Sv,
        elevation: f64, 
        azimuth: f64, 
        refsv: f64, 
        srsv: f64,
        refsys: f64, 
        srsys:f64, 
        dsg: f64, 
        ioe: u16, 
        mdtr: f64,
        smdt: f64, 
        mdio: f64, 
        smdi: f64, 
        fr: GlonassChannel,
        hc: u8, 
        frc: &str,
    ) -> Self {
        Self {
            date: chrono::Utc::today().naive_utc(),
            class,
            sv,
            trktime,
            duration,
            elevation,
            azimuth,
            refsv,
            srsv,
            refsys,
            srsys,
            dsg,
            ioe,
            mdtr,
            smdt,
            mdio,
            smdi,
            ionospheric: None,
            fr,
            hc,
            frc: frc.to_string(),
        }
    }

    /// Returns a new `Track` with given Ionospheric parameters,
    /// if parameters were previously assigned, they get overwritten)
    pub fn with_ionospheric_data (&self, data: ionospheric::IonosphericData) -> Self {
        let mut t = self.clone();
        t.class = CommonViewClass::Multiple; // always when Iono provided
        t.ionospheric = Some(data);
        t
    }

    /// Returns a `Track` with desired duration
    pub fn with_duration (&self, duration: std::time::Duration) -> Self {
        let mut t = self.clone();
        t.duration = duration;
        t
    }

    /// Returns true if Self was estimated using a combination
    /// of Space Vehicules from the same constellation
    pub fn sv_combination (&self) -> bool {
        self.sv.prn == 99
    }

    /// Returns true if Self was measured against a unique
    /// Space Vehicule
    pub fn unique_sv (&self) -> bool {
        !self.sv_combination()
    }

    /// Returns true if Self was measured against given `GNSS` Constellation 
    pub fn uses_constellation (&self, c: Constellation) -> bool {
        self.sv.constellation == c
    }

    /// Returns True if Self follows BIPM specifications / requirements,
    /// in terms of tracking pursuit
    pub fn follows_bipm_specs (&self) -> bool {
        self.duration.as_secs() == scheduler::BIPM_RECOMMENDED_TRACKING.as_secs()
    }
    
    /// Returns a `Track` with desired unique space vehicule
    pub fn with_sv (&self, sv: Sv) -> Self {
        let mut t = self.clone();
        t.sv = sv.clone();
        t
    }

    /// Returns a track with desired elevation angle in Degrees
    pub fn with_elevation (&self, elevation: f64) -> Self {
        let mut t = self.clone();
        t.elevation = elevation;
        t
    }

    /// Returns a `Track` with given azimuth angle in Degrees, at tracking midpoint 
    pub fn with_azimuth (&self, azimuth: f64) -> Self { 
        let mut t = self.clone();
        t.azimuth = azimuth;
        t
    }

    /// Returns a `Track` with desired Frequency carrier code
    pub fn with_carrier_code (&self, code: &str) -> Self {
        let mut t = self.clone();
        t.frc = code.to_string();
        t
    }
    
    /// Returns true if Self comes with Ionospheric parameter estimates
    pub fn has_ionospheric_data (&self) -> bool { 
        self.ionospheric.is_some()
    }
}

impl Default for Track {
    /// Builds a default `Track` (measurement) structure,
    /// where `trktime` set to `now()` and producion date set to `today()`,
    /// common view class set to single,
    /// and other parameters set to defaults,
    /// and missing Ionospheric parameter estimates.
    fn default() -> Track {
        let now = chrono::Utc::now();
        Track {
            sv: {
                Sv {
                    constellation: Constellation::default(),
                    prn: 99,
                }
            },
            class: CommonViewClass::Single,
            ionospheric: None,
            date: chrono::Utc::today().naive_utc(),
            trktime: chrono::NaiveTime::from_hms(
                now.time().hour(),
                now.time().minute(),
                now.time().second()
            ),
            duration: scheduler::BIPM_RECOMMENDED_TRACKING, 
            elevation: 0.0_f64,
            azimuth: 0.0_f64,
            refsv: 0.0_f64,
            srsv: 0.0_f64,
            refsys: 0.0_f64,
            srsys: 0.0_f64,
            dsg: 0.0_f64,
            ioe: 0,
            mdtr: 0.0_f64,
            smdt: 0.0_f64,
            mdio: 0.0_f64,
            smdi: 0.0_f64,
            fr: GlonassChannel::default(),
            hc: 0,
            frc: String::from("XXX"), 
        }
    }
}

impl std::fmt::Display for Track {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = String::new();
        let num = NumberFormat::new();
        string.push_str(&format!("{} {} {} {} ",
            self.sv,
            self.class,
            julianday::ModifiedJulianDay::from(self.date).inner(),
            self.trktime.format("%H%M%S")));
        string.push_str(&format!("{} {} {} {} {} {} {} {} {} {} {} {} {} ",
            num.format("04d", self.duration.as_secs() as f64),
            num.format("03d", self.elevation * 10.0),
            num.format("04d", self.azimuth * 10.0),
            num.format("+11d", self.refsv * 1E10),
            num.format("+4d", self.srsv * 1E13),
            num.format("+11d", self.refsys * 1E10),
            num.format("+6d", self.srsys * 1E13),
            num.format("4d", self.dsg * 1E10),
            num.format("03d", self.ioe),
            num.format("04d", self.mdtr * 1E10),
            num.format("+04d", self.smdt * 1E13),
            num.format("04d", self.mdio * 1E10),
            num.format("+04d", self.smdi * 1E13),
            ));
        if let Some(iono) = self.ionospheric {
            string.push_str(&format!("{} {} {} ",
                num.format("11d", iono.msio * 1E10),
                num.format("+6d", iono.smsi * 1E13),
                num.format("04d", iono.isg * 1E10),
            ));
        }

        string.push_str(&format!("{:02} {:02X} {}",
            self.fr,
            self.hc,
            self.frc));

        if let Ok(crc) = calc_crc(&string) {
            string.push_str(&format!(" {:2X}", crc+32))
        }
        fmt.write_str(&string)
    }
}

impl std::str::FromStr for Track {
    type Err = Error; 
    /// Builds a `Track` from given str content
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let cleanedup = String::from(line.trim());
        let items: Vec<&str> = cleanedup.split_ascii_whitespace().collect();
        if items.len() != TRACK_WITH_IONOSPHERIC {
            if items.len() != TRACK_WITHOUT_IONOSPHERIC {
                return Err(Error::InvalidDataFormatError(String::from(cleanedup)))
            }
        }

        let sv = Sv::from_str(items[0])?;
        let class = items[1];
        let mjd = i32::from_str_radix(items[2], 10)?;
        let trktime = chrono::NaiveTime::parse_from_str(items[3], "%H%M%S")?;
        let duration_secs = u64::from_str_radix(items[4], 10)?;
        let elevation = f64::from_str(items[5])? * 0.1;
        let azimuth = f64::from_str(items[6])? * 0.1;
        let refsv = f64::from_str(items[7])? * 0.1E-9;
        let srsv = f64::from_str(items[8])? * 0.1E-12;
        let refsys = f64::from_str(items[9])? * 0.1E-9;
        let srsys = f64::from_str(items[10])? * 0.1E-12;
        let dsg = f64::from_str(items[11])? * 0.1E-9;
        let ioe = u16::from_str_radix(items[12], 10)?;
        let mdtr = f64::from_str(items[13])? * 0.1E-9;
        let smdt = f64::from_str(items[14])? * 0.1E-12;
        let mdio = f64::from_str(items[15])? * 0.1E-9;
        let smdi = f64::from_str(items[16])? * 0.1E-12;

        let (msio, smsi, isg, fr, hc, frc, ck) : 
            (Option<f64>,Option<f64>,Option<f64>,u8,u8,String,&str) 
            = match items.len() {
                TRACK_WITHOUT_IONOSPHERIC => {
                    (None,None,None,
                    u8::from_str_radix(items[17], 16)?, 
                    u8::from_str_radix(items[18], 10)?,
                    items[19].to_string(),
                    items[20])
                },
                TRACK_WITH_IONOSPHERIC => {
                    (Some(f64::from_str(items[17])? * 0.1E-9), 
                    Some(f64::from_str(items[18])? * 0.1E-12), 
                    Some(f64::from_str(items[19])? * 0.1E-9),
                    u8::from_str_radix(items[20], 16)?, 
                    u8::from_str_radix(items[21], 16)?,
                    items[22].to_string(),
                    items[23])
                },
                _ => return Err(Error::InvalidDataFormatError(String::from(cleanedup))),
        };

        // checksum 
        let end_pos = line.rfind(ck)
            .unwrap(); // already matching
        let _cksum = calc_crc(&line.split_at(end_pos-1).0)?;
        // verification
        /*if cksum != ck {
            println!("GOT {} EXPECT {}", ck, cksum);
            return Err(Error::ChecksumError(cksum, ck))
        }*/

        Ok(Track {
            class: {
                if class.eq("FF") {
                    CommonViewClass::Multiple
                } else {
                    CommonViewClass::Single
                }
            },
            sv,
            date: julianday::ModifiedJulianDay::new(mjd).to_date(),
            trktime,
            duration: std::time::Duration::from_secs(duration_secs),
            elevation,
            azimuth,
            refsv,
            srsv,
            refsys,
            srsys,
            dsg,
            ioe,
            mdtr,
            smdt,
            mdio,
            smdi,
            ionospheric: {
                if let (Some(msio), Some(smsi), Some(isg)) = (msio, smsi, isg) {
                    Some(ionospheric::IonosphericData {
                        msio,
                        smsi,
                        isg,
                    })
                } else {
                    None
                }
            },
            fr: {
                if fr == 0 {
                    GlonassChannel::Unknown
                } else {
                    GlonassChannel::Channel(fr)
                }
            },
            hc,
            frc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::GlonassChannel;
    #[test]
    fn test_glonass_channel() {
        let c = GlonassChannel::Unknown;
        assert_eq!(c.to_string(), "00");
        let c = GlonassChannel::Channel(1);
        assert_eq!(c.to_string(), "01");
        let c = GlonassChannel::Channel(10);
        assert_eq!(c.to_string(), "0A");
        assert_eq!(c, GlonassChannel::Channel(10));
        assert_eq!(c != GlonassChannel::Unknown, true);
        assert_eq!(GlonassChannel::default(), GlonassChannel::Unknown);
    }
}
