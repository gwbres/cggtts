//! CGGTTS is the core structure, it comprises a list of Track.
//! Homepage: <https://github.com/gwbres/cggtts>
use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;
use rinex::Constellation;

use crate::{Track, Delay, delay::SystemDelay, CalibratedDelay};

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

#[derive(Clone, Debug)]
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
    /// date of file revision 
    pub rev_date: chrono::NaiveDate, 
    /// date of production
    pub date: chrono::NaiveDate,
    /// laboratory / agency where measurements were performed (if unknown)
    pub lab: Option<String>, 
    /// possible GNSS receiver infos
    pub rcvr: Option<Rcvr>, 
    /// nb of GNSS receiver channels
    pub nb_channels: u16, 
    /// IMS Ionospheric Measurement System (if any)
    pub ims: Option<Rcvr>, 
    /// Description of Reference time system (if any)
    pub time_reference: Option<String>, 
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
    #[error("file naming convention")]
    FileNamingConvention,
    #[error("failed to identify date of creation")]
    DateMjdFormatError,
    #[error("failed to parse mjd date")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("only revision 2E is supported")]
    VersionMismatch,
    #[error("version format mismatch")]
    VersionFormatError,
    #[error("revision date format mismatch")]
    RevisionDateFormatError,
    #[error("failed to parse revision date")]
    RevisionDateParsingError,
    #[error("\"rcvr\" format mismatch")]
    RcvrFormatError,
    #[error("\"reference\" format mismatch")]
    ReferenceFormatError,
    #[error("failed to parse \"lab\" field")]
    LabParsingError,
    #[error("comments format mismatch")]
    CommentsFormatError,
    #[error("\"ims\" format mismatch")]
    ImsFormatError,
    #[error("frame format mismatch")]
    FrameFormatError,
    #[error("channel format mismatch")]
    ChannelFormatError,
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
    //#[error("CggttsTrack error")]
    //CggttsTrackError(#[from] track::Error),
}

impl Default for Cggtts {
    /// Buils default `Cggtts` structure,
    fn default() -> Cggtts {
        Cggtts {
            rev_date: chrono::NaiveDate::parse_from_str(
                LATEST_REVISION_DATE,
                "%Y-%m-%d").unwrap(),
            date: chrono::Utc::today().naive_utc(),
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
            time_reference: None,
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
    pub fn with_time_reference (&self, reference: &str) -> Self { 
        let mut c = self.clone();
        c.time_reference = Some(reference.to_string());
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

    /// Builds Self from given `Cggtts` file.
    pub fn from_file (fp: &str) -> Result<Self, Error> {
        // check against file naming convetion
        let path = std::path::Path::new(fp);
        let file_name = path.file_name()
            .unwrap()
            .to_str()
                .unwrap();
        let file_re = Regex::new(r"(G|R|E|C|J)(S|M|Z)....[1-9][0-9]\.[0-9][0-9][0-9]")
            .unwrap();
        if !file_re.is_match(file_name) {
            return Err(Error::FileNamingConvention)
        }

        // identify date of creation 
        // using file naming convention 
        let mjd: f64 = match file_name.find(".") {
            Some(location) => {
                f64::from_str(file_name.split_at(location-2).1)?
            },
            _ => return Err(Error::DateMjdFormatError),
        };
        
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
        let mut line = lines.next()
            .unwrap();
        let _ = match scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => {
                if !version.eq(&CURRENT_RELEASE) {
                    return Err(Error::VersionMismatch)
                }
            },
            _ => return Err(Error::VersionFormatError),
        };
        
        let mut rev_date = chrono::NaiveDate::parse_from_str(LATEST_REVISION_DATE, "%Y-%m-%d")
            .unwrap();
        let mut nb_channels :u16 = 0;
        let mut rcvr : Option<Rcvr> = None;
        let mut ims : Option<Rcvr> = None;
        let mut lab: Option<String> = None;
        let mut cksum :u8 = calc_crc(&line)?;
        let mut comments: Vec<String> = Vec::new();
        let mut reference_frame: Option<String> = None;
        let mut time_reference : Option<String> = None;
        let (mut x, mut y, mut z) : (f64,f64,f64) = (0.0, 0.0, 0.0);

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
                        rcvr = Some(Rcvr {
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
                match scan_fmt!(&line, "FRAME = {}", String) {
                    Some(fr) => {
                        if !fr.trim().eq("?") {
                            reference_frame = Some(fr)
                        }
                    },
                    _ => {},
                }

            } else if line.starts_with("COMMENTS = ") {
                comments.push(line.strip_prefix("COMMENTS = ")
                    .unwrap()
                    .trim()
                    .to_string())

            } else if line.starts_with("REF = ") {
                match scan_fmt!(&line, "REF = {}", String) {
                    Some(s) => {
                        time_reference = Some(s) 
                    },
                    _ => {},
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
                let start_off = line.find("=").unwrap();
                let end_off   = line.find("ns").unwrap();
                let data = &line[start_off+1..end_off];
                let value = f64::from_str(data.trim())?;

                match label.as_str() {
                    "CAB" => {
                        system_delay.add_delay(
                            CalibratedDelay {
                                delay: Delay::RfCable(value),
                                constellation: Constellation::Mixed,
                                info: None,
                            }
                        )
                    },
                    "REF" => {
                        system_delay.add_delay(
                            CalibratedDelay {
                                delay: Delay::Reference(value),
                                constellation: Constellation::Mixed,
                                info: None,
                            }
                        )
                    },
                    "SYS" => {
                        system_delay.add_delay(
                            CalibratedDelay {
                                delay: Delay::System(value),
                                constellation: Constellation::Mixed,
                                info: None,
                            }
                        )
                    },
                    "INT" => {
                        system_delay.add_delay(
                            CalibratedDelay {
                                delay: Delay::Internal(value),
                                constellation: Constellation::Mixed,
                                info: None,
                            }
                        )
                    },
                    "TOT" => {
                        // special case, Total delay is given,
                        // assumes all other delays are not known
                        system_delay.add_delay(
                            CalibratedDelay { // we declare it as RfCable arbitrarily,
                                delay: Delay::RfCable(value), // which is convenient because it is
                                constellation: Constellation::Mixed, // not constellation dependent
                                info: None,
                            }
                        )
                    },
                    _ => {}, // non recognized delay type
                };
            
            } else if line.starts_with("CKSUM = ") {

                let ck :u8 = match scan_fmt!(&line, "CKSUM = {x}", String) {
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
                cksum = cksum.wrapping_add(
                    calc_crc(
                        &line.split_at(end_pos+2).0)?);
        
                //if cksum != ck {
                //    return Err(Error::ChecksumError(ck, cksum))
                //}
                break
            }

            // CRC
            cksum = cksum.wrapping_add(
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
            /*if let Ok(track) = Track::from_str(&line) {
                tracks.push(track)
            }*/
        }

        Ok(Cggtts {
            rev_date,
            date: julianday::JulianDay::new(((mjd * 1000.0) + 2400000.5) as i32).to_date(),
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
    
/*
    /// Writes self into a `Cggtts` file
    pub fn to_file (&self, fp: &str) -> Result<(), Error> {
        let mut content = String::new();

        let line = format!("CGGTTS GENERIC DATA FORMAT VERSION = {}\n", VERSION);
        content.push_str(&line);
        let line = format!("REV DATE = {}\n", LATEST_REV_DATE);
        content.push_str(&line);

        if let Some(rcvr) = &self.rcvr {
            let line = format!("RCVR = {}\n", &rcvr.to_string());
            content.push_str(&line);
        } else {
            content.push_str("RCVR = RRRRRRRR\n")
        }
        
        let line = format!("CH = {}\n", self.nb_channels); 
        content.push_str(&line);

        if let Some(ims) = &self.ims {
            let line = format!("IMS = {}\n", &ims.to_string());
            content.push_str(&line)
        } else {
            content.push_str("IMS = 99999\n")
        }
        
        let line = format!("LAB = {}\n", self.nb_channels); 
        content.push_str(&line);
        let line = format!("X = {}\n", self.coordinates.0); 
        content.push_str(&line);
        let line = format!("Y = {}\n", self.coordinates.1); 
        content.push_str(&line);
        let line = format!("Z = {}\n", self.coordinates.2); 
        content.push_str(&line);
        let line = format!("FRAME = {}\n", self.frame); 
        content.push_str(&line);

        if let Some(comments) = &self.comments {
            let line = format!("COMMENTS = {}\n", comments.to_string());
            content.push_str(&line);
        
        } else {
            content.push_str("COMMENTS = NO COMMENTS\n")
        }

        // system delays
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
        }
        content.push_str(&format!("REF = {}\n", self.reference.to_string()));
        content.push_str(&format!("CKSUM = {:2X}\n", calc_crc(&content)?));
        content.push_str("\n"); // blank

        if self.has_ionospheric_parameters() {
            content.push_str(track::TRACK_LABELS_WITH_IONOSPHERIC_DATA);
            content.push_str("\n");
            content.push_str(
"              hhmmss s .1dg .1dg .1ns .1ps/s .1ns .1ps/s .1ns .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns\n")
        } else {
            content.push_str(track::TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA);
            content.push_str("\n");
            content.push_str(
"             hhmmss s   .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s\n")
        }

        for i in 0..self.tracks.len() {
            content.push_str(&self.tracks[i].to_string());
            content.push_str("\n")
        }
        Ok(std::fs::write(fp, content)?) 
    }
*/
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
            assert_eq!(ck,expect,"Failed for \"{}\", expect \"{}\" but \"{}\" locally computed",content[i],expect,ck)
        }
    }
}
