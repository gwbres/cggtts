//! CGGTTS is the core structure, it comprises a list of Track.
//! Homepage: <https://github.com/gwbres/cggtts>
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
use rinex::carrier;
use rinex::constellation::Constellation;

use crate::{Track, Delay, delay::SystemDelay};

/// supported `Cggtts` version,
/// non matching input files will be rejected
const CURRENT_RELEASE: &str = "2E";

/// Latest revision date
const LATEST_REVISION_DATE : &str = "2014-02-20";

/// labels in case we provide Ionospheric parameters estimates
const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
"SAT CL MJD STTIME TRKL ELV AZTH REFSV SRSV REFSYS SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
"SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

#[derive(Clone, PartialEq, Debug)]
/// `Rcvr` describes a GNSS receiver
/// (hardware). Used to describe the
/// GNSS receiver or hardware used to evaluate IMS parameters
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

#[derive(Clone, PartialEq, Debug)]
/// Known Reference Time Systems
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

                // determine delay denomination
                let label : String = match scan_fmt!(&line, "{} DLY =.*", String) {
                    Some(l) => l,
                    _ => return Err(Error::DelayIdentificationError(String::from(line))),
                };

                // currently we do not support separate values
                // for two carrier and different System delay values.
                // We grab all delay values, treat them as single carrier,
                // and possible second carrier for delays like SYS DLY are left out
                match label.as_str() {
                    "CAB" => {
                        let start_off = line.find("=").unwrap();
                        let end_off   = line.find("ns").unwrap();
                        let data = &line[start_off+1..end_off];
                        let value = f64::from_str(data.trim())?;
                        system_delay.rf_cable_delay = value
                    },
                    "REF" => {
                        let start_off = line.find("=").unwrap();
                        let end_off   = line.find("ns").unwrap();
                        let data = &line[start_off+1..end_off];
                        let value = f64::from_str(data.trim())?;
                        system_delay.rf_cable_delay = value
                        system_delay.ref_delay = value
                    },
                    "SYS" => {
                        let items : Vec<&str> = line.split_ascii_whitespace().collect();
"SYS DLY = 000.0 ns (GPS C1)     CAL_ID = NA"
                    },
                    "INT" => {
                    },
                    "TOT" => {
                        // special case
                        // Build a calibrated delay for all encountered carriers
                        break
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

        /*// system delays
        if let Some(delay) = &self.tot_dly {
            // total delay defined
            content.push_str(&format!("TOT DLY = {}\n", delay.to_string()))
        
        } else {
            // total delay not defined
            // => SYS or INT DELAY ?
            // INT DELAY prioritary
            if let Some(delay) = &self.int_dly {
                content.push_str(&format!("INT DLY = {}\n", delay))

            } else if let Some(delay) = &self.sys_dly {
                content.push_str(&format!("SYS DLY = {}\n", delay))
            
            } else {
                // neither SYS / INT delay
                // => specify null SYS DLY
                let null_delay = CalibratedDelay {
                    constellation: track::Constellation::default(),
                    values: vec![0.0_f64],
                    codes: vec![String::from("C1")],
                    report: String::from("NA"),
                };
                content.push_str(&format!("SYS DLY = {}\n", null_delay))
            }
            // other delays always there
            content.push_str(&format!("CAB DLY = {:.1}\n", self.cab_dly * 1E9));
            content.push_str(&format!("REF DLY = {:.1}\n", self.ref_dly * 1E9))
        }*/

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
}
