use regex::Regex;
use thiserror::Error;
use chrono::Timelike;

use crate::{CrcError, calc_crc};

/// `BIPM` tracking duration specifications.
/// `Cggtts` tracks must respect that duration
/// to be BIPM compliant, which is not mandatory 
pub const BIPM_SPECIFIED_TRACKING_DURATION: std::time::Duration = std::time::Duration::from_secs(13*60); 

/// labels in case we provide Ionospheric parameters estimates
pub const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
"SAT CL MJD STTIME TRKL ELV AZTH REFSV SRSV REFSYS SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

pub const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
"SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

/// Describes all known GNSS constellations
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Constellation {
    GPS,
    Glonass,
    Beidou,
    QZSS,
    Galileo,
    Mixed,
}

impl Default for Constellation {
    fn default() -> Constellation {
        Constellation::GPS
    }
}

impl std::fmt::Display for Constellation {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constellation::GPS => fmt.write_str("GPS"),
            Constellation::Glonass => fmt.write_str("GLO"),
            Constellation::Beidou => fmt.write_str("BDS"),
            Constellation::QZSS => fmt.write_str("QZS"),
            Constellation::Galileo => fmt.write_str("GAL"),
            _ => fmt.write_str("M"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConstellationError {
    #[error("unknown constellation '{0}'")]
    UnknownConstellation(String),
}

impl std::str::FromStr for Constellation {
    type Err = ConstellationError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("G") {
            Ok(Constellation::GPS)
        } else if s.starts_with("E") {
            Ok(Constellation::Galileo)
        } else if s.starts_with("R") {
            Ok(Constellation::Glonass)
        } else if s.starts_with("J") {
            Ok(Constellation::QZSS)
        } else if s.starts_with("C") {
            Ok(Constellation::Beidou)
        } else if s.starts_with("M") {
            Ok(Constellation::Mixed)
        } else {
            Err(ConstellationError::UnknownConstellation(s.to_string()))
        }
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
/// `CommonViewClassType` describes
/// whether this common view is based on a unique 
/// Satellite Vehicule, or a combination of SVs
enum CommonViewClassType {
    SingleFile,
    MultiFiles,
}

impl std::str::FromStr for CommonViewClassType {
    type Err = std::str::Utf8Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("FF") {
            Ok(CommonViewClassType::MultiFiles)
        } else {
            Ok(CommonViewClassType::SingleFile)
        }
    }
}

const TRACK_WITH_IONOSPHERIC_LENGTH           : usize = 24;
const PADDED_TRACK_WITH_IONOSPHERIC_LENGTH    : usize = 25;
const TRACK_WITHOUT_IONOSPHERIC_LENGTH        : usize = 21;
const PADDED_TRACK_WITHOUT_IONOSPHERIC_LENGTH : usize = 22;

#[derive(Debug, Clone)]
/// `CggttsTrack` describes a `Cggtts` measurement
pub struct CggttsTrack {
    constellation: Constellation,
    sat_id: u8,
    class: CommonViewClassType,
    trktime: chrono::NaiveTime, // Tracking start date (hh:mm:ss)
    duration: std::time::Duration, // Tracking duration
    elevation: f64, // Elevation (angle) at Tracking midpoint [in degrees]
    azimuth: f64, // Azimuth (angle) at Tracking midpoint [in degrees]
    refsv: f64,
    srsv: f64,
    refsys: f64,
    srsys: f64,
    // DSG [Data Sigma]
    // Root-mean-square of the residuals to linear fit from solution B in section 2.3.3
    dsg: f64,
    // IOE [Issue of Ephemeris]
    // Three-digit decimal code indicating the ephemeris used for the computation.
    // As no IOE is associated with the GLONASS navigation messages, the values 1-96 have to be
    // used to indicate the date of the ephemeris used, given by the number of the quarter of an hour in
    // the day, starting at 1=00h00m00s. For BeiDou, IOE will report the integer hour in the date of the
    // ephemeris (Time of Clock).
    ioe: u16,
    mdtr: f64, // Modeled tropospheric delay corresponding to the solution C in section 2.3.3
    smdt: f64, // Slope of the modeled tropospheric delay corresponding to the solution C in section 2.3.3
    mdio: f64, // Modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    smdi: f64, // Slope of the modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    msio: Option<f64>, // Measured ionospheric delay corresponding to the solution E in section 2.3.3.
    smsi: Option<f64>, // Slope of the measured ionospheric delay corresponding to the solution E in section 2.3.3.
    isg: Option<f64>, // [Ionospheric Sigma] Root-mean-square of the residuals corresponding to the solution E in section2.3.3
    fr: u8, // Glonass Channel Frequency [1:24], O for other GNSS
    hc: u8, // Receiver Hardware Channel [0:99], 0 if Unknown
    frc: ConstellationRinexCode 
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
    #[error("unknown gnss constellation")]
    ConstellationError(#[from] ConstellationError),
    #[error("unknown constellation rinex code \"{0}\"")]
    ConstellationRinexCodeError(#[from] ConstellationRinexCodeError),
    #[error("failed to parse common view class")]
    CommonViewClassError(#[from] std::str::Utf8Error),
    #[error("crc calc() failed over non utf8 data: \"{0}\"")]
    NonAsciiData(#[from] CrcError),
    #[error("checksum error - expecting \"{0}\" - got \"{1}\"")]
    ChecksumError(u8, u8),
}

impl CggttsTrack {
    /// Builds `CggttsTrack` object with
    /// default attributes
    pub fn new() -> CggttsTrack { Default::default() }

    /// Returns track start time
    pub fn get_start_time (&self) -> chrono::NaiveTime { self.trktime }
    /// Returns track duration
    pub fn get_duration (&self) -> std::time::Duration { self.duration }
    /// Assigns track duration
    pub fn set_duration (&mut self, duration: std::time::Duration) { self.duration = duration }

    /// Returns satellite vehicule ID (PRN#),
    /// returns 0xFF in case we're using a combination of SVs
    pub fn get_satellite_id (&self) -> u8 { self.sat_id }
    /// Assigns satellite vehicule ID (PRN#),
    /// set 0xFF when using a combination of SVs
    pub fn set_satellite_id (&mut self, id: u8) { self.sat_id = id }
    
    /// Returns elevation at tracking midpoint [degrees] 
    pub fn get_elevation (&self) -> f64 { self.elevation }
    /// Sets elevation at tracking midpoint [degrees] 
    pub fn set_elevation (&mut self, elevation: f64) { self.elevation = elevation }

    /// Returns azimuth angle [degrees] at tracking midpoint 
    pub fn get_azimuth (&self) -> f64 { self.azimuth }
    /// Sets azimuth angle [degrees] at tracking midpoint 
    pub fn set_azimuth (&mut self, azimuth: f64) { self.azimuth = azimuth }

    /// Returns constellation RINEX code
    pub fn get_constellation_rinex_code (&self) -> ConstellationRinexCode { self.frc }
    /// Assigns constellation RINEX code
    pub fn set_constellation_rinex_code (&mut self, code: ConstellationRinexCode) { self.frc = code }
    
    /// Returns track (refsv, srsv) duplet
    pub fn get_refsv_srsv (&self) -> (f64, f64) { (self.refsv, self.srsv) }
    /// Assigns track (refsv, srsv) duplet
    pub fn set_refsv_srsv (&mut self, data: (f64, f64)) { 
        self.refsv = data.0;
        self.srsv = data.1
    }

    /// Returns track (refsys, srsys) duplet 
    pub fn get_refsys_srsys (&self) -> (f64, f64) { (self.refsys, self.srsys) }
    /// Assigns track (refsys, srsys) duplet
    pub fn set_refsys_srsys (&mut self, data: (f64, f64)) { 
        self.refsys = data.0;
        self.srsys = data.1
    }

    /// Returns true if track comes with ionospheric parameters estimates
    pub fn has_ionospheric_parameters (&self) -> bool { self.msio.is_some() && self.smsi.is_some() && self.isg.is_some() }
    
    /// Returns ionospheric parameters estimates (if any)
    pub fn get_ionospheric_parameters (&self) -> Option<(f64, f64, f64)> {
        if self.has_ionospheric_parameters() {
            Some((self.msio.unwrap(),self.smsi.unwrap(),self.isg.unwrap()))
        } else {
            None
        }
    }
    
    /// Assigns ionospheric parameters
    /// params (MSIO, SMSI, ISG)
    pub fn set_ionospheric_parameters (&mut self, params: (f64,f64,f64)) {
        self.msio = Some(params.0);
        self.smsi = Some(params.1);
        self.isg = Some(params.2)
    }
}

impl Default for CggttsTrack {
    /// Builds default `CggttsTrack` structure
    fn default() -> CggttsTrack {
        let now = chrono::Utc::now();
        let msio: Option<f64> = None;
        let smsi: Option<f64> = None;
        let isg: Option<f64> = None;
        CggttsTrack {
            constellation: Constellation::GPS,
            sat_id: 0,
            class: CommonViewClassType::SingleFile,
            trktime: chrono::NaiveTime::from_hms(
                now.time().hour(),
                now.time().minute(),
                now.time().second()
            ),
            duration: std::time::Duration::from_secs(0),
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
            msio,
            smsi,
            isg,
            fr: 0,
            hc: 0,
            frc: ConstellationRinexCode::GPS_GLO_QZ_SBA_L1C,
        }
    }
}

impl std::fmt::Display for CggttsTrack {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::approx_eq;
    
    #[test]
    /// Tests `CggttsTrack` default constructor
    fn cggtts_track_default() {
        let track = CggttsTrack::new();
        assert_eq!(track.get_duration().as_secs(), 0);
        assert_eq!(track.get_elevation(), 0.0);
        assert_eq!(track.get_azimuth(), 0.0);
        assert_eq!(track.get_refsv_srsv(), (0.0,0.0));
        assert_eq!(track.get_refsys_srsys(), (0.0,0.0));
        assert_eq!(track.has_ionospheric_parameters(), false);
        assert_eq!(track.get_ionospheric_parameters().is_none(), true); // missing params
        assert_eq!(track.get_constellation_rinex_code(), ConstellationRinexCode::GPS_GLO_QZ_SBA_L1C); // missing params
    }

    #[test]
    /// Tests `CggttsTrack` basic usage
    fn cggtts_track_basic_use() {
        let mut track = CggttsTrack::new();
        track.set_duration(BIPM_SPECIFIED_TRACKING_DURATION);
        track.set_elevation(90.0);
        track.set_azimuth(180.0);
        track.set_refsys_srsys((1E-9,1E-12));
        assert_eq!(track.get_duration().as_secs(), BIPM_SPECIFIED_TRACKING_DURATION.as_secs());
        assert_eq!(track.get_elevation(), 90.0);
        assert_eq!(track.get_azimuth(), 180.0);
        assert!(
            approx_eq!(f64,
                track.get_refsv_srsv().0, 
                1E-9,
                epsilon = 1E-9
            )
        );
        assert!(
            approx_eq!(f64,
                track.get_refsv_srsv().1, 
                1E-12,
                epsilon = 1E-12
            )
        )
    }
}
