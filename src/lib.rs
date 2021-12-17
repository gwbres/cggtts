//! CGGTTS Rust package
//!
//! A package to handle CGGTTS data files.
//! Only 2E Version (latest) supported
//!
//! url: https://github.com/gwbres/cggtts
//!
//! Refer to official doc: 
//! https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf

use std::fmt;
use regex::Regex;
use thiserror::Error;
use scan_fmt::scan_fmt;

/// CGGTTS track description
mod track;

/// supported CGGTTS version
const VERSION: &str = "2E";

/// CGGTTS structure
#[derive(Debug)]
pub struct Cggtts {
    version: String, // file version info
    rev_date: chrono::NaiveDate, // header data rev. date
    date: chrono::NaiveDate, // production date
    lab: String, // LAB where measurements were performed (possibly unknown)
    recvr: Rcvr, // possible GNSS receiver infos
    nb_channels: u16, // nb of GNSS receiver channels
    ims: Option<Rcvr>, // IMS Ionospheric Measurement System (if any)
    xyz: (f32,f32,f32), // antenna phase center coordinates [in m]
    frame: String,
    comments: Option<String>, // comments (if any)
    // delays
    //  sys delay: total system delay [internal + cable delay] [ns]
    //  cable delay: delay from antenna to receiver [ns]
    //  ref. delay: receiver to local clock delay [ns]
    delays: (f64,f64,f64), 
    reference: String,
    tracks: Vec<track::CggttsTrack>
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
pub enum CggttsError {
    #[error("Failed to parse file")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("File naming convention")]
    FileNamingConvention,
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
    DelayFormatError(String),
    #[error("File Checksum error - got '{0}' but '{1}' locally computed")]
    ChecksumError(u8, u8),
}

impl Cggtts {
    /// Builds CGGTTS from given file
    pub fn new(fp: &std::path::Path) -> Result<Cggtts, CggttsError> {
        let file_name = fp.file_name()
            .unwrap()
            .to_str()
                .unwrap();
        // regex to check for file naming convetion
        let file_re = Regex::new(r"(G|R|E|C|J)(S|M|Z)....[1-9][1-9]\.[1-9][1-9][1-9]")
            .unwrap();
        if !file_re.is_match(file_name) {
            return Err(CggttsError::FileNamingConvention)
        }

        let mut chksum: u8 = 0;
        let file_content = std::fs::read_to_string(&fp).unwrap();
        let mut lines = file_content.split("\n").map(|x| x.to_string()).into_iter();

        // Identify date from MJD contained in file name
        let mjd: f32 = 10.0; //mjd_str.parse().unwrap();

        // version
        let line = lines.next().unwrap();
        let _ = match scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => {
                if !version.eq(&VERSION) {
                    return Err(CggttsError::DeprecatedVersion)
                }
            },
            _ => return Err(CggttsError::VersionFormatError),
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
                    _ => return Err(CggttsError::RevisionDateParsingError),
                }
            },
            _ => return Err(CggttsError::RevisionDateFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // rcvr
        let line = lines.next().unwrap();
        let rcvr_infos: Rcvr = match scan_fmt! (&line, "RCVR = {} {} {} {d} {}", String, String, String, String, String) {
            (Some(manufacturer),
            Some(recv_type),
            Some(serial_number),
            Some(year),
            Some(software_number)) => Rcvr{
                manufacturer, 
                recv_type, 
                serial_number, 
                year: u16::from_str_radix(&year, 10)?, 
                software_number
            },
            _ => return Err(CggttsError::RcvrFormatError),
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
            _ => return Err(CggttsError::ChannelFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // ims
        let line = lines.next().unwrap();
        let ims_infos: Option<Rcvr> = match line.contains("IMS = 99999") { 
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
                    _ => return Err(CggttsError::ImsFormatError),
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
            _ => return Err(CggttsError::LabParsingError),
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
            _ => return Err(CggttsError::CoordinatesParsingError(String::from("X")))
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
            _ => return Err(CggttsError::CoordinatesParsingError(String::from("Y")))
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
            _ => return Err(CggttsError::CoordinatesParsingError(String::from("Z")))
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
            _ => return Err(CggttsError::FrameFormatError),
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
            _ => return Err(CggttsError::CommentsFormatError),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // system delay 
        let line = lines.next().unwrap();
        let sys_dly: f64 = match scan_fmt!(&line, "SYS DLY = {f} {} {}", f64, String, String) {
            (Some(dly),Some(scaling),Some(_)) => {
                if scaling.eq("ms") {
                    dly * 1E-3
                } else if scaling.eq("us") {
                    dly * 1E-6
                } else if scaling.eq("ns") {
                    dly * 1E-9
                } else if scaling.eq("ps") {
                    dly * 1E-12
                } else if scaling.eq("fs") {
                    dly * 1E-15
                } else {
                    dly
                }
            },
            _ => return Err(CggttsError::DelayFormatError(String::from("System"))),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // cable delay
        let line = lines.next().unwrap();
        let cab_dly: f64 = match scan_fmt!(&line, "CAB DLY = {f} {}", f64, String) {
            (Some(dly),Some(scaling)) => {
                if scaling.eq("ms") {
                    dly * 1E-3
                } else if scaling.eq("us") {
                    dly * 1E-6
                } else if scaling.eq("ns") {
                    dly * 1E-9
                } else if scaling.eq("ps") {
                    dly * 1E-12
                } else if scaling.eq("fs") {
                    dly * 1E-15
                } else {
                    dly
                }
            },
            _ => return Err(CggttsError::DelayFormatError(String::from("Cable"))),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }
        // ref. delay
        let line = lines.next().unwrap();
        let ref_dly: f64 = match scan_fmt!(&line, "REF DLY = {f} {}", f64, String) {
            (Some(dly),Some(scaling)) => {
                if scaling.eq("ms") {
                    dly * 1E-3
                } else if scaling.eq("us") {
                    dly * 1E-6
                } else if scaling.eq("ns") {
                    dly * 1E-9
                } else if scaling.eq("ps") {
                    dly * 1E-12
                } else if scaling.eq("fs") {
                    dly * 1E-15
                } else {
                    dly
                }
            },
            _ => return Err(CggttsError::DelayFormatError(String::from("Ref."))),
        };
        // crc
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // REFERENCE
        let line = lines.next().unwrap();
        let reference = scan_fmt!(&line, "REF = {}", String).unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // CKSUM
        let line = lines.next().unwrap();
        let cksum_parsed = scan_fmt!(&line, "CKSUM = {x}", String).unwrap();
        let cksum = u8::from_str_radix(&cksum_parsed, 16).unwrap();

        // CRC calc. ends on 'CHKSUM = ' (line 15)
        let end_pos = line.find("= ").unwrap();
        let bytes = line.clone().into_bytes();
        for i in 0..end_pos+2
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        chksum = chksum.wrapping_add(15*10); // 15*'\n', TODO: faulty syref2-5.c ??
        if chksum != cksum {
            return Err(CggttsError::ChecksumError(cksum, chksum))
        }

        let _ = lines.next().unwrap(); // Blank line
        let _ = lines.next().unwrap(); // labels line
        let _ = lines.next().unwrap(); // units line currently discarded, TODO: to be improved

        // Parsing tracks
        let mut tracks: Vec<track::CggttsTrack> = Vec::new();
        loop {
            // grab new line
            let line = match lines.next() {
                Some(s) => s,
                _ => break // we're done parsing
            };
            if line.len() == 0 { // empty line
                break // we're done parsing
            }
            tracks.push (track::CggttsTrack::new(&line).unwrap());
        }

        return Ok(Cggtts {
                version: VERSION.to_string(),
                rev_date,
                date: mjd_to_date((mjd * 1000.0) as u32), /* TODO @GBR conversion mjd est a revoir */
                nb_channels,
                recvr: rcvr_infos,
                ims: ims_infos,
                lab,
                xyz: (x,y,z),
                frame,
                comments,
                delays: (sys_dly, cab_dly, ref_dly),
                reference,
                tracks
            }
        );
    }

    pub fn get_date (&self) -> &chrono::NaiveDate { &self.date }

    /* retuns requested track in self */
    pub fn get_track (&self, index: usize) -> Option<&track::CggttsTrack> { self.tracks.get(index) }

    /* returns latest track to date */
    pub fn get_latest_track (&self) -> Option<&track::CggttsTrack> {
        self.tracks.get(self.tracks.len()-1)
    }

    /* returns earlist track in date */
    pub fn get_earliest_track (&self) -> Option<&track::CggttsTrack> {
        self.tracks.get(0)
    }
}

// custom display formatter
impl fmt::Display for Cggtts {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "Version: '{}' | REV DATE '{}' | LAB '{}' | Nb Channels {}\nRECVR: {:?}\nIMS: {:?}\nXYZ: {:?}\nFRAME: {}\nCOMMENTS: {:#?}\nDELAYS {:?} [ns]\nREFERENCE: {}\n",
            self.version, self.rev_date, self.lab, self.nb_channels,
            self.recvr,
            self.ims,
            self.xyz,
            self.frame,
            self.comments,
            self.delays,
            self.reference,
        ).unwrap();
        write! (f, "-------------------------\n").unwrap();
        for i in 0..self.tracks.len() {
            write! (f, "MEAS #{}: {}\n",i, self.tracks[i]).unwrap()
        }
        write!(f, "\n")
    }
}

/* converts MJD to chrono::naivedate */
fn mjd_to_date (mjd: u32) -> chrono::NaiveDate {
    let z = 2400000.5 + 0.5 + mjd as f32; // julian day 0h
    let alpha: u32 = ((z - 867216.25) /  36524.25) as u32;
    let s: u32 = z as u32 +1 +alpha - alpha / 4;
    let b: u32 = s + 1524;
    let c: u32 = ((b as f32 -122.1)/365.25) as u32;
    let d: u32 = (365.25 * c as f32) as u32;
    let e: u32 = ((b-d) as f32 / 30.6001) as u32;
    let q = b - d - ((30.6001 * e as f32) as u32);
    let m: u32;
    let mut annee: u32 = c - 4715;

    if e < 14 {
        m = e-1;
    } else {
        m = e-13;
    }

    if m > 2 {
        annee = c - 4716;
    }

    return chrono::NaiveDate::from_ymd(annee as i32, m, q)
}
#[cfg(test)]
mod test {
    use chrono::Datelike;
    use super::*;

    #[test]
    /*
     * Tests lib against test resources
     */
    fn cggtts_constructor() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                assert_eq!(
                    Cggtts::new(&fp).is_err(),
                    false,
                    "Cggtts::new() failed for '{:?}' with '{:?}'",
                    path, 
                    Cggtts::new(&fp))
            }
        }
    }
    
    #[test]
    /*
     * Tests MJD convertion
     */
    fn test_mdj_convertion () {
        let date = mjd_to_date(57000);
        assert_eq!(date.year(), 2014);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 9);
    }
}
