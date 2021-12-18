//! CGGTTS Rust package
//!
//! A package to handle CGGTTS data files.
//! Only 2E Version (latest) supported
//!
//! url: https://github.com/gwbres/cggtts
//!
//! Refer to official doc: 
//! https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf

use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

/// CGGTTS track description
mod track;

/// supported CGGTTS version
/// non matching CGGTTS file input will be rejected
const VERSION: &str = "2E";

/// last revision date
//const REV_DATE: &str = "2014-02-20";

/// CGGTTS structure
#[derive(Debug)]
pub struct Cggtts {
    version: String, // file version info
    rev_date: chrono::NaiveDate, // revision date 
    date: chrono::NaiveDate, // production / creation date
    lab: String, // lab where measurements were performed (possibly unknown)
    rcvr: Option<Rcvr>, // possible GNSS receiver infos
    nb_channels: u16, // nb of GNSS receiver channels
    ims: Option<Rcvr>, // IMS Ionospheric Measurement System (if any)
    xyz: (f32,f32,f32), // antenna phase center coordinates [in m]
    frame: String,
    comments: Option<String>, // comments (if any)
    tot_dly: Option<f64>, // total system + cable delay
    cab_dly: Option<f64>, // ANT cable delay
    int_dly: Option<f64>, // combined delay 
    sys_dly: Option<f64>, // combined delay
    ref_dly: Option<f64>, // LO / RCVR delta
    reference: String, // reference time
    tracks: Vec<track::CggttsTrack> // CGGTTS track(s)
}

/// GNSS Receiver and external system description
#[derive(Clone, Debug)]
struct Rcvr {
    manufacturer: String,
    recv_type: String,
    serial_number: String,
    year: u16,
    software_number: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse file")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("File naming convention")]
    FileNamingConvention,
    #[error("Failed to identify date of creation")]
    DateMjdFormatError,
    #[error("Failed to parse MJD date of creation")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Deprecated versions are not supported")]
    DeprecatedVersion,
    #[error("Version format mismatch")]
    VersionFormatError,
    #[error("Rev. date format mismatch")]
    RevisionDateFormatError,
    #[error("Failed to parse Rev. date")]
    RevisionDateParsingError,
    #[error("RCVR format mismatch")]
    RcvrFormatError,
    #[error("Reference format mismatch")]
    ReferenceFormatError,
    #[error("Failed to parse 'lab' field")]
    LabParsingError,
    #[error("Comments format mismatch")]
    CommentsFormatError,
    #[error("IMS format mismatch")]
    ImsFormatError,
    #[error("Frame format mismatch")]
    FrameFormatError,
    #[error("Channel format mismatch")]
    ChannelFormatError,
    #[error("Failed to parse '{0}' coordinates")]
    CoordinatesParsingError(String),
    #[error("'{0}' delay format mismatch")]
    DelayParsingError(String),
    #[error("Checksum format error")]
    ChecksumFormatError,
    #[error("Failed to parse checksum value")]
    ChecksumParsingError,
    #[error("File Checksum error - got '{0}' but '{1}' locally computed")]
    ChecksumError(u8, u8),
    #[error("CGGTTS Track error")]
    CggttsTrackError(#[from] track::Error)
}

impl Cggtts {
    /// Builds CGGTTS object
    /// lab: production agency
    /// nb channels: GNSS receiver nb channels
    /// coordinates: antenna phase center
    /// reference: reference system time
    /// tracks: measurements (use `empty` if none available)
    /// delays: user should provide a valid combination
    ///       + tot delay (minimum required)
    ///       + ref_dly + sys_dly (average)
    ///       + ref_dly + cable dly + sys_dly (optimum)
    /// rcvr: optionnal GNSS Receiver information
    /// ims: optionnal ionospheric measurement system
    /*pub fn new (lab: &str, nb_channels: u16, 
        coordinates: geo_types::Point, total_delay: Option<f64>,
            internal_delay: Option<f64>, cable_delay: Option<f64>, 
                ref_delay: Option<f64>, system_delay: Option<f64>,
                    tracks: Vec<track::CggttsTrack>,
                        rcvr: Option<Rcvr>, ims: Option<Rcvr>
    ) -> Result<Cggtts, CggttsError> {
        now = chrono::Utc::now();
        Ok(Cggtts{
            version: VERSION.to_string(),
            rev_date: LATEST_REV_DATE;
            date: now.(),
            lab: lab.to_string(),
            rcvr,
            nb_channels,
            ims,
            coordinates,
            frame: String::from(""), // TODO ?
            tot_dly: total_delay,
            int_dly: internal_delay, 
            cab_dly: cable_delay, 
            sys_dly: system_delay,
            ref_dly: ref_delay,
            reference: reference,
        }) 
    }*/
        
    /// Builds CGGTTS from given file
    pub fn from_file (fp: &std::path::Path) -> Result<Cggtts, Error> {
        let file_name = fp.file_name()
            .unwrap()
            .to_str()
                .unwrap();
        // regex to check for file naming convetion
        let file_re = Regex::new(r"(G|R|E|C|J)(S|M|Z)....[1-9][1-9]\.[1-9][1-9][1-9]")
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

        let mut chksum: u8 = 0;
        let file_content = std::fs::read_to_string(&fp).unwrap();
        let mut lines = file_content.split("\n").map(|x| x.to_string()).into_iter();

        // version
        let line = lines.next().unwrap();
        let _ = match scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => {
                if !version.eq(&VERSION) {
                    return Err(Error::DeprecatedVersion)
                }
            },
            _ => return Err(Error::VersionFormatError),
        };

        // CRC is the %256 summation
        // of all ASCII bytes contained in the header
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // rev date 
        let line = lines.next().unwrap();
        let rev_date: chrono::NaiveDate = match scan_fmt!(&line, "REV DATE = {}", String) {
            Some(string) => {
                match chrono::NaiveDate::parse_from_str(string.trim(), "%Y-%m-%d") {
                    Ok(date) => date,
                    _ => return Err(Error::RevisionDateParsingError),
                }
            },
            _ => return Err(Error::RevisionDateFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // rcvr
        let line = lines.next().unwrap();
        let rcvr: Option<Rcvr> = match line.contains("RCVR = RRRRRRRR") {
            true => None,
            false => {
                match scan_fmt! (&line, "RCVR = {} {} {} {d} {}", String, String, String, String, String) {
                    (Some(manufacturer),
                    Some(recv_type),
                    Some(serial_number),
                    Some(year),
                    Some(software_number)) => Some(Rcvr{
                        manufacturer, 
                        recv_type, 
                        serial_number, 
                        year: u16::from_str_radix(&year, 10)?, 
                        software_number
                    }),
                    _ => return Err(Error::RcvrFormatError),
                }
            },
        };
        // crc 
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // channel
        let line = lines.next().unwrap();
        let nb_channels: u16 = match scan_fmt!(&line, "CH = {d}", u16) {
            Some(channel) => channel,
            _ => return Err(Error::ChannelFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // ims
        let line = lines.next().unwrap();
        let ims : Option<Rcvr> = match line.contains("IMS = 99999") { 
            true => None,
            false => { 
                match line.contains("IMS = IIIII") {
                    true => None,
                    false => { // IMS data provided
                        match scan_fmt! (&line, "IMS = {} {} {} {d} {}", String, String, String, String, String) {
                            (Some(manufacturer),
                                Some(recv_type),
                                Some(serial_number),
                                Some(year),
                                Some(software_number)) => Some(
                                    Rcvr {
                                        manufacturer, 
                                        recv_type, 
                                        serial_number, 
                                        year: u16::from_str_radix(&year, 10)?, 
                                        software_number
                                    }),
                            _ => return Err(Error::ImsFormatError),
                        }
                    }
                }
            }
        };

        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // lab
        let line = lines.next().unwrap();
        let lab: String = match scan_fmt!(&line, "LAB = {}", String) {
            Some(lab) => {
                if lab.eq("ABC") {
                    String::from("Unknown")
                } else {
                    lab
                }
            },
            _ => return Err(Error::LabParsingError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // X
        let line = lines.next().unwrap();
        let x: f32 = match scan_fmt!(&line, "X = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("X")))
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // Y
        let line = lines.next().unwrap();
        let y: f32 = match scan_fmt!(&line, "Y = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("Y")))
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // Y
        let line = lines.next().unwrap();
        let z: f32 = match scan_fmt!(&line, "Z = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("Z")))
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // frame 
        let line = lines.next().unwrap();
        let frame: String = match scan_fmt!(&line, "FRAME = {}", String) {
            Some(fr) => fr,
            _ => return Err(Error::FrameFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // comments 
        let line = lines.next().unwrap();
        let comments: Option<String> = match scan_fmt!(&line, "COMMENTS = {}", String) {
            Some(string) => {
                if string.eq("NO COMMENTS") {
                    None
                } else {
                    Some(String::from(string))
                }
            },
            _ => return Err(Error::CommentsFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // system & cable delays 
        let line = lines.next().unwrap();
        let mut ref_dly: Option<f64> = None;
        let mut cab_dly: Option<f64> = None;
        let (tot_dly, int_dly, sys_dly): (Option<f64>,Option<f64>,Option<f64>) = match line.contains("TOT DLY =") {
            true => {
                match scan_fmt!(&line, "TOT DLY = {f} {}", f64, String) {
                    (Some(f),Some(unit)) => {
                        if unit.eq("ms") {
                            (Some(f*1E-3),None,None)
                        } else if unit.eq("us") {
                            (Some(f*1E-6),None,None) 
                        } else if unit.eq("ns") {
                            (Some(f*1E-9),None,None)
                        } else if unit.eq("ps") {
                            (Some(f*1E-12),None,None)
                        } else if unit.eq("fs") {
                            (Some(f*1E-15),None,None)
                        } else {
                            (Some(f),None,None)
                        }
                    },
                    _ => return Err(Error::DelayParsingError(String::from("Total"))),
                }
            },
            false => {
                match line.contains("SYS DLY =") {
                    true => {
                        match scan_fmt!(&line, "SYS DLY = {f} {}", f64, String) {
                            (Some(f),Some(unit)) => {
                                if unit.eq("ms") {
                                    (None,None,Some(f*1E-3))
                                } else if unit.eq("us") {
                                    (None,None,Some(f*1E-6))
                                } else if unit.eq("ns") {
                                    (None,None,Some(f*1E-9))
                                } else if unit.eq("ps") {
                                    (None,None,Some(f*1E-12))
                                } else if unit.eq("fs") {
                                    (None,None,Some(f*1E-15))
                                } else {
                                    (None,None,Some(f))
                                }
                            }
                            _ => return Err(Error::DelayParsingError(String::from("System"))),
                        }
                    },
                    false => {
                        match scan_fmt!(&line, "INT DLY = {f} {}", f64, String) {
                            (Some(f),Some(unit)) => {
                                if unit.eq("ms") {
                                    (None,Some(f*1E-3),None)
                                } else if unit.eq("us") {
                                    (None,Some(f*1E-6),None)
                                } else if unit.eq("ns") {
                                    (None,Some(f*1E-9),None)
                                } else if unit.eq("ps") {
                                    (None,Some(f*1E-12),None)
                                } else if unit.eq("fs") {
                                    (None,Some(f*1E-15),None)
                                } else {
                                    (None,Some(f),None)
                                }
                            },
                            _ => return Err(Error::DelayParsingError(String::from("Internal"))),
                        }
                    }
                }
            }
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // ref delay ?
        if !tot_dly.is_some() {
            if int_dly.is_some() {
                // missing cable delay
                let line = lines.next().unwrap();
                cab_dly = match scan_fmt!(&line, "CAB DLY = {f} {}", f64, String) {
                    (Some(f),Some(unit)) => {
                        if unit.eq("ms") {
                            Some(f*1E-3)
                        } else if unit.eq("us") {
                            Some(f*1E-6)
                        } else if unit.eq("ns") {
                            Some(f*1E-9)
                        } else if unit.eq("ps") {
                            Some(f*1E-12)
                        } else if unit.eq("fs") {
                            Some(f*1E-15)
                        } else {
                            Some(f)
                        }
                    },
                    _ => return Err(Error::DelayParsingError(String::from("Cable"))),
                };
                // crc
                let bytes = line.clone().into_bytes();
                for i in 0..bytes.len() {
                    chksum = chksum.wrapping_add(bytes[i]);
                }
            } // needed cab delay
            // still missing ref. delay
            let line = lines.next().unwrap();
            ref_dly = Some(0.0); /*match scan_fmt!(&line, "REF DLY = {f} {}", f64, String) {
                (Some(f),Some(unit)) => {
                    if unit.eq("ms") {
                        Some(f*1E-3)
                    } else if unit.eq("us") {
                        Some(f*1E-6)
                    } else if unit.eq("ns") {
                        Some(f*1E-9)
                    } else if unit.eq("ps") {
                        Some(f*1E-12)
                    } else if unit.eq("fs") {
                        Some(f*1E-15)
                    } else {
                        Some(f)
                    }
                },
                //_ => return Err(Error::DelayParsingError(String::from("REF"))),//line))),
                _ => return Err(Error::DelayParsingError(String::from(line))),
            };*/
            // crc
            let bytes = line.clone().into_bytes();
            for i in 0..bytes.len() {
                chksum = chksum.wrapping_add(bytes[i]);
            }
        }
        // reference time
        let line = lines.next().unwrap();
        let reference: String = match scan_fmt!(&line, "REF = {}", String) {
            Some(string) => string,
            _ => return Err(Error::ReferenceFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
            // checksum
        let line = lines.next().unwrap();
        let cksum_parsed: u8 = match scan_fmt!(&line, "CKSUM = {x}", String) {
            Some(string) => {
                match u8::from_str_radix(&string, 16) {
                    Ok(hex) => hex,
                    _ => return Err(Error::ChecksumParsingError),
                }
            },
            _ => return Err(Error::ChecksumFormatError),
        };

        // CRC calc. ends on 'CHKSUM = ' (line 15)
        let end_pos = line.find("= ")
            .unwrap(); // already checked
        let bytes = line.clone().into_bytes();
        for i in 0..end_pos+2 {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        //if chksum != cksum_parsed {
        //    return Err(Error::ChecksumError(cksum_parsed, chksum))
        //}

        let _ = lines.next().unwrap(); // Blank line
        let _ = lines.next().unwrap(); // labels line
        let _ = lines.next().unwrap(); // units line currently discarded

        // tracks parsing
        let mut tracks: Vec<track::CggttsTrack> = Vec::new();
        loop {
            let line = match lines.next() {
                Some(s) => s,
                _ => break // we're done parsing
            };
            if line.len() == 0 { // empty line
                break // we're done parsing
            }
            tracks.push(track::CggttsTrack::new(&line)?);
        }

        Ok(Cggtts {
            version: VERSION.to_string(),
            rev_date,
            date: julianday::JulianDay::new((mjd * 1000.0) as i32).to_date(),
            nb_channels,
            rcvr,
            ims,
            lab,
            xyz: (x,y,z),
            frame,
            comments,
            tot_dly, 
            int_dly,
            cab_dly,
            sys_dly,
            ref_dly,
            reference,
            tracks
        })
    }

    /// Returns production date
    pub fn get_date (&self) -> &chrono::NaiveDate { &self.date }
    /// Returns first track produced in file
    pub fn get_first_track (&self) -> Option<&track::CggttsTrack> { self.tracks.get(0) }
    /// Returns last track produced in file
    pub fn get_latest_track (&self) -> Option<&track::CggttsTrack> { self.tracks.get(self.tracks.len()-1) }
    /// Returns requested track
    pub fn get_track (&self, index: usize) -> Option<&track::CggttsTrack> { self.tracks.get(index) }
    /// grabs last track from self 
    pub fn pop_track (&mut self) -> Option<track::CggttsTrack> { self.tracks.pop() }
    /// Appends one track to self
    pub fn push_track (&mut self, track: track::CggttsTrack) { self.tracks.push(track) }
}

// custom display formatter
impl std::fmt::Display for Cggtts {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f, "Version: '{}' | REV DATE '{}' | LAB '{}' | Nb Channels {}\nRECVR: {:?}\nIMS: {:?}\nXYZ: {:?}\nFRAME: {}\nCOMMENTS: {:#?}\nREFERENCE: {}\n",
            self.version, self.rev_date, self.lab, self.nb_channels,
            self.rcvr,
            self.ims,
            self.xyz,
            self.frame,
            self.comments,
            self.reference,
        ).unwrap();
        write! (f, "-------------------------\n").unwrap();
        for i in 0..self.tracks.len() {
            write! (f, "MEAS #{}: {}\n",i, self.tracks[i]).unwrap()
        }
        write!(f, "\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    /*
     * Tests lib against standard test resources
     */
    fn cggtts_test_from_standard_data() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/standard");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                assert_eq!(
                    Cggtts::from_file(&fp).is_err(),
                    false,
                    "Cggtts::new() failed for '{:?}' with '{:?}'",
                    path, 
                    Cggtts::from_file(&fp))
            }
        }
    }
    
    #[test]
    /*
     * Tests lib against advanced test resources
     */
    fn cggtts_test_from_ionospheric_data() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/ionospheric");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                assert_eq!(
                    Cggtts::from_file(&fp).is_err(),
                    false,
                    "Cggtts::new() failed for '{:?}' with '{:?}'",
                    path, 
                    Cggtts::from_file(&fp))
            }
        }
    }
}
