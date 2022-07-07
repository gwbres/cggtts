//! CGGTTS is the core structure, it comprises
//! the list of tracks (measurements) and some header information.
//!
//! # Example
//! ```
//! use cggtts::Cggtts;
//! fn main() {
//!     let cggtts = Cggtts::from_file("../data/standard/GZSY8259.506")
//!         .unwrap();
//!     assert_eq!(cggtts.lab, Some(String::from("SY82")));
//!     assert_eq!(cggtts.follows_bipm_specs(), true);
//!     if let Some(track) = cggtts.tracks.first() {
//!         let duration = track.duration;
//!         let (refsys, srsys) = (track.refsys, track.srsys);
//!         assert_eq!(track.has_ionospheric_data(), false);
//!         assert_eq!(track.follows_bipm_specs(), true);
//!     }
//! }
//! ```
//!
//! # Advanced CGGTTS
//! Comes with ionospheric parameters estimates
//!
//!```
//! use cggtts::Cggtts;
//! fn main() {
//!     let cggtts = Cggtts::from_file("../data/advanced/RZSY8257.000")
//!         .unwrap();
//!     if let Some(track) = cggtts.tracks.first() {
//!         assert_eq!(track.has_ionospheric_data(), true);
//!         if let Some(iono) = track.ionospheric {
//!             let (msio, smsi, isg) = (iono.msio, iono.smsi, iono.isg);
//!         }
//!     }
//! }
//!```
//!
//! # CGGTTS production
//! Use `to_string` to dump CGGTTS data
//!
//! ```
//! use cggtts::{Rcvr, Cggtts, Track};
//! use rinex::constellation::Constellation;
//! use std::io::Write;
//! fn main() {
//!     let nb_channels = 16;
//!     let hardware = Rcvr {
//!         manufacturer: String::from("GoodManufacturer"),
//!         recv_type: String::from("Unknown"),
//!         serial_number: String::from("1234"),
//!         year: 2022,
//!         release: String::from("V1"),
//!     };
//!     let mut cggtts = Cggtts::new(Some("MyAgency"), nb_channels, Some(hardware));
//!     // add some tracks
//!     // CGGTTS says we should set "99" as PRN when data
//!     // is estimated from several space vehicules
//!     let sv = rinex::sv::Sv {
//!         constellation: Constellation::GPS,
//!         prn: 99,
//!     };
//!     let mut track = Track::default();
//!     cggtts.tracks.push(track);
//!     let mut fd = std::fs::File::create("output.cggtts") // does not respect naming conventions
//!         .unwrap();
//!     write!(fd, "{}", cggtts).unwrap();
//! }
//! ```
//!
//! To produced advanced CGGTTS data correctly, one should specify / provide
//! - secondary hardware info [IMS]
//! - ionospheric parameter estimates
//! - specify carrier dependent delays [see Delay]

use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
use strum_macros::{EnumString};
use rinex::constellation::Constellation;

use crate::{Track, Delay, delay::SystemDelay};

/// Supported `Cggtts` version,
/// non matching input files will be rejected
const CURRENT_RELEASE: &str = "2E";

/// Latest revision date
const LATEST_REVISION_DATE : &str = "2014-02-20";

/// labels in case we provide Ionospheric parameters estimates
const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
"SAT CL MJD STTIME TRKL ELV AZTH REFSV SRSV REFSYS SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
"SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

/*
pub struct ITRF {
    x: f64,
    y: f64,
    z: f64,
    epoch: chrono::NaiveDateTime,
}

pub struct HelmertCoefs {
    /// translation coefficients (cx, cy, cz) in meters
    c: (f64, f64, f64),
    /// scaling [ppb]
    s: f64,
    /// rotation matrix coefficients (rx, ry, rz) in arc second
    r: (f64, f64, f64),
}

/// Transforms input 3D vector using Helmert transforms.
/// Used to convert ITRF and other data
pub fn helmert_transform (v: (f64,f64,f64), h: HelmertCoefs) -> (f64,f64,f64)  {
    let (x, y, z) = v;
    let (cx, cy, cz) = h.c;
    let (rx, ry, rz) = h.z;
    let x = cx + (1.0 * h.s*1E-9) * (x - rz*y + ry * z);
    let y = cy + (1.0 * h.s*1E-9) * (rz*x + y -rx*z);
    let z = cz + (1.0 * h.s*1E-9) * (-ry*x +rx*y + z);
}*/

/// `Rcvr` describes a GNSS receiver
/// (hardware). Used to describe the
/// GNSS receiver or hardware used to evaluate IMS parameters
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Manufacturer of this hardware
    pub manufacturer: String,
    /// Type of receiver
    pub recv_type: String,
    /// Receiver's serial number
    pub serial_number: String,
    /// Receiver manufacturing year
    pub year: u16,
    /// Receiver software revision number
    pub release: String,
}

/// Known Reference Time Systems
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub enum TimeSystem {
    /// TAI: International Atomic Time
    TAI,
    /// UTC: Universal Coordinate Time
    UTC,
    /// UTC(k): Laboratory local official
    /// UTC image, agency name
    /// and optionnal |offset| to universal UTC
    /// in nanoseconds
    UTCk(String, Option<f64>),
    /// Unknown Time system
    Unknown(String),
}

impl Default for TimeSystem {
    fn default() -> TimeSystem {
        TimeSystem::UTC
    }
}

impl TimeSystem {
    pub fn from_str(s: &str) -> TimeSystem {
        if s.eq("TAI") {
            TimeSystem::TAI
        } else if s.contains("UTC") {
            // UTCk with lab + offset
            if let (Some(lab), Some(offset)) = scan_fmt!(s, "UTC({},{})", String, f64) {
                TimeSystem::UTCk(lab, Some(offset))
            } 
            // UTCk with only agency name
            else if let Some(lab) = scan_fmt!(s, "UTC({})", String) {
                TimeSystem::UTCk(lab, None)
            } else {
                TimeSystem::UTC
            }
        } else {
            TimeSystem::Unknown(s.to_string()) 
        }
    }
}

impl std::fmt::Display for TimeSystem {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TimeSystem::TAI => fmt.write_str("TAI"),
            TimeSystem::UTC => fmt.write_str("UTC"),
            TimeSystem::UTCk(lab, _) => write!(fmt, "UTC({})", lab),
            TimeSystem::Unknown(s) => fmt.write_str(s),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[derive(EnumString)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
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

#[derive(Error, Debug)]
pub enum CrcError {
    #[error("failed to compute CRC over non utf8 data")] 
    NonAsciiData(String),
}

/// computes crc for given str content
pub fn calc_crc (content: &str) -> Result<u8, CrcError> {
    match content.is_ascii() {
        true => {
            let mut ck: u8 = 0;
            let mut ptr = content.encode_utf16();
            for _ in 0..ptr.clone().count() {
                ck = ck.wrapping_add(
                    ptr.next()
                    .unwrap()
                    as u8)
            }
            Ok(ck)
        },
        false => return Err(CrcError::NonAsciiData(String::from(content))),
    }
}

impl std::fmt::Display for Rcvr { 
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.manufacturer)?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.recv_type)?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.serial_number)?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.year.to_string())?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.release)?;
        Ok(())
    }
}

/// `Cggtts` structure comprises
/// a measurement system and 
/// and its Common View realizations (`tracks`)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub struct Cggtts {
    /// file revision release date 
    pub rev_date: chrono::NaiveDate, 
    /// laboratory / agency where measurements were performed (if unknown)
    pub lab: Option<String>, 
    /// possible GNSS receiver infos
    pub rcvr: Option<Rcvr>, 
    /// nb of GNSS receiver channels
    pub nb_channels: u16, 
    /// IMS Ionospheric Measurement System (if any)
    pub ims: Option<Rcvr>, 
    /// Description of Reference time system (if any)
    pub time_reference: TimeSystem, 
    /// Reference frame, coordinates system and conversions,
    /// used in `coordinates` field
    pub reference_frame: Option<String>,
    /// Antenna phase center coordinates [m]
    /// in `ITFR`, `ECEF` or other spatial systems
    pub coordinates: rust_3d::Point3D, 
    /// Comments (if any..)
    pub comments: Option<Vec<String>>, 
    /// Describes the measurement systems delay.
    /// Refer to [Delay] enum,
    /// to understand their meaning.
    /// Refer to [SystemDelay] and [CalibratedDelay] to understand
    /// how to specify the measurement systems delay.
    pub delay: SystemDelay, 
    /// Tracks: actual measurements / realizations
    pub tracks: Vec<Track>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse file")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("only revision 2E is supported")]
    VersionMismatch,
    #[error("version format mismatch")]
    VersionFormatError,
    #[error("revision date format mismatch")]
    RevisionDateFormatError,
    #[error("failed to parse revision date")]
    RevisionDateParsingError,
    #[error("failed to parse \"{0}\" coordinates")]
    CoordinatesParsingError(String),
    #[error("failed to identify delay value in line \"{0}\"")]
    DelayIdentificationError(String),
    #[error("failed to parse frequency dependent delay from \"{0}\"")]
    FrequencyDependentDelayParsingError(String),
    #[error("checksum format error")]
    ChecksumFormatError,
    #[error("failed to parse checksum value")]
    ChecksumParsingError,
    #[error("crc calc() failed over non utf8 data: \"{0}\"")]
    NonAsciiData(#[from] CrcError),
    #[error("checksum error, got \"{0}\" but \"{1}\" locally computed")]
    ChecksumError(u8, u8),
}

impl Default for Cggtts {
    /// Buils default `Cggtts` structure,
    fn default() -> Cggtts {
        Cggtts {
            rev_date: chrono::NaiveDate::parse_from_str(
                LATEST_REVISION_DATE,
                "%Y-%m-%d").unwrap(),
            lab: None,
            nb_channels: 0,
            coordinates: rust_3d::Point3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rcvr: None,
            tracks: Vec::new(),
            ims: None, 
            reference_frame: None,
            time_reference: TimeSystem::default(),
            comments: None,
            delay: SystemDelay::new(), 
        }
    }
}

impl Cggtts {
    /// Builds `Cggtts` object with desired attributes.
    /// Date is set to `now` by default, use
    /// `with_date()` to customize.
    pub fn new (lab: Option<&str>, nb_channels: u16, rcvr: Option<Rcvr>) -> Self { 
        let mut c = Self::default();
        if let Some(lab) = lab {
            c = c.with_lab_agency(lab);
        }
        c = c.with_nb_channels(nb_channels);
        if let Some(rcvr) = rcvr {
            c = c.with_receiver(rcvr);
        }
        c
    }

    /// Returns true if all tracks follow 
    /// BIPM tracking specifications
    pub fn follows_bipm_specs (&self) -> bool {
        for track in self.tracks.iter() {
            if !track.follows_bipm_specs() {
                return false
            }
        }
        true
    }
    
    /// Returns `Cggtts` with same attributes
    /// but desired `Lab` agency
    pub fn with_lab_agency (&self, lab: &str) -> Self { 
        let mut c = self.clone();
        c.lab = Some(lab.to_string());
        c
    }
    
    /// Returns ̀`Cggtts` with desired number of channels
    pub fn with_nb_channels (&self, ch: u16) -> Self { 
        let mut c = self.clone();
        c.nb_channels = ch;
        c
    }

    /// Returns `Cggtts` with desired Receiver infos
    pub fn with_receiver (&self, rcvr: Rcvr) -> Self { 
        let mut c = self.clone();
        c.rcvr = Some(rcvr);
        c
    }

    /// Returns `Cggtts` with desired `IMS` evaluation
    /// hardware infos
    pub fn with_ims_infos (&self, ims: Rcvr) -> Self { 
        let mut c = self.clone();
        c.ims = Some(ims);
        c
    }

    /// Returns `cggtts` but with desired antenna phase center
    /// coordinates, coordinates should be in `IRTF` reference system,
    /// and expressed in meter.
    pub fn with_antenna_coordinates (&self, coords: rust_3d::Point3D) -> Self {
        let mut c = self.clone();
        c.coordinates = coords;
        c
    }
    
    /// Returns `Cggtts` with desired reference time system description 
    pub fn with_time_reference (&self, reference: TimeSystem) -> Self { 
        let mut c = self.clone();
        c.time_reference = reference;
        c
    }

    /// Returns `Cggtts` with desired Reference Frame 
    pub fn with_reference_frame (&self, reference: &str) -> Self {
        let mut c = self.clone();
        c.reference_frame = Some(reference.to_string());
        c
    }

    /// Returns true if Self only contains tracks (measurements)
    /// that have ionospheric parameter estimates
    pub fn has_ionospheric_data (&self) -> bool {
        for track in self.tracks.iter() {
            if !track.has_ionospheric_data() {
                return false
            }
        }
        true
    }

    /// Returns production date (y/m/d) of this file
    /// using MJD field of first track produced
    pub fn date (&self) -> Option<chrono::NaiveDate> {
        if let Some(t) = self.tracks.first() {
            Some(t.date)
        } else {
            None
        }
    }

    /// Returns total set duration,
    /// by cummulating all measurements duration
    pub fn total_duration (&self) -> std::time::Duration {
        let mut s = 0;
        for t in self.tracks.iter() {
            s += t.duration.as_secs()
        }
        std::time::Duration::from_secs(s)
    }

    /// Returns a filename that would match
    /// specifications / standard requirements
    /// to represent self
    pub fn filename (&self) -> String {
        let mut res = String::new();
        res
    }

    /// Builds Self from given `Cggtts` file.
    pub fn from_file (fp: &str) -> Result<Self, Error> {
        let file_content = std::fs::read_to_string(fp)
            .unwrap();
        let mut lines = file_content.split("\n")
            .map(|x| x.to_string())
            //.map(|x| x.to_string() +"\n")
            //.map(|x| x.to_string() +"\r"+"\n")
                .into_iter();
        
        // init variables
        let mut line = lines.next()
            .unwrap();
        let mut system_delay = SystemDelay::new();
        
        // VERSION must be first
        let _ = match scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => {
                if !version.eq(&CURRENT_RELEASE) {
                    return Err(Error::VersionMismatch)
                }
            },
            _ => return Err(Error::VersionFormatError),
        };
        
        let mut _cksum :u8 = calc_crc(&line)?;
        
        let mut rev_date = chrono::NaiveDate::parse_from_str(LATEST_REVISION_DATE, "%Y-%m-%d")
            .unwrap();
        let mut nb_channels :u16 = 0;
        let mut rcvr : Option<Rcvr> = None;
        let mut ims : Option<Rcvr> = None;
        let mut lab: Option<String> = None;
        let mut comments: Vec<String> = Vec::new();
        let mut reference_frame: Option<String> = None;
        let mut time_reference = TimeSystem::default(); 
        let (mut x, mut y, mut z) : (f64,f64,f64) = (0.0, 0.0, 0.0);

        line = lines.next()
            .unwrap();

        loop {
            if line.starts_with("REV DATE = ") {
                match scan_fmt! (&line, "REV DATE = {d} {d} {d}", i32, u32, u32) {
                    (Some(year),
                    Some(month),
                    Some(day)) => {
                        rev_date = chrono::NaiveDate::from_ymd(year, month, day);
                    },
                    _ => {}
                }
            } else if line.starts_with("RCVR = ") {
                match scan_fmt! (&line, "RCVR = {} {} {} {d} {}", String, String, String, String, String) {
                    (Some(manufacturer),
                    Some(recv_type),
                    Some(serial_number),
                    Some(year),
                    Some(release)) => {
                        rcvr = Some(Rcvr {
                            manufacturer, 
                            recv_type, 
                            serial_number, 
                            year: u16::from_str_radix(&year, 10)?, 
                            release, 
                        })
                    },
                    _ => {}
                }

            } else if line.starts_with("CH = ") {
                match scan_fmt!(&line, "CH = {d}", u16) {
                    Some(n) => nb_channels = n,
                    _ => {} 
                };

            } else if line.starts_with("IMS = ") {
                match scan_fmt!(&line, "IMS = {} {} {} {d} {}", String, String, String, String, String) {
                    (Some(manufacturer),
                    Some(recv_type),
                    Some(serial_number),
                    Some(year),
                    Some(release)) => { 
                        ims = Some(Rcvr {
                            manufacturer, 
                            recv_type, 
                            serial_number, 
                            year: u16::from_str_radix(&year, 10)?, 
                            release,
                        })
                    },
                    _ => {}, 
                }
            
            } else if line.starts_with("LAB = ") {
                match line.strip_prefix("LAB = ") {
                    Some(s) => {
                        lab = Some(String::from(s.trim()))
                    },
                    _ => {},
                }
            } else if line.starts_with("X = ") {
                match scan_fmt!(&line, "X = {f}", f64) {
                    Some(f) => {
                        x = f
                    },
                    _ => {},
                }
            } else if line.starts_with("Y = ") {
                match scan_fmt!(&line, "Y = {f}", f64) {
                    Some(f) => {
                        y = f
                    },
                    _ => {},
                }
            } else if line.starts_with("Z = ") {
                match scan_fmt!(&line, "Z = {f}", f64) {
                    Some(f) => {
                        z = f
                    },
                    _ => {},
                }
            
            } else if line.starts_with("FRAME = ") {
                let frame = line.split_at(7).1.trim();
                if !frame.eq("?") {
                    reference_frame = Some(frame.to_string())
                }

            } else if line.starts_with("COMMENTS = ") {
                let c = line.strip_prefix("COMMENTS =")
                    .unwrap()
                    .trim();
                if !c.eq("NO COMMENTS") {
                    comments.push(c.to_string())
                }

            } else if line.starts_with("REF = ") {
                if let Some(s) = scan_fmt!(&line, "REF = {}", String) {
                    time_reference = TimeSystem::from_str(&s)
                }

            } else if line.contains("DLY = ") {

                let items : Vec<&str> = line
                    .split_ascii_whitespace()
                    .collect();
                
                let dual_carrier = line.contains(",");
    
                if items.len() < 4 {
                    continue // format mismatch
                }

                match items[0] {
                    "CAB" => system_delay.rf_cable_delay = f64::from_str(items[3]).unwrap(),
                    "REF" => system_delay.ref_delay = f64::from_str(items[3]).unwrap(),
                    "SYS" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind("=").unwrap();
                            let cal_id = line[offset+1..].trim();
                            if !cal_id.eq("NA") {
                                system_delay = system_delay
                                    .with_calibration_id(cal_id)
                            }
                        }
                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value))); 
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(")","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value))); 
                                }
                            }

                        } else {
                            let value = f64::from_str(items[3]).unwrap();
                            let code = items[6].replace(")","");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay.delays.push((code, Delay::System(value))); 
                            }
                        }
                    },
                    "INT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind("=").unwrap();
                            let cal_id = line[offset+1..].trim();
                            if !cal_id.eq("NA") {
                                system_delay = system_delay
                                    .with_calibration_id(cal_id)
                            }
                        }
                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value))); 
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[10].replace(")","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value))); 
                                }
                            }

                        } else {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace(")","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value))); 
                                }
                            }
                        }
                    },
                    "TOT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind("=").unwrap();
                            let cal_id = line[offset+1..].trim();
                            if !cal_id.eq("NA") {
                                system_delay = system_delay
                                    .with_calibration_id(cal_id)
                            }
                        }
                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value))); 
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(")","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value))); 
                                }
                            }

                        } else {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace(")","");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value))); 
                                }
                                    
                            }
                        }
                    },
                    _ => {}, // non recognized delay type
                };
            
            } else if line.starts_with("CKSUM = ") {

                let _ck :u8 = match scan_fmt!(&line, "CKSUM = {x}", String) {
                    Some(s) => {
                        match u8::from_str_radix(&s, 16) {
                            Ok(hex) => hex,
                            _ => return Err(Error::ChecksumParsingError),
                        }
                    },
                    _ => return Err(Error::ChecksumFormatError),
                };
                
                // check CRC
                let end_pos = line.find("= ")
                    .unwrap();
                _cksum = _cksum.wrapping_add(
                    calc_crc(
                        &line.split_at(end_pos+2).0)?);
        
                //if cksum != ck {
                //    return Err(Error::ChecksumError(ck, cksum))
                //}
                break
            }

            // CRC
            _cksum = _cksum.wrapping_add(
                calc_crc(&line)?);
            
            if let Some(l) = lines.next() {
                line = l 
            } else {
                break
            }
        }
        
        // BLANKS 
        let _ = lines.next().unwrap(); // Blank
        let _ = lines.next().unwrap(); // labels
        let _ = lines.next().unwrap(); // units currently discarded
        // tracks parsing
        let mut tracks: Vec<Track> = Vec::new();
        loop {
            let line = match lines.next() {
                Some(s) => s,
                _ => break // we're done parsing
            };
            if line.len() == 0 { // empty line
                break // we're done parsing
            }
            if let Ok(track) = Track::from_str(&line) {
                tracks.push(track)
            }
        }

        Ok(Cggtts {
            rev_date,
            nb_channels,
            rcvr,
            ims,
            lab,
            reference_frame,
            coordinates: rust_3d::Point3D {
                x,
                y,
                z,
            },
            comments: {
                if comments.len() == 0 {
                    None
                } else {
                    Some(comments)
                }
            },
            delay: system_delay,
            time_reference,
            tracks
        })
    }
}

impl std::fmt::Display for Cggtts {
    /// Writes self into a `Cggtts` file
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut content = String::new();
        let line = format!("CGGTTS GENERIC DATA FORMAT VERSION = {}\n", CURRENT_RELEASE);
        content.push_str(&line);
        let line = format!("REV DATE = {}\n", LATEST_REVISION_DATE);
        content.push_str(&line);

        if let Some(rcvr) = &self.rcvr {
            content.push_str(&format!("RCVR = {}\n", rcvr));
        } else {
            content.push_str("RCVR = RRRRRRRR\n")
        }
        
        let line = format!("CH = {}\n", self.nb_channels); 
        content.push_str(&line);

        if let Some(ims) = &self.ims {
            content.push_str(&format!("RCVR = {}\n", ims));
        } else {
            content.push_str("IMS = 99999\n")
        }
        
        content.push_str(&format!("LAB = {}\n", self.nb_channels)); 
        content.push_str(&format!("X = {}\n", self.coordinates.x)); 
        content.push_str(&format!("Y = {}\n", self.coordinates.y)); 
        content.push_str(&format!("Z = {}\n", self.coordinates.z)); 

        if let Some(r) = &self.reference_frame {
            content.push_str(&format!("FRAME = {}\n", r)); 
        } else {
            content.push_str(&format!("FRAME = ITRF\n"));
        }

        if let Some(comments) = &self.comments {
            content.push_str(&format!("COMMENTS = {}\n", comments[0].to_string()));
        } else {
            content.push_str("COMMENTS = NO COMMENTS\n")
        }

        let delays = self.delay.delays.clone();
        let constellation: Constellation = if self.tracks.len() > 0 {
            self.tracks[0].space_vehicule.constellation
        } else {
            Constellation::default()
        };
        if delays.len() == 1 {
            // Single frequency
            let (code, value) = delays[0];
            match value {
                Delay::Internal(v) => {
                    content.push_str(
                        &format!("INT DLY = {:.1} ns ({} {})",
                            v, constellation.to_3_letter_code(),
                                code));
                },
                Delay::System(v) => {
                    content.push_str(
                        &format!("SYS DLY = {:.1} ns ({} {})",
                            v, constellation.to_3_letter_code(),
                                code));
                },
            }
            if let Some(cal_id) = &self.delay.cal_id {
                content.push_str(
                    &format!("       CAL_ID = {}\n", cal_id));
            } else {
                content.push_str("       CAL_ID = NA\n");
            }

        } else if delays.len() == 2 {
            // Dual frequency
            let (c1, v1) = delays[0];
            let (c2, v2) = delays[1];
            match v1 {
                Delay::Internal(_) => {
                    content.push_str(
                        &format!("INT DLY = {:.1} ns ({} {}), {:.1} ns ({} {})",
                            v1.value(), constellation.to_3_letter_code(), c1,
                                v2.value(), constellation.to_3_letter_code(), c2));
                },
                Delay::System(_) => {
                    content.push_str(
                        &format!("SYS DLY = {:.1} ns ({} {}), {:.1} ns ({} {})",
                            v1.value(), constellation.to_3_letter_code(), c1,
                                v2.value(), constellation.to_3_letter_code(), c2));
                },
            }
            if let Some(cal_id) = &self.delay.cal_id {
                content.push_str(
                    &format!("     CAL_ID = {}\n", cal_id));
            } else {
                content.push_str("     CAL_ID = NA\n");
            }
        }

        content.push_str(
            &format!("CAB DLY = {:.1} ns\n", self.delay.rf_cable_delay));
        
        content.push_str(
            &format!("REF DLY = {:.1} ns\n", self.delay.ref_delay));

        content.push_str(&format!("REF = {}\n", self.time_reference));

        let crc = calc_crc(&content)
            .unwrap();
        content.push_str(&format!("CKSUM = {:2X}\n", crc));
        content.push_str("\n"); // blank

        if self.has_ionospheric_data() {
            content.push_str(TRACK_LABELS_WITH_IONOSPHERIC_DATA);
            content.push_str("\n");
            content.push_str(
"              hhmmss s .1dg .1dg .1ns .1ps/s .1ns .1ps/s .1ns .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns\n")
        } else {
            content.push_str(TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA);
            content.push_str("\n");
            content.push_str(
"             hhmmss s   .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s\n")
        }

        for i in 0..self.tracks.len() {
            content.push_str(&self.tracks[i].to_string());
            content.push_str("\n")
        }
        fmt.write_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_crc() {
        let content = vec![
            "R24 FF 57000 000600  780 347 394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P",
            "G99 99 59509 002200 0780 099 0099 +9999999999 +99999 +9999989831   -724    35 999 9999 +999 9999 +999 00 00 L1C"
        ];
        let expected = vec![
            0x0F, 
            0x71,
        ];
        for i in 0..content.len() {
            let ck = calc_crc(content[i])
                .unwrap();
            let expect = expected[i];
            assert_eq!(ck,
                expect,
                "Failed for \"{}\", expect \"{}\" but \"{}\" locally computed",
                content[i],expect,
                ck)
        }
    }
    
    #[test]
    fn test_time_system() {
        assert_eq!(TimeSystem::default(), TimeSystem::UTC);
        assert_eq!(TimeSystem::from_str("TAI"), TimeSystem::TAI);
        assert_eq!(TimeSystem::from_str("UTC"), TimeSystem::UTC);
        assert_eq!(TimeSystem::from_str("UTC(LAB)"), TimeSystem::UTCk(String::from("LAB"),None));
        assert_eq!(TimeSystem::from_str("UTC(LAB, 10.0)"), TimeSystem::UTCk(String::from("LAB"), Some(10.0)));
    }

    #[test]
    fn test_code() {
        assert_eq!(Code::default(), Code::C1);
        assert_eq!(Code::from_str("C2").unwrap(), Code::C2);
        assert_eq!(Code::from_str("P1").unwrap(), Code::P1);
        assert_eq!(Code::from_str("P2").unwrap(), Code::P2);
        assert_eq!(Code::from_str("E5a").unwrap(), Code::E5a);
    }
}
