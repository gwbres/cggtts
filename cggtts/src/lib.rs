//! Rust package to parse, analyze and generate CGGTTS data.   
//! CGGTTS data files are dedicated to common view (two way satellite)
//! time transfer.
//!
//! Official BIPM `Cggtts` specifications:  
//! <https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf>
//!
//! Only "2E" Version (latest to date) is supported by this parser
//!
//!
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
//! use cggtts::{Rcvr, Cggtts};
//! use cggtts::track::Track;
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

mod crc;
mod time_system;
mod version;

pub mod delay;
pub mod processing;
pub mod track;

extern crate gnss_rs as gnss;

use hifitime::{Duration, Epoch, TimeScale};
use std::str::FromStr;
use strum_macros::EnumString;
use thiserror::Error;

use crate::delay::{Delay, SystemDelay};
use crate::track::CommonViewClass;
use crate::track::Track;
use gnss::prelude::{Constellation, SV};
use time_system::TimeSystem;
use version::Version;

use scan_fmt::scan_fmt;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

pub mod prelude {
    pub use crate::track::CommonViewClass;
    pub use crate::{track::Track, version::Version, Cggtts};
    pub use hifitime::prelude::Duration;
    pub use hifitime::prelude::Epoch;
    pub use hifitime::prelude::TimeScale;
    pub use rinex::prelude::Constellation;
    pub use rinex::prelude::SV;
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

/// Latest CGGTTS release : only version we truly support
pub const CURRENT_RELEASE: &str = "2E";

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Clone, Copy, PartialEq, Debug, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[cfg(feature = "serde")]
mod point3d_formatter {
    pub fn serialize<S>(p: rust_3d::Point3D, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{},{},{}", p.x, p.y, p.z);
        serializer.serialize_str(&s)
    }
}

/// `Cggtts` structure comprises
/// a measurement system and
/// and its Common View realizations (`tracks`)
#[derive(Debug, Clone)]
pub struct Cggtts {
    /// CGGTTS release used in this file.
    /// We currently only support 2E (latest)
    pub version: Version,
    /// Release date of this file revision.
    pub release_date: hifitime::Epoch,
    /// Laboratory / agency (data producer)
    pub lab: Option<String>,
    /// Possible GNSS receiver infos
    pub rcvr: Option<Rcvr>,
    /// # of GNSS receiver channels
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
    //#[cfg_attr(feature = "serde", serde(with = "point3d_formatter"))]
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

#[derive(Debug, Error)]
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
    #[error("non supported revision \"{0}\"")]
    NonSupportedRevision(String),
    #[error("failed to parse \"{0}\" coordinates")]
    CoordinatesParsingError(String),
    #[error("failed to identify delay value in line \"{0}\"")]
    DelayIdentificationError(String),
    #[error("failed to parse frequency dependent delay from \"{0}\"")]
    FrequencyDependentDelayParsingError(String),
    #[error("bad common view class")]
    BadCommonViewClass,
    #[error("checksum format error")]
    ChecksumFormatError,
    #[error("failed to parse checksum value")]
    ChecksumParsingError,
    #[error("file format error")]
    FormatError,
    #[error("header crc error")]
    ChecksumError(#[from] crc::Error),
    #[error("missing crc field")]
    CrcMissing,
    #[error("track parsing error")]
    TrackParsing(#[from] track::Error),
}

impl Default for Cggtts {
    /// Buils default `Cggtts` structure,
    fn default() -> Self {
        Self {
            version: Version::default(),
            release_date: Epoch::from_gregorian_utc_at_midnight(2014, 02, 20), /* latest rev. */
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
    /// Returns Self with desired "lab" agency
    pub fn lab_agency(&self, lab: &str) -> Self {
        let mut c = self.clone();
        c.lab = Some(lab.to_string());
        c
    }
    /// Returns ̀Self with desired number of channels
    pub fn nb_channels(&self, ch: u16) -> Self {
        let mut c = self.clone();
        c.nb_channels = ch;
        c
    }
    /// Returns Self with desired receiver info
    pub fn receiver(&self, rcvr: Rcvr) -> Self {
        let mut c = self.clone();
        c.rcvr = Some(rcvr);
        c
    }
    /// Returns Self with desired "ims" hardware info
    pub fn ims(&self, ims: Rcvr) -> Self {
        let mut c = self.clone();
        c.ims = Some(ims);
        c
    }
    /// Returns Self but with desired antenna phase center
    /// coordinates, coordinates should be in IRTF meters.
    pub fn antenna_coordinates(&self, coords: rust_3d::Point3D) -> Self {
        let mut c = self.clone();
        c.coordinates = coords;
        c
    }

    /// Returns `Cggtts` with desired reference time system description
    pub fn time_reference(&self, reference: TimeSystem) -> Self {
        let mut c = self.clone();
        c.time_reference = reference;
        c
    }

    /// Returns `Cggtts` with desired Reference Frame
    pub fn reference_frame(&self, reference: &str) -> Self {
        let mut c = self.clone();
        c.reference_frame = Some(reference.to_string());
        c
    }

    /// Returns true if all tracks follow
    /// BIPM tracking specifications
    pub fn follows_bipm_specs(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.follows_bipm_specs() {
                return false;
            }
        }
        true
    }

    /// Returns true if Self only contains tracks (measurements)
    /// that have ionospheric parameter estimates
    pub fn has_ionospheric_data(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.has_ionospheric_data() {
                return false;
            }
        }
        true
    }

    /// Returns common view class, used in this file.
    /// For a file to be SingleChannel, all tracks must be SingleChannel tracks,
    /// otherwise we consider MultiChannel
    pub fn common_view_class(&self) -> CommonViewClass {
        for trk in self.tracks.iter() {
            if trk.class != CommonViewClass::SingleChannel {
                return CommonViewClass::MultiChannel;
            }
        }
        CommonViewClass::SingleChannel
    }

    /// Returns true if Self is a single channel file
    pub fn single_channel(&self) -> bool {
        self.common_view_class() == CommonViewClass::SingleChannel
    }

    /// Returns true if Self is a multi channel file
    pub fn multi_channel(&self) -> bool {
        self.common_view_class() == CommonViewClass::MultiChannel
    }

    /// Returns true if Self is only made of melting pot tracks
    pub fn melting_pot(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.melting_pot() {
                return false;
            }
        }
        true
    }

    /// Returns true if Self is only made of single SV realizations
    pub fn single_sv(&self) -> bool {
        !self.melting_pot()
    }

    /// Returns true if Self has at least one track
    /// referenced against given constellation
    pub fn uses_constellation(&self, c: Constellation) -> bool {
        self.tracks
            .iter()
            .filter_map(|trk| {
                if trk.sv.constellation == c {
                    Some(trk)
                } else {
                    None
                }
            })
            .count()
            > 0
    }

    /// Returns track Iterator
    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

    /// Returns an iterator over CGGTTS tracks that were generated by tracking
    /// this vehicle
    pub fn sv_tracks(&self, sv: SV) -> impl Iterator<Item = &Track> {
        self.tracks
            .iter()
            .filter_map(move |trk| if trk.sv == sv { Some(trk) } else { None })
    }

    /// Returns an iterator over CGGTTS tracks that were generated by tracking
    /// this constellation
    pub fn constellation_tracks(&self, c: Constellation) -> impl Iterator<Item = &Track> {
        self.tracks.iter().filter_map(move |trk| {
            if trk.sv.constellation == c {
                Some(trk)
            } else {
                None
            }
        })
    }

    /// Returns true if Self was generated by tracking a uique constellation
    pub fn unique_constellation(&self, c: Constellation) -> bool {
        for track in self.tracks.iter() {
            if !track.uses_constellation(c) {
                return false;
            }
        }
        true
    }

    /// Returns Epoch of this file's generation
    pub fn epoch(&self) -> Option<Epoch> {
        self.tracks.first().map(|trk| trk.epoch)
    }

    /// Returns total set duration,
    /// by cummulating all measurements duration
    pub fn total_duration(&self) -> Duration {
        let mut dt = Duration::default();
        for trk in self.tracks.iter() {
            dt += trk.duration;
        }
        dt
    }

    /// Returns a filename that would match
    /// specifications / standard requirements
    /// to represent self.
    /// If lab is not known, we set XX.   
    /// We use the first two characters for the Rcvr hardware
    /// info as identification number (only if they are digits).
    /// We replace by "__" otherwise.
    pub fn filename(&self) -> String {
        let mut res = String::new();

        let constellation = match self.tracks.first() {
            Some(track) => track.sv.constellation,
            None => Constellation::default(),
        };
        res.push_str(&format!("{:x}", constellation));

        if self.has_ionospheric_data() {
            res.push_str("Z") // Dual Freq / Multi channel
        } else {
            if self.single_channel() {
                res.push_str("S") // Single Freq / Channel
            } else {
                res.push_str("M") // Single Freq / Multi Channel
            }
        }

        if let Some(lab) = &self.lab {
            res.push_str(&lab[0..2])
        } else {
            res.push_str("XX") // unknown lab
        }

        if let Some(rcvr) = &self.rcvr {
            let sn = rcvr.serial_number.as_str();
            if let Ok(d0) = u16::from_str_radix(&sn[0..0], 10) {
                res.push_str(&d0.to_string());
                if let Ok(d1) = u16::from_str_radix(&sn[1..1], 10) {
                    res.push_str(&d1.to_string())
                } else {
                    res.push_str("_")
                }
            } else {
                res.push_str("__")
            }
        } else {
            res.push_str("__")
        }

        if let Some(epoch) = self.epoch() {
            let mjd = epoch.to_mjd_utc_days();
            res.push_str(&format!(
                "{:02}.{:03}",
                mjd.floor() as u32,
                mjd.fract() as u32
            ));
        } else {
            res.push_str("YY.YYY")
        }

        res
    }

    /// Builds Self from given `Cggtts` file.
    pub fn from_file(fp: &str) -> Result<Self, Error> {
        let file_content = std::fs::read_to_string(fp)?;
        let mut lines = file_content.lines().into_iter();

        // init variables
        let mut system_delay = SystemDelay::new();

        let mut cksum: u8 = crc::calc_crc(lines.next().ok_or(Error::CrcMissing)?)?;

        let mut line = lines.next().ok_or(Error::VersionFormatError)?;
        let mut release_date = Epoch::default();
        let mut nb_channels: u16 = 0;
        let mut rcvr: Option<Rcvr> = None;
        let mut ims: Option<Rcvr> = None;
        let mut lab: Option<String> = None;
        let mut comments: Vec<String> = Vec::new();
        let mut reference_frame: Option<String> = None;
        let mut time_reference = TimeSystem::default();
        let (mut x, mut y, mut z): (f64, f64, f64) = (0.0, 0.0, 0.0);

        // VERSION must come first
        let version = lines.next().ok_or(Error::VersionFormatError)?;

        let version = match scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => Version::from_str(&version)?,
            _ => return Err(Error::VersionFormatError),
        };

        line = lines.next().ok_or(Error::FormatError)?;

        while let Some(line) = lines.next() {
            if line.starts_with("REV DATE = ") {
                match scan_fmt!(&line, "REV DATE = {d}-{d}-{d}", i32, u8, u8) {
                    (Some(y), Some(m), Some(d)) => {
                        release_date = Epoch::from_gregorian_utc_at_midnight(y, m, d);
                    },
                    _ => {},
                }
            } else if line.starts_with("RCVR = ") {
                match scan_fmt!(
                    &line,
                    "RCVR = {} {} {} {d} {}",
                    String,
                    String,
                    String,
                    String,
                    String
                ) {
                    (
                        Some(manufacturer),
                        Some(recv_type),
                        Some(serial_number),
                        Some(year),
                        Some(release),
                    ) => {
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
            } else if line.starts_with("CH = ") {
                match scan_fmt!(&line, "CH = {d}", u16) {
                    Some(n) => nb_channels = n,
                    _ => {},
                };
            } else if line.starts_with("IMS = ") {
                match scan_fmt!(
                    &line,
                    "IMS = {} {} {} {d} {}",
                    String,
                    String,
                    String,
                    String,
                    String
                ) {
                    (
                        Some(manufacturer),
                        Some(recv_type),
                        Some(serial_number),
                        Some(year),
                        Some(release),
                    ) => {
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
                    Some(s) => lab = Some(String::from(s.trim())),
                    _ => {},
                }
            } else if line.starts_with("X = ") {
                match scan_fmt!(&line, "X = {f}", f64) {
                    Some(f) => x = f,
                    _ => {},
                }
            } else if line.starts_with("Y = ") {
                match scan_fmt!(&line, "Y = {f}", f64) {
                    Some(f) => y = f,
                    _ => {},
                }
            } else if line.starts_with("Z = ") {
                match scan_fmt!(&line, "Z = {f}", f64) {
                    Some(f) => z = f,
                    _ => {},
                }
            } else if line.starts_with("FRAME = ") {
                let frame = line.split_at(7).1.trim();
                if !frame.eq("?") {
                    reference_frame = Some(frame.to_string())
                }
            } else if line.starts_with("COMMENTS = ") {
                let c = line.strip_prefix("COMMENTS =").unwrap().trim();
                if !c.eq("NO COMMENTS") {
                    comments.push(c.to_string())
                }
            } else if line.starts_with("REF = ") {
                if let Some(s) = scan_fmt!(&line, "REF = {}", String) {
                    time_reference = TimeSystem::from_str(&s)
                }
            } else if line.contains("DLY = ") {
                let items: Vec<&str> = line.split_ascii_whitespace().collect();

                let dual_carrier = line.contains(",");

                if items.len() < 4 {
                    continue; // format mismatch
                }

                match items[0] {
                    "CAB" => system_delay.rf_cable_delay = f64::from_str(items[3])?,
                    "REF" => system_delay.ref_delay = f64::from_str(items[3])?,
                    "SYS" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind("=").unwrap();
                            let cal_id = line[offset + 1..].trim();
                            if !cal_id.eq("NA") {
                                system_delay = system_delay.with_calibration_id(cal_id)
                            }
                        }
                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(")", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                        } else {
                            let value = f64::from_str(items[3]).unwrap();
                            let code = items[6].replace(")", "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay.delays.push((code, Delay::System(value)));
                            }
                        }
                    },
                    "INT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind("=").unwrap();
                            let cal_id = line[offset + 1..].trim();
                            if !cal_id.eq("NA") {
                                system_delay = system_delay.with_calibration_id(cal_id)
                            }
                        }
                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[10].replace(")", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value)));
                                }
                            }
                        } else {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace(")", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value)));
                                }
                            }
                        }
                    },
                    "TOT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind("=").unwrap();
                            let cal_id = line[offset + 1..].trim();
                            if !cal_id.eq("NA") {
                                system_delay = system_delay.with_calibration_id(cal_id)
                            }
                        }
                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(")", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                        } else {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace(")", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                        }
                    },
                    _ => {}, // non recognized delay type
                };
            } else if line.starts_with("CKSUM = ") {
                let _ck: u8 = match scan_fmt!(&line, "CKSUM = {x}", String) {
                    Some(s) => match u8::from_str_radix(&s, 16) {
                        Ok(hex) => hex,
                        _ => return Err(Error::ChecksumParsingError),
                    },
                    _ => return Err(Error::ChecksumFormatError),
                };

                // check CRC
                let end_pos = line.find("= ").unwrap();
                cksum = cksum.wrapping_add(crc::calc_crc(&line.split_at(end_pos + 2).0)?);

                //if cksum != ck {
                //    return Err(Error::ChecksumError(ck, cksum))
                //}
                break;
            }

            // CRC
            cksum = cksum.wrapping_add(crc::calc_crc(&line)?);

            if let Some(l) = lines.next() {
                line = l
            } else {
                break;
            }
        }

        // BLANKS
        let _ = lines.next(); // Blank
        let _ = lines.next(); // labels
        let _ = lines.next(); // units currently discarded
                              // tracks parsing
        let mut tracks: Vec<Track> = Vec::new();
        loop {
            let line = match lines.next() {
                Some(s) => s,
                _ => break, // we're done parsing
            };
            if line.len() == 0 {
                // empty line
                break; // we're done parsing
            }

            let track = Track::from_str(&line)?;
            tracks.push(track)
        }

        Ok(Cggtts {
            version,
            release_date,
            nb_channels,
            rcvr,
            ims,
            lab,
            reference_frame,
            coordinates: rust_3d::Point3D { x, y, z },
            comments: {
                if comments.len() == 0 {
                    None
                } else {
                    Some(comments)
                }
            },
            delay: system_delay,
            time_reference,
            tracks,
        })
    }
}

impl std::fmt::Display for Cggtts {
    /// Writes self into a `Cggtts` file
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let LATEST_REVISION_DATE: Epoch = Epoch::from_gregorian_utc_at_midnight(2014, 02, 20);

        /*
         * Labels in case we provide Ionospheric parameters estimates
         */
        let TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
        "SAT CL MJD STTIME TRKL ELV AZTH REFSV SRSV REFSYS SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

        /*
         * Labels in case Ionospheric compensation is not available
         */
        let TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
            "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

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
        let constellation = if self.tracks.len() > 0 {
            self.tracks[0].sv.constellation
        } else {
            Constellation::default()
        };
        if delays.len() == 1 {
            // Single frequency
            let (code, value) = delays[0];
            match value {
                Delay::Internal(v) => {
                    content.push_str(&format!(
                        "INT DLY = {:.1} ns ({:X} {})",
                        v, constellation, code
                    ));
                },
                Delay::System(v) => {
                    content.push_str(&format!(
                        "SYS DLY = {:.1} ns ({:X} {})",
                        v, constellation, code
                    ));
                },
            }
            if let Some(cal_id) = &self.delay.cal_id {
                content.push_str(&format!("       CAL_ID = {}\n", cal_id));
            } else {
                content.push_str("       CAL_ID = NA\n");
            }
        } else if delays.len() == 2 {
            // Dual frequency
            let (c1, v1) = delays[0];
            let (c2, v2) = delays[1];
            match v1 {
                Delay::Internal(_) => {
                    content.push_str(&format!(
                        "INT DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})",
                        v1.value(),
                        constellation,
                        c1,
                        v2.value(),
                        constellation,
                        c2
                    ));
                },
                Delay::System(_) => {
                    content.push_str(&format!(
                        "SYS DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})",
                        v1.value(),
                        constellation,
                        c1,
                        v2.value(),
                        constellation,
                        c2
                    ));
                },
            }
            if let Some(cal_id) = &self.delay.cal_id {
                content.push_str(&format!("     CAL_ID = {}\n", cal_id));
            } else {
                content.push_str("     CAL_ID = NA\n");
            }
        }

        content.push_str(&format!("CAB DLY = {:.1} ns\n", self.delay.rf_cable_delay));

        content.push_str(&format!("REF DLY = {:.1} ns\n", self.delay.ref_delay));

        content.push_str(&format!("REF = {}\n", self.time_reference));

        let crc = crc::calc_crc(&content)?;
        content.push_str(&format!("CKSUM = {:2X}\n", crc));
        content.push_str("\n"); // blank

        if self.has_ionospheric_data() {
            content.push_str(TRACK_LABELS_WITH_IONOSPHERIC_DATA);
            content.push_str("\n");
            content.push_str("              hhmmss s .1dg .1dg .1ns .1ps/s .1ns .1ps/s .1ns .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns\n")
        } else {
            content.push_str(TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA);
            content.push_str("\n");
            content.push_str("             hhmmss s   .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s\n")
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
        let expected = vec![0x0F, 0x71];
        for i in 0..content.len() {
            let ck = calc_crc(content[i]).unwrap();
            let expect = expected[i];
            assert_eq!(
                ck, expect,
                "Failed for \"{}\", expect \"{}\" but \"{}\" locally computed",
                content[i], expect, ck
            )
        }
    }

    #[test]
    fn test_time_system() {
        assert_eq!(TimeSystem::default(), TimeSystem::UTC);
        assert_eq!(TimeSystem::from_str("TAI"), TimeSystem::TAI);
        assert_eq!(TimeSystem::from_str("UTC"), TimeSystem::UTC);
        assert_eq!(
            TimeSystem::from_str("UTC(LAB)"),
            TimeSystem::UTCk(String::from("LAB"), None)
        );
        assert_eq!(
            TimeSystem::from_str("UTC(LAB, 10.0)"),
            TimeSystem::UTCk(String::from("LAB"), Some(10.0))
        );
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
