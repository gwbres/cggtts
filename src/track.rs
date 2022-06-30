use regex::Regex;
use chrono::Timelike;
use thiserror::Error;
use rinex::{Constellation, Sv};
use crate::cggtts::{CrcError, calc_crc};

/// `BIPM` tracking duration specifications.
/// `Cggtts` tracks must respect that duration
/// to be BIPM compliant, which is not mandatory 
const BIPM_SPECIFIED_DURATION: std::time::Duration = std::time::Duration::from_secs(13*60); 

#[derive(PartialEq, Clone, Copy, Debug)]
/// Describes whether this common view is based on a unique 
/// Space Vehicule or a combination of several vehicules
pub enum CommonViewClass {
    /// Single 
    Single,
    /// Multiple
    Multiple,
}

impl std::fmt::Display for CommonViewClass {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommonViewClass::Single => write!(fmt, "99"),
            CommonViewClass::Multiple => write!(fmt, "FF"),
        }
    }
}

/// Describes Glonass Frequency channel,
/// in case this `Track` was esimated using Glonass
#[derive(Debug, Copy, Clone)]
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
const PADDED_TRACK_WITHOUT_IONOSPHERIC :usize = 22;
const PADDED_TRACK_WITH_IONOSPHERIC :usize = 25;

#[derive(Debug, PartialEq, Clone)]
/// A `Track` is a `Cggtts` measurement
pub struct Track {
    /// Common view class.
    /// Most of the time, `Tracks` are estimated
    /// using a combination of Spave Vehicules
    pub class: CommonViewClass,
    /// Tracking start date (hh:mm:ss)
    pub trktime: chrono::NaiveTime, 
    /// Tracking duration
    pub duration: std::time::Duration, 
    /// Space vehicule against which this 
    /// measurement / track was realized.
    /// Is only relevant, as a whole, 
    /// if `class` is set to CommonViewClass::Single.
    /// Refer to [class]
    pub space_vehicule: rinex::Sv,
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
    pub ionospheric: Option<IonosphericData>,
    /// Glonass Channel Frequency [1:24], O for other GNSS
    pub fr: GlonassChannel, 
    /// Receiver Hardware Channel [0:99], 0 if Unknown
    pub hc: u8, 
    /// Carrier frequency standard 3 letter code,
    /// refer to RINEX specifications for meaning
    pub frc: String, 
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct IonosphericData {
    /// Measured ionospheric delay corresponding to the solution E in section 2.3.3.
    pub msio: f64, 
    /// Slope of the measured ionospheric delay corresponding to the solution E in section 2.3.3.
    pub smsi: f64, 
    /// [Ionospheric Sigma] Root-mean-square of the residuals corresponding to the solution E in section2.3.3
    pub isg: f64, 
}

impl Default for IonosphericData {
    /// Builds Null Ionospheric Parameter estimates
    fn default() -> Self {
        Self {
            msio: 0.0,
            smsi: 0.0,
            isg: 0.0,
        }
    }
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
    /// without known Ionospheric parameters. 
    /// To add Ionospheric data, use `with_ionospheric_data()` later on
    pub fn new (class: CommonViewClass,
            trktime: chrono::NaiveTime, duration: std::time::Duration,
                space_vehicule: rinex::Sv,
                elevation: f64, azimuth: f64, refsv: f64, srsv: f64,
                    refsys: f64, srsys:f64, dsg: f64, ioe: u16, mdtr: f64,
                        smdt: f64, mdio: f64, smdi: f64, fr: GlonassChannel,
                            hc: u8, frc: &str) -> Self {
        Self {
            class,
            space_vehicule,
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
    pub fn with_ionospheric_data (&self, data: IonosphericData) -> Self {
        let mut t = self.clone();
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
    pub fn space_vehicule_combination (&self) -> bool {
        !self.unique_space_vehicule()
    }

    /// Returns true if Self was measured against a unique
    /// Space Vehicule
    pub fn unique_space_vehicule (&self) -> bool {
        self.space_vehicule.prn != 99
    }

    /// Returns true if Self was measured against given `GNSS` Constellation 
    pub fn uses_constellation (&self, c: Constellation) -> bool {
        self.space_vehicule.constellation == c
    }

    /// Returns True if Self follows BIPM specifications / requirements,
    /// in terms of tracking pursuit
    pub fn follows_bipm_specs (&self) -> bool {
        self.duration.as_secs() == BIPM_SPECIFIED_DURATION.as_secs()
    }
    
    /// Returns a `Track` with desired unique space vehicule
    pub fn with_space_vehicule (&self, sv: Sv) -> Self {
        let mut t = self.clone();
        t.space_vehicule = sv.clone();
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
    /// where `trktime` set to `now()` as `UTC` time,
    /// common view is set to single,
    /// and other parameters set to defaults,
    /// and missing Ionospheric parameter estimates.
    fn default() -> Track {
        let now = chrono::Utc::now();
        Track {
            space_vehicule: {
                Sv {
                    constellation: Constellation::default(),
                    prn: 99,
                }
            },
            class: CommonViewClass::Single,
            ionospheric: None,
            trktime: chrono::NaiveTime::from_hms(
                now.time().hour(),
                now.time().minute(),
                now.time().second()
            ),
            duration: BIPM_SPECIFIED_DURATION, 
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
        string.push_str(&format!("{} {} 57000 {} {:0>4} {:0>3} {:0>4} {:0>11} {:0>6} {:0>11}   {:0>4}   {:0>2} {} {} {:0>4} ", 
            self.space_vehicule,
            self.class,
            //mjd //TODO,
            self.trktime.format("%H%M%S"),
            self.duration.as_secs(),
            (self.elevation * 10.0) as u16,
            (self.azimuth* 10.0) as u16,
            (self.refsv * 1.0E10) as i32,
            (self.srsv * 1.0E13) as i32,
            (self.refsys * 1.0E10) as i32,
            (self.srsys * 1.0E13) as i32,
            (self.dsg * 1.0E10) as i32,
            self.ioe,
            (self.mdtr * 1.0E10) as i32,
            (self.smdt * 1.0E13) as i32,
        ));
        
        if let Some(iono) = self.ionospheric {
            string.push_str(&format!("{} {} {} ", 
                (iono.msio * 1.0E10) as i32, 
                (iono.smsi * 1.0E13) as i32, 
                (iono.isg * 1.0E10) as i32, 
            ))
        }

        string.push_str(&format!("{} {:2X} {} ",
            self.fr,
            self.hc,
            self.frc));

        if let Ok(crc) = calc_crc(&string) {
            string.push_str(&format!("{:2X}", crc))
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
            (Option<f64>,Option<f64>,Option<f64>,u8,u8,String,u8) 
            = match items.len() {
                TRACK_WITHOUT_IONOSPHERIC => {
                    (None,None,None,
                    u8::from_str_radix(items[17], 16)?, 
                    u8::from_str_radix(items[18], 10)?,
                    items[19].to_string(),
                    u8::from_str_radix(items[20], 16)?)
                },
                TRACK_WITH_IONOSPHERIC => {
                    (Some(f64::from_str(items[17])? * 0.1E-9), 
                    Some(f64::from_str(items[18])? * 0.1E-12), 
                    Some(f64::from_str(items[19])? * 0.1E-9),
                    u8::from_str_radix(items[20], 16)?, 
                    u8::from_str_radix(items[21], 16)?,
                    items[22].to_string(),
                    u8::from_str_radix(items[23], 16)?)
                },
                PADDED_TRACK_WITHOUT_IONOSPHERIC_LENGTH => {
                    (None,None,None,
                    u8::from_str_radix(items[17+1], 16)?, 
                    u8::from_str(items[18+1])?,
                    items[19+1].to_string(),
                    u8::from_str_radix(items[20+1], 16)?)
                },
                PADDED_TRACK_WITH_IONOSPHERIC_LENGTH => {
                    (Some(f64::from_str(items[17+1])? * 0.1E-9), 
                    Some(f64::from_str(items[18+1])? * 0.1E-12), 
                    Some(f64::from_str(items[19+1])? * 0.1E-9),
                    u8::from_str_radix(items[20+1], 16)?, 
                    u8::from_str_radix(items[21+1], 16)?,
                    items[22+1].to_string(),
                    u8::from_str_radix(items[23+1], 16)?)
                },
                _ => return Err(Error::InvalidDataFormatError(String::from(cleanedup))),
        };

        // checksum field
        let mut cksum: u8 = 0;
        let end_pos = line.rfind(&format!("{:2X}",ck))
            .unwrap(); // already matching
        cksum = cksum.wrapping_add(
            calc_crc(
                &line.split_at(end_pos).0)?);
        // verification
        if cksum != ck {
            return Err(Error::ChecksumError(cksum, ck))
        }

        Ok(Track {
            class: {
                if class.eq("FF") {
                    CommonViewClass::Multiple
                } else {
                    CommonViewClass::Single
                }
            },
            space_vehicule: sv,
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
                    Some(IonosphericData {
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
