//! CGGTTS Rust package
//!
//! Rust package to parse and manipulate CGGTTS data files.
//! CGGTTS data files are dedicated to common view (two way satellite)
//! time transfer.
//!
//! Official BIPM `Cggtts` specifications:
//! <https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf>
//!
//! Only single frequency `Cggtts` is supported at the moment,
//! dual frequency `Cggtts` to be supported in coming releases.
//! 
//! Only "2E" Version (latest to date) supported
//!
//! Homepage: <https://github.com/gwbres/cggtts>

pub mod track;
use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

/// supported `Cggtts` version,
/// non matching input files will be rejected
const VERSION: &str = "2E";

/// latest revision date
const REV_DATE: &str = "2014-02-20";

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

#[derive(Debug)]
/// `CalibratedDelay` are delays that are
/// specified to a specific carrier frequency,
/// thefore, to a specific `GNSS` constellation.
/// Some extra information regarding calibration process
/// might be avaiable
pub struct CalibratedDelay {
    constellation: track::Constellation, // specific constellation 
    values: Vec<f64>, // actual value
    codes: Vec<String>, // rinex carrier codes 
    report: String, // calibration report
}

impl Default for CalibratedDelay {
    fn default() -> CalibratedDelay {
        CalibratedDelay {
            constellation: track::Constellation::default(),
            values: Vec::new(), 
            codes: Vec::new(),
            report: String::from("NA"),
        }
    }
}

impl CalibratedDelay {
    /// Builds a new `CalibratedDelay` object
    pub fn new(constellation: track::Constellation, values: Vec<f64>, codes: Vec<String>, report: &str) -> CalibratedDelay {
        CalibratedDelay {
            constellation,
            values,
            codes,
            report: String::from(report),
        }
    }
    /// Returns constellation against which this delay
    /// has been estimated
    pub fn get_constellation (&self) -> track::Constellation { self.constellation }
    /// Returns estimated delay values
    pub fn get_values (&self) -> &Vec<f64> { &self.values }
    /// Returns carrier identification codes for which this delay was estimated
    pub fn get_codes (&self) -> &Vec<String> { &self.codes }
    /// Returns true if self has some extra information related
    /// to the calibration process
    pub fn has_calibration_report (&self) -> bool { !self.report.eq("NA") }
    /// Returns calibration info
    pub fn get_calibration_report (&self) -> &str { &self.report }
}

/// Identifies carrier dependant informations
/// from a string shaped like '53.9 ns (GLO C1)'
fn carrier_dependant_delay_parsing (string: &str) 
        -> Result<(f64,track::Constellation,String),Error> 
{
    let (delay, const_str, code) : (f64, String, String) = match scan_fmt!(string, "{f} ns ({} {})", f64, String, String) {
        (Some(delay),Some(constellation),Some(code)) => (delay,constellation,code),
        _ => return Err(Error::FrequencyDependentDelayParsingError(String::from(string)))
    };
    let mut constellation: track::Constellation = track::Constellation::default();
    if const_str.eq("GPS") {
        constellation = track::Constellation::GPS
    } else if const_str.eq("GLO") {
        constellation = track::Constellation::Glonass
    } else if const_str.eq("BDS") {
        constellation = track::Constellation::Beidou
    } else if const_str.eq("GAL") {
        constellation = track::Constellation::Galileo
    } else if const_str.eq("QZS") {
        constellation = track::Constellation::QZSS
    }
    Ok((delay,constellation,code))
}

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
    tot_dly: Option<CalibratedDelay>,
    int_dly: Option<CalibratedDelay>,
    sys_dly: Option<CalibratedDelay>,
    cab_dly: f64,
    ref_dly: f64,
    reference: String, // reference time
    tracks: Vec<track::CggttsTrack> // CGGTTS track(s)
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
            version: VERSION.to_string(),
            rev_date: chrono::NaiveDate::parse_from_str(REV_DATE, "%Y-%m-%d")
                .unwrap(),
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

    /// Evaluates total system delay as `CalibratedDelay`.
    /// 
    /// If no system delays were specified by user
    /// or parsed from a file: this returns a null + uncalibrated. 
    /// 
    /// Returns delay in case some specific
    /// values were specified .
    ///
    /// In more advanced usage, returns the combination
    /// of all delays for each carrier frequencies
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
                            None => {} // no delay at all, 0 assumed then
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
    pub fn get_first_track (&self) -> Option<&track::CggttsTrack> { self.tracks.get(0) }
    /// Returns last track produced in file (if any)
    pub fn get_latest_track (&self) -> Option<&track::CggttsTrack> { self.tracks.get(self.tracks.len()-1) }
    /// Returns requested track (if possible)
    pub fn get_track (&self, index: usize) -> Option<&track::CggttsTrack> { self.tracks.get(index) }
    /// Grabs last track from self (if possible)
    pub fn pop_track (&mut self) -> Option<track::CggttsTrack> { self.tracks.pop() }
    /// Appends one track to self (if possible)
    pub fn push_track (&mut self, track: track::CggttsTrack) { self.tracks.push(track) }

    /// returns true if self is `Dual Frequency Cggtts`
    pub fn is_dual_frequency (&self) -> bool { self.total_delay().values.len() == 1 }

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
        let mut cksum: u8 = track::calc_crc(&line)?;
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
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
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
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
        // channel
        let line = lines.next().unwrap();
        let nb_channels: u16 = match scan_fmt!(&line, "CH = {d}", u16) {
            Some(channel) => channel,
            _ => return Err(Error::ChannelFormatError),
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
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
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
        // lab
        let line = lines.next()
            .unwrap();
        let lab: String = match line.strip_prefix("LAB = ") {
            Some(s) => String::from(s.trim()),
            _ => return Err(Error::LabParsingError),
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
        // X
        let line = lines.next().unwrap();
        let x: f32 = match scan_fmt!(&line, "X = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("X")))
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
        // Y
        let line = lines.next()
            .unwrap();
        let y: f32 = match scan_fmt!(&line, "Y = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("Y")))
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
        // Z
        let line = lines.next()
            .unwrap();
        let z: f32 = match scan_fmt!(&line, "Z = {f}", f32) {
            Some(f) => f,
            _ => return Err(Error::CoordinatesParsingError(String::from("Z")))
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
        // frame 
        let line = lines.next()
            .unwrap();
        let frame: String = match scan_fmt!(&line, "FRAME = {}", String) {
            Some(fr) => fr,
            _ => return Err(Error::FrameFormatError),
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
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
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
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
                println!("CLEANED UP '{}'", cleanedup);
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
                        let (delay2, constellation, code2) = carrier_dependant_delay_parsing(content2)?; 
                        //let mut values : Vec<f64> = Vec::new();
                        //values.push(delay1);
                        //values.push(delay2);
                        //let mut codes  : Vec<String> = Vec::new();
                        //codes.push(code1);
                        //codes.push(code2);
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
                    tot_dly = Some(CalibratedDelay::new(constellation, values, codes, &report))
                } else if label.eq("SYS") {
                    sys_dly = Some(CalibratedDelay::new(constellation, values, codes, &report))
                } else if label.eq("INT") {
                    int_dly = Some(CalibratedDelay::new(constellation, values, codes, &report))
                }
            }

            // crc
            cksum = cksum.wrapping_add(
                track::calc_crc(&line)?);
            // grab next
            line = lines.next()
                .unwrap();
        }
        let reference: String = match scan_fmt!(&line, "REF = {}", String) {
            Some(string) => string,
            _ => return Err(Error::ReferenceFormatError),
        };
        // crc
        cksum = cksum.wrapping_add(track::calc_crc(&line)?);
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
            track::calc_crc(
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
        //assert_eq!(cggtts.tot_dly, None);
        //assert_eq!(cggtts.ref_dly, None);
        //assert_eq!(cggtts.int_dly, None);
        //assert_eq!(cggtts.cab_dly, None);
        //assert_eq!(cggtts.sys_dly, None);
        assert_eq!(cggtts.coordinates, (0.0,0.0,0.0));
        assert_eq!(cggtts.rev_date, rev_date); 
        assert_eq!(cggtts.date, today.naive_utc());
        println!("{:#?}", cggtts)
    }

    #[test]
    /// Tests basic usage 
    fn cggtts_basic_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        //cggtts.set_total_delay(300E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        //assert_eq!(cggtts.get_system_delay().is_none(), true); // not provided
        //assert_eq!(cggtts.get_cable_delay().is_none(), true); // not provided
        //assert_eq!(cggtts.get_reference_delay().is_none(), true); // not provided
        //assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        //assert_eq!(cggtts.get_total_delay().unwrap(), 300E-9); // basic usage
        println!("{:#?}", cggtts)
    }

    #[test]
    /// Test normal / intermediate usage
    fn cgggts_intermediate_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        //cggtts.set_reference_delay(100E-9);
        //cggtts.set_system_delay(150E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        //assert_eq!(cggtts.get_cable_delay().is_some(), false); // not provided
        //assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_system_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        //assert_eq!(cggtts.get_total_delay().unwrap(), 250E-9); // intermediate usage
        println!("{:#?}", cggtts)
    }

    #[test]
    /// Test advanced usage
    fn cgggts_advanced_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        //cggtts.set_reference_delay(100E-9);
        //cggtts.set_internal_delay(25E-9);
        //cggtts.set_cable_delay(300E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        //assert_eq!(cggtts.get_system_delay().is_some(), false); // not provided: we have granularity
        //assert_eq!(cggtts.get_cable_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_internal_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        /*assert!(
            approx_eq!(f64,
                cggtts.get_total_delay().unwrap(),
                425E-9, // advanced usage
                epsilon = 1E-9
            )
        );*/
        println!("{:#?}", cggtts)
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
                    "Cggtts::from_file() failed for {:#?} with {:#?}",
                    path,
                    cggtts);
                println!("File \"{:?}\" {:#?}", &path, cggtts.unwrap())
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
                    "Cggtts::from_file() failed for {:#?} with {:#?}",
                    path, 
                    cggtts);
                println!("File \"{:?}\" {:#?}", &path, cggtts.unwrap())
            }
        }
    }
}
