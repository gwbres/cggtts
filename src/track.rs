use regex::Regex;
use chrono::Timelike;
use thiserror::Error;
use rinex::{Constellation, Sv};
//use crate::{CrcError, calc_crc};

/// `BIPM` tracking duration specifications.
/// `Cggtts` tracks must respect that duration
/// to be BIPM compliant, which is not mandatory 
const BIPM_SPECIFIED_DURATION: std::time::Duration = std::time::Duration::from_secs(13*60); 

/// labels in case we provide Ionospheric parameters estimates
const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
"SAT CL MJD STTIME TRKL ELV AZTH REFSV SRSV REFSYS SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
"SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

#[derive(PartialEq, Clone, Copy, Debug)]
/// Describes whether this common view is based on a unique 
/// Space Vehicule or a combination of several vehicules
pub enum CommonViewClass {
    /// Single Space Vehicule used in measurement
    Single(Sv),
    /// Multiple Space Vehicules from the same constellation
    /// used in measurement
    Combination(Constellation),
}

/// Describes Glonass Frequency channel,
/// Constellation codes, refer to
/// `RINEX` denominations
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConstellationRinexCode {
    GPS_GLO_QZ_SBA_L1C,
    GPS_GLO_L1P,
    GAL_E1,
    QZSS_L1C,
    BEIDOU_B1i,
    GPS_C1_P1C2_P2,
    GAL_E1E5a,
    BEIDOU_BliB2i,
    GLO_C1_P1C2_P2,
    GZSS_C1C5,
    NonSupportedCode,
}

impl std::fmt::Display for ConstellationRinexCode {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            ConstellationRinexCode::GPS_GLO_QZ_SBA_L1C => String::from("L1C"),
            ConstellationRinexCode::GPS_GLO_L1P => String::from("L1P"),
            ConstellationRinexCode::GAL_E1 => String::from("E1C"),
            ConstellationRinexCode::GAL_E1E5a => String::from("E5C"),
            ConstellationRinexCode::QZSS_L1C => String::from("Q1C"),
            ConstellationRinexCode::BEIDOU_B1i => String::from("B1C"),
            ConstellationRinexCode::BEIDOU_BliB2i => String::from("B1C"),
            ConstellationRinexCode::GPS_C1_P1C2_P2 => String::from("P1C"),
            ConstellationRinexCode::GLO_C1_P1C2_P2 => String::from("Q5C"),
            ConstellationRinexCode::GZSS_C1C5 => String::from("Q5C"),
            ConstellationRinexCode::NonSupportedCode => String::from("???"),
        };
        fmt.write_str(&s)
    }
}

#[derive(Error, Debug)]
pub enum ConstellationRinexCodeError {
    #[error("unknown constellation code '{0}'")]
    UnknownCode(String),
}

impl std::str::FromStr for ConstellationRinexCode {
    type Err = ConstellationRinexCodeError;   
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("L1C") {
            Ok(ConstellationRinexCode::GPS_GLO_QZ_SBA_L1C)
        } else if s.eq("L1P") {
            Ok(ConstellationRinexCode::GPS_GLO_L1P)
        } else if s.eq("E1") {
            Ok(ConstellationRinexCode::GAL_E1)
        } else if s.eq("L1C") {
            Ok(ConstellationRinexCode::QZSS_L1C)
        } else if s.eq("B1i") {
            Ok(ConstellationRinexCode::BEIDOU_B1i)
        } else if s.eq("L3P") {
            Ok(ConstellationRinexCode::GPS_C1_P1C2_P2)
        } else if s.eq("L3E") {
            Ok(ConstellationRinexCode::GPS_C1_P1C2_P2)
        } else if s.eq("L3B") {
            Ok(ConstellationRinexCode::GAL_E1E5a)
        } else if s.eq("L3P") {
            Ok(ConstellationRinexCode::BEIDOU_BliB2i)
        } else if s.eq("L3Q") {
            Ok(ConstellationRinexCode::GZSS_C1C5)
        } else {
            Err(ConstellationRinexCodeError::UnknownCode(s.to_string()))
        }
    }
}

/// in case this `Track` was esimated using Glonass
#[derive(Debug, Copy, Clone)]
pub enum GlonassChannel {
    /// Other, default value when not using Glonass constellation
    Other,
    /// Glonass Frequency channel number
    Channel(u8),
}

impl PartialEq for GlonassChannel {
    fn eq (&self, other: &Self) -> bool {
        match self {
            GlonassChannel::Other => {
                match other {
                    GlonassChannel::Other => true,
                    _ => false
                }
            },
            GlonassChannel::Channel(c0) => {
                match other {
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
    /// Default Glonass Channel Other/Unused
    fn default() -> Self {
        Self::Other
    }
}

#[repr(usize)]
enum StandardTrackLength {
    WithoutIonospheric = 21,
    PaddedWithoutIonospheric = 22,
    WithIonospheric = 24,
    PaddedWithIonospheric = 25,
}

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
    pub space_vehicule: Option<rinex::Sv>,
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
    /// Constellation Carrier RINEX code
    pub frc: ConstellationRinexCode, 
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
    #[error("failed to parse int number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse track time")]
    ChronoParseError(#[from] chrono::ParseError),
   // #[error("unknown gnss constellation")]
   // ConstellationError(#[from] ConstellationError),
    #[error("unknown constellation rinex code \"{0}\"")]
    ConstellationRinexCodeError(#[from] ConstellationRinexCodeError),
    #[error("failed to parse common view class")]
    CommonViewClassError(#[from] std::str::Utf8Error),
    //#[error("crc calc() failed over non utf8 data: \"{0}\"")]
    //NonAsciiData(#[from] CrcError),
    #[error("checksum error - expecting \"{0}\" - got \"{1}\"")]
    ChecksumError(u8, u8),
}

impl Track {
    /// Builds a new CGGTTS measurement, referred to as `Track`,
    /// without known Ionospheric parameters. 
    /// To add Ionospheric data, use `with_ionospheric_data()` later on
    pub fn new (class: CommonViewClass,
            trktime: chrono::NaiveTime, duration: std::time::Duration,
                space_vehicule: Option<rinex::Sv>,
                elevation: f64, azimuth: f64, refsv: f64, srsv: f64,
                    refsys: f64, srsys:f64, dsg: f64, ioe: u16, mdtr: f64,
                        smdt: f64, mdio: f64, smdi: f64, fr: GlonassChannel,
                            hc: u8, frc: ConstellationRinexCode) -> Self {
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
            frc,
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
    /// of Space Vehicules
    pub fn space_vehicule_combination (&self) -> bool {
        !self.unique_space_vehicule()
    }

    /// Returns true if Self was measured against a unique
    /// Space Vehicule
    pub fn unique_space_vehicule (&self) -> bool {
        match self.class {
            CommonViewClass::Single(_) => true,
            _ => false,
        }
    }

    /// Returns true if Self was measured against given `GNSS` Constellation 
    pub fn uses_constellation (&self, c: Constellation) -> bool {
        match self.class {
            CommonViewClass::Single(s) => {
                s.constellation == c
            },
            CommonViewClass::Combination(constell) => {
                constell == c
            },
        }
    }

    /// Returns true if Self was measured against specific Space Vehicule uniquely
    pub fn uses_unique_space_vehicule (&self, sv: Sv) -> bool {
        match self.class {
            CommonViewClass::Single(s) => {
                s == sv
            },
            _ => false,
        }
    }

    /// Returns True if Self follows BIPM specifications / requirements,
    /// * track pursuit duration was at least 13 * 60 sec long
    pub fn follows_bipm_specs (&self) -> bool {
        self.duration.as_secs() >= BIPM_SPECIFIED_DURATION.as_secs()
    }
    
    /// Returns a `Track` with desired unique space vehicule
    pub fn with_space_vehicule (&self, sv: Sv) -> Self {
        let mut t = self.clone();
        t.space_vehicule = Some(sv);
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

    /// Returns a `Track` with desired Constellation RINEX code
    pub fn with_rinex_code (&self, code: ConstellationRinexCode) -> Self {
        let mut t = self.clone();
        t.frc = code;
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
    /// common view is set to a combination of GPS space vehicules,
    /// and other parameters set to defaults,
    /// and missing Ionospheric parameter estimates.
    fn default() -> Track {
        let now = chrono::Utc::now();
        Track {
            space_vehicule: None,
            class: CommonViewClass::Combination(Constellation::default()),
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
            frc: ConstellationRinexCode::GPS_GLO_QZ_SBA_L1C,
        }
    }
}

/*
impl std::fmt::Display for Track {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = String::new();
        match self.constellation {
            Constellation::GPS => string.push_str("G"),
            Constellation::Glonass => string.push_str("R"),
            Constellation::Beidou => string.push_str("B"),
            Constellation::QZSS => string.push_str("Q"),
            Constellation::Galileo => string.push_str("E"),
            _ => string.push_str("M"),
        }
        string.push_str(&format!("{: >2}", self.sat_id));
        string.push_str(&format!(" {:02X}", self.class as u8));
        string.push_str(" 57000");

        // self.trktime.format("%H%M%S");
        // let fmt = StrftimeItems::new("%dH%M%S")
        //string.push_str(self.trktime.strptime());
        string.push_str(&format!(" {:0>3}", self.duration.as_secs()));
        string.push_str(&format!(" {:0>3}", (self.elevation * 10.0) as u16));
        string.push_str(&format!(" {:0>3}", (self.azimuth * 10.0) as u16));
        string.push_str(&format!(" {}", (self.refsv  * 1.0E10) as i32));
        string.push_str(&format!(" {}", (self.srsv   * 1.0E13) as i32));
        string.push_str(&format!(" {}", (self.refsys * 1.0E10) as i32));
        string.push_str(&format!(" {}", (self.srsys  * 1.0E13) as i32));
        string.push_str(&format!(" {}", (self.dsg    * 1.0E10) as i32));
        string.push_str(&format!(" {}", self.ioe));
        string.push_str(&format!(" {}", (self.mdtr   * 1.0E10) as i32));
        string.push_str(&format!(" {}", (self.smdt   * 1.0E13) as i32));
        string.push_str(&format!(" {}", (self.mdio   * 1.0E10) as i32));
        string.push_str(&format!(" {}", (self.smdi   * 1.0E13) as i32));
        
        if let Some((msio, smsi, isg)) = self.get_ionospheric_parameters() {
            string.push_str(&format!(" {}", (msio * 1.0E10) as i32));
            string.push_str(&format!(" {}", (smsi * 1.0E13) as i32));
            string.push_str(&format!(" {}", (isg *  1.0E10) as i32))       
        }
        string.push_str(&format!(" {:2X}", self.fr));
        string.push_str(&format!(" {:2X}", self.hc));
        string.push_str(&format!(" {}", self.frc));
        if let Ok(crc) = calc_crc(&string) {
            string.push_str(&format!(" {:2X}", crc))
        } 
        fmt.write_str(&string)
    }
}

impl std::str::FromStr for CggttsTrack {
    type Err = Error; 
    /// Builds `CggttsTrack` from given str content
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let cleanedup = String::from(line.trim());
        let items: Vec<&str> = cleanedup.split_ascii_whitespace().collect();
        // checking content validity
        let content_is_valid = Regex::new(r"^(G|R|E|J|C) \d")
            .unwrap();
        let content_is_valid2 = Regex::new(r"^(G|R|E|J|C)\d\d")
            .unwrap();
        match content_is_valid.is_match(&cleanedup) {
            false => {
                match content_is_valid2.is_match(&cleanedup) {
                    false => return Err(Error::InvalidDataFormatError(String::from(cleanedup))),
                    _ => {},
                }
            },
            _ => {},
        };

        // sat # prn is right padded
        let is_single_digit_prn = Regex::new(r"^. \d")
            .unwrap();
        let offset : usize = match is_single_digit_prn.is_match(&cleanedup) { 
            true => 1,
            false => 0,
        };
        if items.len() != TRACK_WITH_IONOSPHERIC_LENGTH+offset {
            if items.len() != TRACK_WITHOUT_IONOSPHERIC_LENGTH+offset {
                return Err(Error::InvalidDataFormatError(String::from(cleanedup)))
            }
        }

        let constellation = Constellation::from_str(items.get(0)
            .unwrap())?;
        let (_, sat_id) = items.get(0).unwrap_or(&"").split_at(1);
        let class = CommonViewClassType::from_str(items.get(1+offset).unwrap_or(&""))?;
        let trktime = chrono::NaiveTime::parse_from_str(items.get(3+offset).unwrap_or(&""), "%H%M%S")?;
        let duration_secs = u64::from_str_radix(items.get(4+offset).unwrap_or(&""), 10)?;
        let elevation = f64::from_str(items.get(5+offset).unwrap_or(&""))? * 0.1;
        let azimuth = f64::from_str(items.get(6+offset).unwrap_or(&""))? * 0.1;
        let refsv = f64::from_str(items.get(7+offset).unwrap_or(&""))? * 0.1E-9;
        let srsv = f64::from_str(items.get(8+offset).unwrap_or(&""))? * 0.1E-12;
        let refsys = f64::from_str(items.get(9+offset).unwrap_or(&""))? * 0.1E-9;
        let srsys = f64::from_str(items.get(10+offset).unwrap_or(&""))? * 0.1E-12;
        let dsg = f64::from_str(items.get(11+offset).unwrap_or(&""))? * 0.1E-9;
        let ioe = u16::from_str_radix(items.get(12+offset).unwrap_or(&""), 10)?;
        let mdtr = f64::from_str(items.get(13+offset).unwrap_or(&""))? * 0.1E-9;
        let smdt = f64::from_str(items.get(14+offset).unwrap_or(&""))? * 0.1E-12;
        let mdio = f64::from_str(items.get(15+offset).unwrap_or(&""))? * 0.1E-9;
        let smdi = f64::from_str(items.get(16+offset).unwrap_or(&""))? * 0.1E-12;

        let (msio, smsi, isg, fr, hc, frc, ck) : 
            (Option<f64>,Option<f64>,Option<f64>,u8,u8,ConstellationRinexCode,u8) 
            = match items.len() {
                TRACK_WITHOUT_IONOSPHERIC_LENGTH => {
                    (None,None,None,
                    u8::from_str_radix(items.get(17).unwrap_or(&""), 16)?, 
                    u8::from_str(items.get(18).unwrap_or(&""))?,
                    ConstellationRinexCode::from_str(items.get(19).unwrap_or(&""))?,
                    u8::from_str_radix(items.get(20).unwrap_or(&""), 16)?)
                },
                TRACK_WITH_IONOSPHERIC_LENGTH => {
                    (Some(f64::from_str(items.get(17).unwrap_or(&""))? * 0.1E-9), 
                    Some(f64::from_str(items.get(18).unwrap_or(&""))? * 0.1E-12), 
                    Some(f64::from_str(items.get(19).unwrap_or(&""))? * 0.1E-9),
                    u8::from_str_radix(items.get(20).unwrap_or(&""), 16)?, 
                    u8::from_str_radix(items.get(21).unwrap_or(&""), 16)?,
                    ConstellationRinexCode::from_str(items.get(22).unwrap_or(&""))?,
                    u8::from_str_radix(items.get(23).unwrap_or(&""), 16)?)
                },
                PADDED_TRACK_WITHOUT_IONOSPHERIC_LENGTH => {
                    (None,None,None,
                    u8::from_str_radix(items.get(17+1).unwrap_or(&""), 16)?, 
                    u8::from_str(items.get(18+1).unwrap_or(&""))?,
                    ConstellationRinexCode::from_str(items.get(19+1).unwrap_or(&""))?,
                    u8::from_str_radix(items.get(20+1).unwrap_or(&""), 16)?)
                },
                PADDED_TRACK_WITH_IONOSPHERIC_LENGTH => {
                    (Some(f64::from_str(items.get(17+1).unwrap_or(&""))? * 0.1E-9), 
                    Some(f64::from_str(items.get(18+1).unwrap_or(&""))? * 0.1E-12), 
                    Some(f64::from_str(items.get(19+1).unwrap_or(&""))? * 0.1E-9),
                    u8::from_str_radix(items.get(20+1).unwrap_or(&""), 16)?, 
                    u8::from_str_radix(items.get(21+1).unwrap_or(&""), 16)?,
                    ConstellationRinexCode::from_str(items.get(22+1).unwrap_or(&""))?,
                    u8::from_str_radix(items.get(23+1).unwrap_or(&""), 16)?)
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

        Ok(CggttsTrack {
            constellation,
            sat_id: u8::from_str_radix(sat_id, 10).unwrap_or(0),
            class,
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
            msio,
            smsi,
            isg,
            fr,
            hc,
            frc
        })
    }
}*/
