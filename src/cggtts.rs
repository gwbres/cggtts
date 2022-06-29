//! CGGTTS is the core structure, it comprises a list of Track.
//! Homepage: <https://github.com/gwbres/cggtts>
use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

use crate::LATEST_REVISION;
use crate::LATEST_RELEASE_DATE;
use crate::{Track, CalibratedDelay};

/*
            version: VERSION.to_string(),
            rev_date: chrono::NaiveDate::parse_from_str(LATEST_REV_DATE, "%Y-%m-%d")
                .unwrap(),
*/

#[derive(Clone, Debug)]
/// `Rcvr` describes a GNSS receiver
/// (hardware). Used to describe the
/// GNSS receiver or hardware used to evaluate IMS parameters
pub struct Rcvr {
    manufacturer: String,
    recv_type: String,
    serial_number: String,
    year: u16,
    software_number: String,
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
        fmt.write_str(&self.software_number)?;
        Ok(())
    }
}

/// `Cggtts` structure comprises
/// a measurement system and 
/// and its Common View realizations (`tracks`)
#[derive(Debug, Clone)]
pub struct Cggtts {
    pub date: chrono::NaiveDate, 
    /// lab where measurements were performed (if unknown)
    pub lab: Option<String>, 
    /// possible GNSS receiver infos
    pub rcvr: Option<Rcvr>, 
    /// nb of GNSS receiver channels
    pub nb_channels: u16, 
    /// IMS Ionospheric Measurement System (if any)
    pub ims: Option<Rcvr>, 
    /// Antenna phase center coordinates [m]
    /// in `ITFR` spatial reference
    pub coordinates: rust_3d::Point3D, 
    /// frame field
    pub frame: String,
    /// Comments (if any..)
    pub comments: Option<String>, 
    /// Summation of internal delays,
    /// refer to [Delay] object definition,
    /// to understand their meaning
    pub delays: Vec<CalibratedDelay>;
    /// reference time
    pub reference: String, 
    /// Tracks: actual measurements / realizations
    pub tracks: Vec<Track>
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
    #[error("deprecated versions are not supported")]
    DeprecatedVersion,
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
    #[error("CggttsTrack error")]
    CggttsTrackError(#[from] track::Error),
}

impl Default for Cggtts {
    /// Buils default `Cggtts` structure,
    /// with production date set to now().
    ///
    /// If nothing more is done regarding `System Delays`,
    /// the system is specified for an uncalibrated and unknown
    /// total delay.
    ///
    /// For more precise use, the user should specify
    /// at least a `total delay` or a esimation
    /// of internal / cable delays is even better
    fn default() -> Cggtts {
        Cggtts {
            date: chrono::Utc::today().naive_utc(),
            lab: String::from("Unknown"),
            nb_channels: 0,
            coordinates: (0.0, 0.0, 0.0),
            rcvr: None,
            tracks: Vec::new(),
            ims: None, 
            reference: String::from("Unknown"),
            comments: None,
            frame: String::from("?"),
            tot_dly: None,
            int_dly: None,
            sys_dly: None,
            cab_dly: 0.0_f64,
            ref_dly: 0.0_f64,
        }
    }
}

impl Cggtts {
    /// Builds `Cggtts` object with desired attributes.
    /// Date is set to `now` by default, use
    /// `with_date()` to customize.
    pub fn new (lab: Option<String>, nb_channels: u16, rcvr: Option<Rcvr>) -> Self { 
        let mut c = Self::default();
        if let Some(lab) = lab {
            c.with_lab_agency(lab)
        }
        c.with_nb_channels(nb_channels);
        if Some(rcvr) = rcvr {
            c.with_receiver(rcvr)
        }
    }

    /// Returns true if all tracks follow 
    /// BIPM tracking specifications
    pub fn follows_bipm_specs (&self) -> bool {
        for track in self.tracks {
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
    pub fn with_nb_channes (&self, ch: u16) { 
        let mut c = self.clone();
        c.nb_channels = ch;
        c
    }

    /// Returns `Cggtts` with desired Receiver infos
    pub fn with_receiver (&self, rcvr: Rcvr) { 
        let mut c = self.clone();
        c.rcvr = Some(rcvr);
        c
    }

    /// Returns `Cggtts` with desired `IMS` evaluation
    /// hardware infos
    pub fn with_ims_infos (&self, ims: Rcvr) { 
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
    
    /// returns `Cggtts` with desired reference time system
    pub fn with_time_reference (&self, reference: &str) { 
        let mut c = self.clone();
        c.reference = reference.to_string();
        c
    }

    /// Returns `cggtts` with desired extra delay value.
    /// If other delay values have already been specified,
    /// this extra delay must correspond to an already existing
    /// GNSS system, otherwise it is not added internally.
    /// This obviously does not stand, if user specifies his
    /// delay on the `Mixed` GNSS constellation, which is not recommended
    pub fn with_delay (&self, CalibratedDelay: delay) -> Cggtts {
        let mut c = self.clone();
        let mut matches: bool = true;
        for dly in self.delays {
            if delay.constellation != Constellation::Mixed {
                if dly.constellation != Constellation::Mixed {
                    matches = false;
                    break
                }
            }
        }
        if !matches {
            return;
        }
        c.delays.push(delay);
        c
    }

    /// Evaluates total delay of this measurement system,
    /// with summation of all declared internal delays
    pub fn total_delay (&self) -> CalibratedDelay {
        let mut ret = CalibratedDelay::default();
        match &self.tot_dly {
            Some(delay) => {
                // parsing / user did provide a total delay
                ret.constellation = delay.constellation.clone();
                for i in 0..delay.values.len() {
                    ret.codes.push(delay.codes[i].clone());
                    ret.values.push(delay.values[i]);
                }
                ret.report = String::from(delay.get_calibration_report())
            },
            None => {
                // parsing / user did not provide a total delay
                // we must evaluate it ourselves
                match &self.int_dly {
                    // internal delay specified
                    // gets *2 (A+B) definition
                    Some(delay) => { 
                        // int delay specified
                        ret.constellation = delay.constellation.clone();
                        for i in 0..delay.values.len() { 
                            ret.codes.push(delay.codes[i].clone()); 
                            ret.values.push(delay.values[i] * 2.0_f64) // (A+B)
                        }
                    },
                    None => {
                        // int delay not specified
                        // => should have a system delay then
                        match &self.sys_dly {
                            Some(delay) => {
                                // system delay specified
                                ret.constellation = delay.constellation.clone();
                                for i in 0..delay.values.len() {
                                    ret.codes.push(delay.codes[i].clone());
                                    ret.values.push(delay.values[i]);
                                }
                            },
                            None => { // no delay at all, 0 assumed then
                                ret.values.push(0.0);
                                ret.codes.push(String::from("C1"))
                            },
                        }
                    }
                }
                // always add cab delay
                for i in 0..ret.values.len() {
                    ret.values[i] += self.cab_dly
                }
            }
        }
        ret
    }
    
    /// Returns number of tracks contained in self
    pub fn len(&self) -> usize { self.tracks.len() }

    /// Returns first track produced in file (if any)
    pub fn first_track (&self) -> Option<&track::CggttsTrack> { 
        self.tracks.get(0) 
    }
    
    /// Returns last track produced in file (if any)
    pub fn latest_track (&self) -> Option<&track::CggttsTrack> { 
        self.tracks.get(self.tracks.len()-1) 
    }

    /// returns true if self is `Single Frequency Cggtts`
    pub fn is_single_frequency (&self) -> bool { 
        self.total_delay().values.len() == 1 
    }
    
    /// returns true if self is `Single Frequency Cggtts`
    pub fn is_dual_frequency (&self) -> bool { 
        !self.is_single_frequency() 
    }

    /// Returns true if self contains ionospheric information
    pub fn has_ionospheric_parameters (&self) -> bool {
        let mut ret = false;
        for i in 0..self.len() {
            if self.get_track(i)
                .unwrap()
                    .has_ionospheric_parameters() {
                        ret = true
                    }
        }
        ret
    }

    /// Builds `Cggtts` with given `CalibratedDelay` defined as `System delay` 
    pub fn with_system_delay (&self, delay: CalibratedDelay) -> Cggtts {
        Cggtts {
            version: self.version.clone(),
            recv_date: self.recv_date.clone(),
            sys_dly: Some(delay),
        }
    }
    /// Sets `internal` delay (refer to README)
    pub fn set_internal_delay (&mut self, delay: CalibratedDelay) { self.int_dly = Some(delay.clone()) }
    /// Sets `total` delay (refer to README)
    pub fn set_total_delay (&mut self, delay: CalibratedDelay) { self.tot_dly = Some(delay.clone()) }
    /// Sets `cable` delay (refer to README)
    pub fn set_cable_delay (&mut self, delay: f64) { self.cab_dly = delay }
    /// Sets `ref` delay (refer to README)
    pub fn set_ref_delay (&mut self, delay: f64) { self.ref_dly = delay }

    /// Builds self from given `Cggtts` file.
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
        
        let file_content = std::fs::read_to_string(&fp).unwrap();
        let mut lines = file_content.split("\n")
            .map(|x| x.to_string())
            //.map(|x| x.to_string() +"\n")
            //.map(|x| x.to_string() +"\r"+"\n")
                .into_iter();
        // version
        let line = lines.next()
            .unwrap();
        let _ = match scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => {
                if !version.eq(&VERSION) {
                    return Err(Error::DeprecatedVersion)
                }
            },
            _ => return Err(Error::VersionFormatError),
        };
        // crc 
        let mut cksum: u8 = calc_crc(&line)?;
        // rev date 
        let line = lines.next()
            .unwrap();
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
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // rcvr
        let line = lines.next()
            .unwrap();
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
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // channel
        let line = lines.next().unwrap();
        let nb_channels: u16 = match scan_fmt!(&line, "CH = {d}", u16) {
            Some(channel) => channel,
            _ => return Err(Error::ChannelFormatError),
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // ims 
        let line = lines.next()
            .unwrap();
        let ims : Option<Rcvr> = match line.contains("IMS = 99999") {
            true => None,
            false => { 
                match scan_fmt!(&line, "IMS = {} {} {} {d} {}", String, String, String, String, String) {
                    (Some(manufacturer),
                    Some(recv_type),
                    Some(serial_number),
                    Some(year),
                    Some(software_number)) => 
                        Some(Rcvr {
                            manufacturer, 
                            recv_type, 
                            serial_number, 
                            year: u16::from_str_radix(&year, 10)?, 
                            software_number
                        }),
                    _ => return Err(Error::ImsFormatError),
                }
            }
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // lab
        let line = lines.next()
            .unwrap();
        let lab: String = match line.strip_prefix("LAB = ") {
            Some(s) => String::from(s.trim()),
            _ => return Err(Error::LabParsingError),
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // X
        let line = lines.next().unwrap();
        let x: f32 = match scan_fmt!(&line, "X = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("X")))
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // Y
        let line = lines.next()
            .unwrap();
        let y: f32 = match scan_fmt!(&line, "Y = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("Y")))
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // Z
        let line = lines.next()
            .unwrap();
        let z: f32 = match scan_fmt!(&line, "Z = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("Z")))
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // frame 
        let line = lines.next()
            .unwrap();
        let frame: String = match scan_fmt!(&line, "FRAME = {}", String) {
            Some(fr) => fr,
            _ => return Err(Error::FrameFormatError),
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // comments 
        let line = lines.next()
            .unwrap();
        let comments : Option<String> = match line.contains("NO COMMENTS") {
            true => None,
            false => {
                Some(String::from(line.strip_prefix("COMMENTS = ").unwrap().trim()))
            }
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // next line
        let mut line = lines.next()
            .unwrap();
        // system delays parsing
        let mut sys_dly : Option<CalibratedDelay> = None; 
        let mut int_dly : Option<CalibratedDelay> = None; 
        let mut tot_dly : Option<CalibratedDelay> = None; 
        let mut ref_dly = 0.0_f64; 
        let mut cab_dly = 0.0_f64; 

        while line.contains("DLY") {
            // determine delay denomination
            let label = match scan_fmt!(&line, "{} DLY =.*", String) {
                Some(label) => label,
                _ => return Err(Error::DelayIdentificationError(String::from(line))),
            };

            if label.eq("CAB") || label.eq("REF") { // carrier independent delay (simple)
                // parse value
                let start_off = line.find("=").unwrap();
                let end_off   = line.rfind("ns").unwrap();
                let cleanedup = &line[start_off+1..end_off];
                let value = f64::from_str(cleanedup.trim()).unwrap();
                if label.eq("CAB") {
                    cab_dly = value
                } else if label.eq("REF") {
                    ref_dly = value
                }
            } else { // is carrier dependent delay
                // 0. remove '{label} {dly} = '
                let mut cleanedup = line.strip_prefix(&label)
                    .unwrap();
                cleanedup = cleanedup.strip_prefix(" DLY = ")
                    .unwrap().trim();
                // 1. parse CAL ID 
                //  => for calibration report info
                //  => then remove it to ease up last content identification
                let offset = cleanedup.rfind("=")
                    .unwrap();
                let (before, after) = cleanedup.split_at(offset+1); 
                let report = String::from(after.trim());
                cleanedup = before.strip_suffix(" CAL_ID =")
                    .unwrap()
                    .trim();
                // final delay identification
                let (constellation, values, codes) : 
                    (track::Constellation, Vec<f64>, Vec<String>)
                    = match cleanedup.contains(",") 
                {
                    true => {
                        // (A) dual frequency: comma separated infos
                        let offset = cleanedup.find(",")
                            .unwrap();
                        let (content1, content2) = cleanedup.split_at(offset);
                        let content2 = content2.strip_prefix(",")
                            .unwrap()
                            .trim();
                        let (delay1, constellation, code1) = carrier_dependant_delay_parsing(content1)?; 
                        let (delay2, _, code2) = carrier_dependant_delay_parsing(content2)?; 
                        (constellation,vec![delay1,delay2],vec![code1,code2]) //codes)
                    },
                    false => {
                        // (B) single frequency: simple 
                        let (delay, constellation, code) = carrier_dependant_delay_parsing(cleanedup)?;
                        (constellation,vec![delay],vec![code])
                    }
                };
                // mapp to corresponding structure
                if label.eq("TOT") {
                    tot_dly = Some(CalibratedDelay::new(constellation, values, codes, Some(&report)))
                } else if label.eq("SYS") {
                    sys_dly = Some(CalibratedDelay::new(constellation, values, codes, Some(&report)))
                } else if label.eq("INT") {
                    int_dly = Some(CalibratedDelay::new(constellation, values, codes, Some(&report)))
                }
            }

            // crc
            cksum = cksum.wrapping_add(
                calc_crc(&line)?);
            // grab next
            line = lines.next()
                .unwrap();
        }
        let reference: String = match scan_fmt!(&line, "REF = {}", String) {
            Some(string) => string,
            _ => return Err(Error::ReferenceFormatError),
        };
        // crc
        cksum = cksum.wrapping_add(calc_crc(&line)?);
        // checksum
        let line = lines.next().unwrap();
        let ck : u8 = match scan_fmt!(&line, "CKSUM = {x}", String) {
            Some(s) => {
                match u8::from_str_radix(&s, 16) {
                    Ok(hex) => hex,
                    _ => return Err(Error::ChecksumParsingError),
                }
            },
            _ => return Err(Error::ChecksumFormatError),
        };
        // final crc
        let end_pos = line.find("= ")
            .unwrap(); // already matching
        cksum = cksum.wrapping_add(
            calc_crc(
                &line.split_at(end_pos+2).0)?);
        // checksum verification
        //if cksum != ck {
        //    return Err(Error::ChecksumError(ck, cksum))
        //}
        /* blank lines */
        let _ = lines.next().unwrap(); // Blank
        let _ = lines.next().unwrap(); // labels
        let _ = lines.next().unwrap(); // units currently discarded
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
            if let Ok(track) = track::CggttsTrack::from_str(&line) {
                tracks.push(track)
            }
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
}
