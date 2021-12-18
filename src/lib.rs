//! CGGTTS Rust package
//!
//! A package to handle CGGTTS data files.
//!
//! Only "2E" Version (latest to date) supported
//!
//! Only single frequency `Cggtts` is supported at the moment,
//! dual frequency `Cggtts` to be supported in coming releases.
//!
//! Homepage: <https://github.com/gwbres/cggtts>
//!
//! Official BIPM `Cggtts` specifications:
//! <https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf>
//!

pub mod track;
use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

/// supported `Cggtts` version
/// non matching input files will be rejected
const VERSION: &str = "2E";

/// last revision date
const REV_DATE: &str = "2014-02-20";

/// `Cggtts` structure
#[derive(Debug)]
pub struct Cggtts {
    version: String, // file version info
    rev_date: chrono::NaiveDate, // revision date 
    date: chrono::NaiveDate, // production / creation date
    lab: String, // lab where measurements were performed (possibly unknown)
    rcvr: Option<Rcvr>, // possible GNSS receiver infos
    nb_channels: u16, // nb of GNSS receiver channels
    ims: Option<Rcvr>, // IMS Ionospheric Measurement System (if any)
    // Antenna phase center coordinates [m]
    // in `ITFR` spatial reference
    coordinates: (f32,f32,f32), 
    frame: String,
    comments: Option<String>, // comments (if any)
    tot_dly: Option<f64>, // absolute total delay
    cab_dly: Option<f64>, // ANT cable delay
    int_dly: Option<f64>, // 
    sys_dly: Option<f64>, // 
    ref_dly: Option<f64>, // delay between local clock & receiver clock
    reference: String, // reference time
    tracks: Vec<track::CggttsTrack> // CGGTTS track(s)
}

#[derive(Clone, Debug)]
/// `Rcvr` describes a GNSS receiver
pub struct Rcvr {
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
    #[error("File Checksum error - expected '{0}' but '{1}' locally computed")]
    ChecksumError(u8, u8),
    #[error("CGGTTS Track error")]
    CggttsTrackError(#[from] track::Error),
    #[error("Missing \"{0}\" delay information")]
    MissingDelayInformation(String),
}

impl Default for Cggtts {
    /// Buils default `Cggtts` structure
    fn default() -> Cggtts {
        let today = chrono::Utc::today(); 
        let rcvr: Option<Rcvr> = None;
        let ims: Option<Rcvr> = None;
        let tracks: Vec<track::CggttsTrack> = Vec::new();
        let comments: Option<String> = None;
        let delays: (Option<f64>,Option<f64>,Option<f64>,Option<f64>, Option<f64>) = (None, None, None, None, None);
        Cggtts {
            version: VERSION.to_string(),
            rev_date: chrono::NaiveDate::parse_from_str(REV_DATE, "%Y-%m-%d")
                .unwrap(),
            date: today.naive_utc(),
            lab: String::from("Unknown"),
            nb_channels: 0,
            coordinates: (0.0, 0.0, 0.0),
            rcvr,
            tracks,
            ims,
            reference: String::from("Unknown"),
            comments,
            frame: String::from("?"),
            tot_dly: delays.0,
            ref_dly: delays.1,
            int_dly: delays.2,
            sys_dly: delays.3,
            cab_dly: delays.4,
        }
    }
}

impl Cggtts {
    /// Builds `Cggtts` object with default attributes
    pub fn new() -> Cggtts { Default::default() }

    /// Returns production date
    pub fn get_date (&self) -> chrono::NaiveDate { self.date }
    /// Returns revision date
    pub fn get_revision_date (&self) -> chrono::NaiveDate { self.rev_date }

    /// Returns true if all tracks follow the tracking specifications
    /// from `BIPM`, ie., all tracks last for `CggttsTrack::BIPM_SPECIFIED_TRACKING_DURATION`
    pub fn matches_bipm_tracking_specs (&self) -> bool {
        for i in 0..self.tracks.len() {
            if self.tracks[i].get_duration() != track::BIPM_SPECIFIED_TRACKING_DURATION {
                return false
            }
        }
        true
    }

    /// Assigns `lab` agency
    pub fn set_lab_agency (&mut self, lab: &str) { self.lab = String::from(lab) }
    /// Returns `lab` agency
    pub fn get_lab_agency (&self) -> &str { &self.lab } 
    
    /// Assigns GNSS receiver number of channels
    pub fn set_nb_channels (&mut self, ch: u16) { self.nb_channels = ch }
    /// Returns GNSS receiver number of channels
    pub fn get_nb_channels (&self) -> u16 { self.nb_channels }

    /// Assigns antenna phase center coordinates [m],
    /// coordinates should use `IRTF` referencing
    pub fn set_antenna_coordinates (&mut self, coords: (f32,f32,f32)) { self.coordinates = coords }
    /// Returns antenna phase center coordinates [m], `IRTF` referencing
    pub fn get_antenna_coordinates (&self) -> (f32,f32,f32) { self.coordinates }

    /// Assigns reference time label
    pub fn set_time_reference (&mut self, reference: &str) { self.reference = String::from(reference) }
    /// Returns reference time label
    pub fn get_reference_time (&self) -> &str { &self.reference }

    /// Returns total delay
    /// basic use: 
    ///    [ANT]---->[system]----> (clock) 
    ///    |-------------------------------> total delay
    ///
    /// intermediate: 
    ///    [ANT]------>[system]-------------------> (clock) 
    ///    |-?->--?--->--------------->system delay 
    ///                   ^
    ///                   |---------------------------ref
    /// cable and intrinsic delays are not known
    /// but deduced by the delta knowledge
    ///
    /// advanced: 
    ///    [ANT]------->[system]---> (clock) 
    ///    |-a->--cab-->--------b-->system delay 
    ///                   ^
    ///                   |----------ref
    /// full system knownledge, including internal granularity 
    /// this scenario is required for Dual Frequency CGGTTS generation
    pub fn get_total_delay (&self) -> Result<f64, Error> {
        match self.tot_dly {
            Some(delay) => Ok(delay), // basic usage
            _ => { // detailed delays provided
                // ref. dly always needed from there
                match self.ref_dly {
                    Some(ref_dly) => {
                        match self.sys_dly {
                            Some(sys_dly) => {
                                // intermediate use case
                                Ok(ref_dly + sys_dly)
                            },
                            None => {
                                // advance usage requires cable + int delays then
                                match self.int_dly {
                                    Some(int_dly) => {
                                        match self.cab_dly {
                                            Some(cab_dly) => {
                                                Ok(cab_dly + int_dly + ref_dly)
                                            },
                                            None => return Err(Error::MissingDelayInformation(String::from("cable"))),
                                        }
                                    },
                                    None => return Err(Error::MissingDelayInformation(String::from("internal")))
                                }
                            }
                        }
                    },
                    None => return Err(Error::MissingDelayInformation(String::from("ref"))),
                }
            }
        }
    }

    /// Assigns `total` delay
    /// `total` delay comprises all internal delays
    /// this interface should only be used when internal system 
    /// is totally unknown, it is not recommended
    pub fn set_total_delay (&mut self, dly: f64) { self.tot_dly = Some(dly) }

    /// Assigns `reference` delay
    /// `reference` delay is defined as the time delay
    /// between the local clock and receiver internal clock
    pub fn set_reference_delay (&mut self, dly: f64) { self.ref_dly = Some(dly) }
    /// Returns `reference` delay (if provided)
    pub fn get_reference_delay (&self) -> Option<f64> { self.ref_dly }
    
    /// Assigns `internal` delay
    /// `internal` delay is the time delay of the gnss signal
    /// inside the antenna and inside the receiver, basically
    /// excluding `cable` delay
    pub fn set_internal_delay (&mut self, dly: f64) { self.int_dly = Some(dly) }
    /// Returns `internal` delay (if provided)
    pub fn get_internal_delay (&self) -> Option<f64> { self.int_dly }
    
    /// Assigns `cable` delay
    /// `cable` delay is defined as the time delay from the antenna to receiver
    pub fn set_cable_delay (&mut self, dly: f64) { self.cab_dly = Some(dly) }
    /// Returns `cable` delay (if provided)
    pub fn get_cable_delay (&self) -> Option<f64> { self.cab_dly }
    
    /// Assigns `system` delay
    /// `system` delay is defined as `internal` + `cable` delay
    /// in case it is impossible to differentiate the two
    pub fn set_system_delay (&mut self, dly: f64) { self.sys_dly = Some(dly) }
    /// Returns `system` delay (if provided)
    pub fn get_system_delay (&self) -> Option<f64> { self.sys_dly }
    
    /// Returns first track produced in file (if any)
    pub fn get_first_track (&self) -> Option<&track::CggttsTrack> { self.tracks.get(0) }
    /// Returns last track produced in file (if any)
    pub fn get_latest_track (&self) -> Option<&track::CggttsTrack> { self.tracks.get(self.tracks.len()-1) }
    /// Returns requested track (if possible)
    pub fn get_track (&self, index: usize) -> Option<&track::CggttsTrack> { self.tracks.get(index) }
    /// Grabs last track from self (if possible)
    pub fn pop_track (&mut self) -> Option<track::CggttsTrack> { self.tracks.pop() }
    /// Appends one track to self (if possible)
    pub fn push_track (&mut self, track: track::CggttsTrack) { self.tracks.push(track) }
    
    /// Builds `Cggtts` from given file.
    /// File must respect naming convention.
    pub fn from_file (fp: &std::path::Path) -> Result<Cggtts, Error> {
        let file_name = fp.file_name()
            .unwrap()
            .to_str()
                .unwrap();
        // check against file naming convetion
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

        // IMS information
        // IMS = IIII is probably for demo simplicity => should be removed
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
            //ref_dly = Some(0.0); 
            ref_dly = match scan_fmt!(&line, "REF DLY = {f} {}", f64, String) {
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
                _ => return Err(Error::DelayParsingError(String::from("Ref"))),
            };
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
        let _cksum_parsed: u8 = match scan_fmt!(&line, "CKSUM = {x}", String) {
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
            .unwrap(); // already matching
        let bytes = line.clone().into_bytes();
        for i in 0..end_pos+2 {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // TODO unlock checksum verification
        //if chksum != cksum_parsed {
        //    return Err(Error::ChecksumError(chksum, ck))
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
            tracks.push(track::CggttsTrack::from_str(&line)?);
        }

        Ok(Cggtts {
            version: VERSION.to_string(),
            rev_date,
            date: julianday::JulianDay::new(((mjd * 1000.0) + 2400000.5) as i32).to_date(),
            nb_channels,
            rcvr,
            ims,
            lab,
            coordinates: (x,y,z), 
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
}

// custom display formatter
impl std::fmt::Display for Cggtts {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f, "Version: '{}' | REV DATE '{}' | LAB '{}' | Nb Channels {}\nRECVR: {:?}\nIMS: {:?}\nCoordinates: {:?}\nFRAME: {}\nCOMMENTS: {:#?}\nREFERENCE: {}\n",
            self.version, self.rev_date, self.lab, self.nb_channels,
            self.rcvr,
            self.ims,
            self.coordinates,
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
    use float_cmp::approx_eq;
    
    #[test]
    /// Tests default constructor 
    fn cggtts_test_default() {
        let cggtts = Cggtts::new();
        let today = chrono::Utc::today();
        let rev_date = chrono::NaiveDate::parse_from_str(REV_DATE,"%Y-%m-%d")
            .unwrap();
        assert_eq!(cggtts.lab, "Unknown");
        assert_eq!(cggtts.nb_channels, 0);
        assert_eq!(cggtts.frame, "?");
        assert_eq!(cggtts.reference, "Unknown");
        assert_eq!(cggtts.tot_dly, None);
        assert_eq!(cggtts.ref_dly, None);
        assert_eq!(cggtts.int_dly, None);
        assert_eq!(cggtts.cab_dly, None);
        assert_eq!(cggtts.sys_dly, None);
        assert_eq!(cggtts.coordinates, (0.0,0.0,0.0));
        assert_eq!(cggtts.rev_date, rev_date); 
        assert_eq!(cggtts.date, today.naive_utc())
    }

    #[test]
    /// Tests basic usage 
    fn cggtts_basic_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_total_delay(300E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        assert_eq!(cggtts.get_system_delay().is_none(), true); // not provided
        assert_eq!(cggtts.get_cable_delay().is_none(), true); // not provided
        assert_eq!(cggtts.get_reference_delay().is_none(), true); // not provided
        assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        assert_eq!(cggtts.get_total_delay().unwrap(), 300E-9); // basic usage
    }

    #[test]
    /// Test normal / intermediate usage
    fn cgggts_intermediate_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_reference_delay(100E-9);
        cggtts.set_system_delay(150E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        assert_eq!(cggtts.get_cable_delay().is_some(), false); // not provided
        assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        assert_eq!(cggtts.get_system_delay().is_some(), true); // provided
        assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        assert_eq!(cggtts.get_total_delay().unwrap(), 250E-9); // intermediate usage
    }

    #[test]
    /// Test advanced usage
    fn cgggts_advanced_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_reference_delay(100E-9);
        cggtts.set_internal_delay(25E-9);
        cggtts.set_cable_delay(300E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        assert_eq!(cggtts.get_system_delay().is_some(), false); // not provided: we have granularity
        assert_eq!(cggtts.get_cable_delay().is_some(), true); // provided
        assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        assert_eq!(cggtts.get_internal_delay().is_some(), true); // provided
        assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        assert!(
            approx_eq!(f64,
                cggtts.get_total_delay().unwrap(),
                425E-9, // advanced usage
                epsilon = 1E-9
            )
        )
    }

    #[test]
    /// Tests standard file parsing
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
                let cggtts = Cggtts::from_file(&fp);
                assert_eq!(
                    cggtts.is_err(),
                    false,
                    "Cggtts::from_file() failed for '{:?}' with '{:?}'",
                    path,
                    cggtts);
                println!("{:?}", cggtts)
            }
        }
    }
    
    #[test]
    /// Tests advanced file parsing
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
                let cggtts = Cggtts::from_file(&fp);
                assert_eq!(
                    cggtts.is_err(), 
                    false,
                    "Cggtts::from_file() failed for '{:?}' with '{:?}'",
                    path, 
                    cggtts); 
                println!("{:?}", cggtts)
            }
        }
    }
}
