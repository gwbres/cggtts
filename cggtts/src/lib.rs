#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

// #[cfg(feature = "tracker")]
// #[cfg_attr(docsrs, doc(cfg(feature = "tracker")))]
// pub mod tracker;

extern crate gnss_rs as gnss;

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch};
use itertools::Itertools;
use scan_fmt::scan_fmt;
use strum_macros::EnumString;
use thiserror::Error;

use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    str::FromStr,
};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::{
    delay::{Delay, SystemDelay},
    hardware::Hardware,
    reference_time::ReferenceTime,
    track::{CommonViewClass, Track},
    version::Version,
};

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[derive(PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub mod prelude {
    pub use crate::cv::CommonViewPeriod;
    pub use crate::hardware::Hardware;
    pub use crate::reference_time::ReferenceTime;
    pub use crate::track::{CommonViewClass, IonosphericData, Track, TrackData};
    pub use crate::version::Version;
    pub use crate::CGGTTS;
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::prelude::{Duration, Epoch, TimeScale};

    // #[cfg(feature = "scheduler")]
    // #[cfg_attr(docsrs, doc(cfg(feature = "scheduler")))]
    // pub use tracker::{FitData, SVTracker};
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Latest CGGTTS release : only version we truly support
pub const CURRENT_RELEASE: &str = "2E";

#[derive(Clone, Copy, PartialEq, Debug, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub enum Code {
    #[default]
    C1,
    C2,
    P1,
    P2,
    E1,
    E5,
    B1,
    B2,
}

impl std::fmt::Display for Code {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Code::C1 => fmt.write_str("C1"),
            Code::C2 => fmt.write_str("C2"),
            Code::P1 => fmt.write_str("P1"),
            Code::P2 => fmt.write_str("P2"),
            Code::E1 => fmt.write_str("E1"),
            Code::E5 => fmt.write_str("E5"),
            Code::B1 => fmt.write_str("B1"),
            Code::B2 => fmt.write_str("B2"),
        }
    }
}

/// [CGGTTS] is a structure that holds a list of [Track]s that describe
/// the behavior of a local clock compared to a satellite onboard clock.
/// The [Track]s were solved by tracking satellites individually.
/// Remote clock comparison is then achieved by exchanging [CGGTTS]
/// between synchronous remote sites and running the common view comparison.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CGGTTS {
    /// CGGTTS [Version] used at production time of this [CGGTTS].
    pub version: Version,
    /// Release date of this file revision.
    pub release_date: Epoch,
    /// Station name, usually the data producer (agency, laboratory..).
    pub station: String,
    /// Possible GNSS receiver infos
    pub receiver: Option<Hardware>,
    /// # of GNSS receiver channels
    pub nb_channels: u16,
    /// IMS Ionospheric Measurement System (if any)
    pub ims_hardware: Option<Hardware>,
    /// Description of Reference time system (if any)
    pub reference_time: ReferenceTime,
    /// Reference frame, coordinates system and conversions,
    /// used in `coordinates` field
    pub reference_frame: Option<String>,
    /// Antenna phase center coordinates [m]
    /// in `ITFR`, `ECEF` or other spatial systems
    pub apc_coordinates: Coordinates,
    /// Comments (if any..)
    pub comments: Option<String>,
    /// Describes the measurement systems delay.
    /// Refer to [Delay] enum. Refer to [SystemDelay] and [CalibratedDelay] to understand
    /// how to specify the measurement systems delay.
    pub delay: SystemDelay,
    /// Tracks: list of successive measurements
    pub tracks: Vec<Track>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("file i/o error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("only revision 2E is supported")]
    VersionMismatch,
    #[error("invalid version")]
    VersionFormat,
    #[error("invalid CGGTTS format")]
    InvalidFormat,
    #[error("invalid revision date")]
    RevisionDateFormat,
    #[error("non supported revision \"{0}\"")]
    NonSupportedRevision(String),
    #[error("failed to parse \"{0}\" coordinates")]
    CoordinatesParsing(String),
    #[error("failed to identify delay value in line \"{0}\"")]
    DelayIdentificationError(String),
    #[error("failed to parse frequency dependent delay from \"{0}\"")]
    FrequencyDependentDelayParsingError(String),
    #[error("invalid common view class")]
    CommonViewClass,
    #[error("checksum format error")]
    ChecksumFormat,
    #[error("failed to parse checksum value")]
    ChecksumParsing,
    #[error("header crc error")]
    ChecksumError(#[from] crc::Error),
    #[error("missing crc field")]
    CrcMissing,
    #[error("track parsing error")]
    TrackParsing(#[from] track::Error),
}

impl Default for CGGTTS {
    fn default() -> Self {
        Self {
            version: Version::default(),
            release_date: Epoch::from_gregorian_utc_at_midnight(2014, 02, 20), /* latest rev. */
            station: String::from("LAB"),
            nb_channels: 0,
            apc_coordinates: Coordinates::default(),
            receiver: None,
            tracks: Vec::new(),
            ims_hardware: None,
            reference_frame: None,
            reference_time: ReferenceTime::default(),
            comments: None,
            delay: SystemDelay::new(),
        }
    }
}

impl CGGTTS {
    /// Returns [CGGTTS] with desired station name
    pub fn with_station(&self, station: &str) -> Self {
        let mut c = self.clone();
        c.station = station.to_string();
        c
    }

    /// Adds one readable comment to [CGGTTS].
    pub fn with_comment(&self, comment: &str) -> Self {
        let mut s = self.clone();
        s.comments = Some(comment.to_string());
        s
    }

    /// Returns a new [CGGTTS] with desired number of channels.
    pub fn with_channels(&self, ch: u16) -> Self {
        let mut c = self.clone();
        c.nb_channels = ch;
        c
    }

    /// Returns a new [CGGTTS] with [Hardware] information about
    /// the GNSS receiver.
    pub fn with_receiver_hardware(&self, receiver: Hardware) -> Self {
        let mut c = self.clone();
        c.receiver = Some(receiver);
        c
    }

    /// Returns a new [CGGTTS] with [Hardware] information about
    /// the device that help estimate the Ionosphere parameters.
    pub fn with_ims_hardware(&self, ims: Hardware) -> Self {
        let mut c = self.clone();
        c.ims_hardware = Some(ims);
        c
    }

    /// Returns new [CGGTTS] with desired APC coordinates.
    pub fn with_apc_coordinates(&self, apc: Coordinates) -> Self {
        let mut c = self.clone();
        c.apc_coordinates = apc;
        c
    }

    /// Returns new [CGGTTS] with desired reference time system description
    pub fn with_reference_time(&self, reference: ReferenceTime) -> Self {
        let mut c = self.clone();
        c.reference_time = reference;
        c
    }

    /// Returns new [CGGTTS] with desired Reference Frame
    pub fn with_reference_frame(&self, reference: &str) -> Self {
        let mut c = self.clone();
        c.reference_frame = Some(reference.to_string());
        c
    }

    /// Returns true if all tracks (measurements) contained in this
    /// [CGGTTS] follow BIPM tracking specifications.
    pub fn follows_bipm_specs(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.follows_bipm_specs() {
                return false;
            }
        }
        true
    }

    /// Returns true if all tracks (measurements) contained in this
    /// [CGGTTS] have ionospheric parameters estimate.
    pub fn has_ionospheric_data(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.has_ionospheric_data() {
                return false;
            }
        }
        true
    }

    /// Returns [CommonViewClass] used in this file.
    /// ## Returns
    /// - [CommonViewClass::MultiChannel] if at least one track (measurement)
    /// is [CommonViewClass::MultiChannel] measurement
    /// - [CommonViewClass::SingleChannel] if all tracks (measurements)
    /// are [CommonViewClass::SingleChannel] measurements
    pub fn common_view_class(&self) -> CommonViewClass {
        for trk in self.tracks.iter() {
            if trk.class != CommonViewClass::SingleChannel {
                return CommonViewClass::MultiChannel;
            }
        }
        CommonViewClass::SingleChannel
    }

    /// Returns true if this [CGGTTS] is a single channel [CGGTTS],
    /// meaning, all tracks (measurements) are [CommonViewClass::SingleChannel] measurements
    pub fn single_channel(&self) -> bool {
        self.common_view_class() == CommonViewClass::SingleChannel
    }

    /// Returns true if this [CGGTTS] is a single channel [CGGTTS],
    /// meaning, all tracks (measurements) are [CommonViewClass::MultiChannel] measurements
    pub fn multi_channel(&self) -> bool {
        self.common_view_class() == CommonViewClass::MultiChannel
    }

    /// Returns true if this [Constellation] contributed to at least one track (measurement)
    /// contained in this [CGGTTS].
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

    /// Returns true if this [CGGTTS] only contains measurements
    /// against a unique [Constellation] (also referred to, as "mono constellation" [CGGTTS]).
    pub fn has_single_constellation(&self) -> bool {
        self.tracks
            .iter()
            .map(|trk| trk.sv.constellation)
            .unique()
            .count()
            == 1
    }

    /// Returns true if this [CGGTTS] contains measurements
    /// against several [Constellation]s.
    pub fn has_mixed_constellations(&self) -> bool {
        self.tracks
            .iter()
            .map(|trk| trk.sv.constellation)
            .unique()
            .count()
            == 1
    }

    /// Returns [Track] (measurements) Iterator
    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

    /// Iterate over [Track]s (measurements) that result from tracking
    /// this particular [SV] only.
    pub fn sv_tracks(&self, sv: SV) -> impl Iterator<Item = &Track> {
        self.tracks
            .iter()
            .filter_map(move |trk| if trk.sv == sv { Some(trk) } else { None })
    }

    /// Iterate over [Track]s (measurements) that result from tracking
    /// this particular [Constellation] only.
    pub fn constellation_tracks(
        &self,
        constellation: Constellation,
    ) -> impl Iterator<Item = &Track> {
        self.tracks.iter().filter_map(move |trk| {
            if trk.sv.constellation == constellation {
                Some(trk)
            } else {
                None
            }
        })
    }

    /// Returns first Epoch contained in this file.
    pub fn first_epoch(&self) -> Option<Epoch> {
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

    /// Returns a filename that would match naming conventions
    /// to name Self correctly.
    /// Note that Self needs to contain at least one track for this to
    /// generate a competely valid name.
    pub fn filename(&self) -> String {
        let mut res = String::new();

        let constellation = match self.tracks.first() {
            Some(track) => track.sv.constellation,
            None => Constellation::default(),
        };
        res.push_str(&format!("{:x}", constellation));

        if self.has_ionospheric_data() {
            res.push('Z') // Dual Freq / Multi channel
        } else if self.single_channel() {
            res.push('S') // Single Freq / Channel
        } else {
            res.push('M') // Single Freq / Multi Channel
        }

        let max_offset = std::cmp::min(self.station.len(), 4);
        res.push_str(&self.station[0..max_offset]);

        if let Some(epoch) = self.first_epoch() {
            let mjd = epoch.to_mjd_utc_days();
            res.push_str(&format!("{:.3}", (mjd / 1000.0)));
        } else {
            res.push_str("YY.YYY");
        }

        res
    }

    /// Parse [CGGTTS] from a local file.
    /// ```
    /// use cggtts::prelude::CGGTTS;
    /// let cggtts = CGGTTS::from_file("../data/single/GZSY8259.506")
    ///     .unwrap();
    ///
    /// assert_eq!(cggtts.station, "SY82");
    /// assert_eq!(cggtts.follows_bipm_specs(), true);
    ///
    /// if let Some(track) = cggtts.tracks.first() {
    ///     let duration = track.duration;
    ///     let (refsys, srsys) = (track.data.refsys, track.data.srsys);
    ///     assert_eq!(track.has_ionospheric_data(), false);
    ///     assert_eq!(track.follows_bipm_specs(), true);
    /// }
    /// ```
    ///
    /// Advanced CGGTTS files generated from modern GNSS
    /// receivers that may describe the ionospheric delay compensation:
    /// ```
    /// use cggtts::prelude::CGGTTS;
    /// let cggtts = CGGTTS::from_file("../data/dual/RZSY8257.000")
    ///     .unwrap();
    /// if let Some(track) = cggtts.tracks.first() {
    ///     assert_eq!(track.has_ionospheric_data(), true);
    ///     if let Some(iono) = track.iono {
    ///         let (msio, smsi, isg) = (iono.msio, iono.smsi, iono.isg);
    ///     }
    /// }
    ///```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let fd = File::open(path)?;
        let mut reader = BufReader::new(fd);
        Self::parse(&mut reader)
    }

    /// Parse [CGGTTS] from gzip compressed local path.
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn from_gzip_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let fd = File::open(path)?;
        let reader = GzDecoder::new(fd);

        let mut reader = BufReader::new(reader);
        Self::parse(&mut reader)
    }

    /// Parse [CGGTTS] from any [Read]able input.
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, Error> {
        let mut lines_iter = reader.lines();

        // init variables
        let mut system_delay = SystemDelay::new();

        let mut header_ck;
        let mut cksum = 0_u8;

        let (mut blank, mut field_labels, mut unit_labels) = (false, false, false);

        let mut release_date = Epoch::default();
        let mut nb_channels: u16 = 0;

        let mut receiver: Option<Hardware> = None;
        let mut ims_hardware: Option<Hardware> = None;

        let mut station = String::from("LAB");
        let mut comments: Option<String> = None;
        let mut reference_frame: Option<String> = None;
        let mut apc_coordinates = Coordinates::default();
        let mut reference_time = ReferenceTime::default();
        let (_x, _y, _z): (f64, f64, f64) = (0.0, 0.0, 0.0);

        // tracks / measurements parsing
        let mut tracks_parsing = false;
        let mut tracks = Vec::with_capacity(8);

        // VERSION must come first
        let version = lines_iter.next().ok_or(Error::VersionFormat)?;
        let version = version.map_err(|_| Error::VersionFormat)?;

        let version = match scan_fmt!(&version, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => Version::from_str(&version)?,
            _ => return Err(Error::VersionFormat),
        };

        for line in lines_iter {
            if line.is_err() {
                continue;
            }

            let line = line.unwrap();

            if line.starts_with("REV DATE = ") {
                match scan_fmt!(&line, "REV DATE = {d}-{d}-{d}", i32, u8, u8) {
                    (Some(y), Some(m), Some(d)) => {
                        release_date = Epoch::from_gregorian_utc_at_midnight(y, m, d);
                    },
                    _ => {
                        return Err(Error::RevisionDateFormat);
                    },
                }
            } else if line.starts_with("RCVR = ") {
                match scan_fmt!(
                    &line,
                    "RCVR = {} {} {} {d} {}",
                    String,
                    String,
                    String,
                    u16,
                    String
                ) {
                    (
                        Some(manufacturer),
                        Some(recv_type),
                        Some(serial_number),
                        Some(year),
                        Some(release),
                    ) => {
                        receiver = Some(
                            Hardware::default()
                                .with_manufacturer(&manufacturer)
                                .with_model(&recv_type)
                                .with_serial_number(&serial_number)
                                .with_release_year(year)
                                .with_release_version(&release),
                        );
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
                    u16,
                    String
                ) {
                    (
                        Some(manufacturer),
                        Some(recv_type),
                        Some(serial_number),
                        Some(year),
                        Some(release),
                    ) => {
                        ims_hardware = Some(
                            Hardware::default()
                                .with_manufacturer(&manufacturer)
                                .with_model(&recv_type)
                                .with_serial_number(&serial_number)
                                .with_release_year(year)
                                .with_release_version(&release),
                        );
                    },
                    _ => {},
                }
            } else if line.starts_with("LAB = ") {
                match line.strip_prefix("LAB = ") {
                    Some(s) => {
                        station = s.trim().to_string();
                    },
                    _ => {},
                }
            } else if line.starts_with("X = ") {
                match scan_fmt!(&line, "X = {f}", f64) {
                    Some(f) => {
                        apc_coordinates.x = f;
                    },
                    _ => {},
                }
            } else if line.starts_with("Y = ") {
                match scan_fmt!(&line, "Y = {f}", f64) {
                    Some(f) => {
                        apc_coordinates.y = f;
                    },
                    _ => {},
                }
            } else if line.starts_with("Z = ") {
                match scan_fmt!(&line, "Z = {f}", f64) {
                    Some(f) => {
                        apc_coordinates.z = f;
                    },
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
                    comments = Some(c.to_string());
                }
            } else if line.starts_with("REF = ") {
                if let Some(s) = scan_fmt!(&line, "REF = {}", String) {
                    reference_time = ReferenceTime::from_str(&s)
                }
            } else if line.contains("DLY = ") {
                let items: Vec<&str> = line.split_ascii_whitespace().collect();

                let dual_carrier = line.contains(',');

                if items.len() < 4 {
                    continue; // format mismatch
                }

                match items[0] {
                    "CAB" => system_delay.rf_cable_delay = f64::from_str(items[3])?,
                    "REF" => system_delay.ref_delay = f64::from_str(items[3])?,
                    "SYS" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind('=').unwrap();
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
                                let code = items[9].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                        } else {
                            let value = f64::from_str(items[3]).unwrap();
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay.delays.push((code, Delay::System(value)));
                            }
                        }
                    },
                    "INT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind('=').unwrap();
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
                                let code = items[10].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::Internal(value)));
                                }
                            }
                        } else if let Ok(value) = f64::from_str(items[3]) {
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay.delays.push((code, Delay::Internal(value)));
                            }
                        }
                    },
                    "TOT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind('=').unwrap();
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
                                let code = items[9].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay.delays.push((code, Delay::System(value)));
                                }
                            }
                        } else if let Ok(value) = f64::from_str(items[3]) {
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay.delays.push((code, Delay::System(value)));
                            }
                        }
                    },
                    _ => {}, // non recognized delay type
                };
            } else if line.starts_with("CKSUM = ") {
                // CKSUM terminates this section

                // verify CK value
                header_ck = match scan_fmt!(&line, "CKSUM = {x}", String) {
                    Some(s) => match u8::from_str_radix(&s, 16) {
                        Ok(hex) => hex,
                        _ => return Err(Error::ChecksumParsing),
                    },
                    _ => return Err(Error::ChecksumFormat),
                };

                let end_pos = line.find("= ").unwrap();
                cksum = cksum.wrapping_add(crc::calc_crc(line.split_at(end_pos + 2).0)?);

                //if cksum != header_ck {
                //    //return Err(Error::ChecksumError(crc::Error::ChecksumError(cksum, ck)));
                //}

                blank = true;
            } else if blank {
                // Field labels expected next
                blank = false;
                field_labels = true;
            } else if field_labels {
                // Unit labels expected next
                field_labels = false;
                unit_labels = true;
            } else if unit_labels {
                tracks_parsing = true;
                unit_labels = false;
            } else if tracks_parsing {
                if let Ok(trk) = Track::from_str(&line) {
                    tracks.push(trk);
                }
            } else {
                // every single line contributes to Header CRC calculation
                if !line.starts_with("COMMENTS = ") {
                    cksum = cksum.wrapping_add(crc::calc_crc(&line)?);
                }
            }
        }

        Ok(CGGTTS {
            version,
            release_date,
            nb_channels,
            receiver,
            ims_hardware,
            station,
            reference_frame,
            apc_coordinates,
            comments,
            delay: system_delay,
            reference_time,
            tracks,
        })
    }
}

mod crc;
mod cv;
mod hardware;
mod reference_time;
mod version;

#[cfg(test)]
mod tests;

pub mod delay;
pub mod track;

/// CGGTTS production is currently permitted by "Displaying"
/// the [CGGTTS] structure. To produce advanced CGGTTS
/// data correctly, one should specify:
/// - a secondary hardware elemenct [IMS]
/// - ionospheric parameters
/// - carrier dependent delays (see [Delay])
/// ```
/// use gnss_rs as gnss;
/// use cggtts::prelude::*;
/// use cggtts::Coordinates;
/// use cggtts::track::Track;
/// use gnss::prelude::{Constellation, SV};
/// use std::io::Write;
/// fn main() {
///     let rcvr = Hardware::default()
///         .with_manufacturer("SEPTENTRIO")  
///         .with_model("POLARRx5")
///         .with_serial_number("#12345")
///         .with_release_year(2023)
///         .with_release_version("v1");
///
///     let mut cggtts = CGGTTS::default()
///         .with_station("AJACFR")
///         .with_receiver_hardware(rcvr)
///         .with_apc_coordinates(Coordinates {
///             x: 0.0_f64,
///             y: 0.0_f64,
///             z: 0.0_f64,
///         })
///         .with_reference_time(ReferenceTime::UTCk("LAB".to_string()))
///         .with_reference_frame("ITRF");
///         
///     // add some tracks
///
///     // TrackData is mandatory
///     let data = TrackData {
///         refsv: 0.0_f64,
///         srsv: 0.0_f64,
///         refsys: 0.0_f64,
///         srsys: 0.0_f64,
///         dsg: 0.0_f64,
///         ioe: 0_u16,
///         smdt: 0.0_f64,
///         mdtr: 0.0_f64,
///         mdio: 0.0_f64,
///         smdi: 0.0_f64,
///     };
///
///     // tracking parameters
///     let epoch = Epoch::default();
///     let sv = SV::default();
///     let (elevation, azimuth) = (0.0_f64, 0.0_f64);
///     let duration = Duration::from_seconds(780.0);
///
///     // receiver channel being used
///     let rcvr_channel = 0_u8;
///
///     // option 1: track resulting from a single SV observation
///     let track = Track::new(
///         sv,
///         epoch,
///         duration,
///         CommonViewClass::SingleChannel,
///         elevation,
///         azimuth,
///         data,
///         None,
///         rcvr_channel,
///         "L1C",
///     );

///     cggtts.tracks.push(track);
///     let mut fd = std::fs::File::create("test.txt") // does not respect naming conventions
///         .unwrap();
///     write!(fd, "{}", cggtts).unwrap();
/// }
/// ```
impl std::fmt::Display for CGGTTS {
    /// Writes self into a `CGGTTS` file
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        /*
         * Labels in case we provide Ionospheric parameters estimates
         */
        const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
            "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK\n";
        /*
         * Labels in case Ionospheric compensation is not available
         */
        const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
            "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK\n";

        let mut content = String::new();

        content.push_str(&format!(
            "CGGTTS GENERIC DATA FORMAT VERSION = {}\n",
            CURRENT_RELEASE
        ));

        // TODO improve this if it ever changes
        content.push_str("REV DATE = 2014-02-20\n");

        if let Some(rcvr) = &self.receiver {
            content.push_str(&format!("RCVR = {:X}\n", rcvr));
        } else {
            content.push_str("RCVR = RRRRRRRR\n");
        }

        content.push_str(&format!("CH = {}\n", self.nb_channels));

        if let Some(ims) = &self.ims_hardware {
            content.push_str(&format!("IMS = {:X}\n", ims));
        } else {
            content.push_str("IMS = 99999\n");
        }

        content.push_str(&format!("LAB = {}\n", self.station));
        content.push_str(&format!("X = {}\n", self.apc_coordinates.x));
        content.push_str(&format!("Y = {}\n", self.apc_coordinates.y));
        content.push_str(&format!("Z = {}\n", self.apc_coordinates.z));

        if let Some(r) = &self.reference_frame {
            content.push_str(&format!("FRAME = {}\n", r));
        } else {
            content.push_str("FRAME = ITRF\n");
        }

        if let Some(comments) = &self.comments {
            content.push_str(&format!("COMMENTS = {}\n", comments.trim()));
        } else {
            content.push_str("COMMENTS = NO COMMENTS\n");
        }

        let delays = self.delay.delays.clone();
        let constellation = if !self.tracks.is_empty() {
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
                        "INT DLY = {:.1} ns ({:X} {})\n",
                        v, constellation, code
                    ));
                },
                Delay::System(v) => {
                    content.push_str(&format!(
                        "SYS DLY = {:.1} ns ({:X} {})\n",
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
                        "INT DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})\n",
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
                        "SYS DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})\n",
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
        content.push_str(&format!("REF = {}\n", self.reference_time));

        let crc = crc::calc_crc(&content).map_err(|_| std::fmt::Error)?;

        content.push_str(&format!("CKSUM = {:2X}\n\n", crc)); // CKSUM + BLANK

        if self.has_ionospheric_data() {
            content.push_str(TRACK_LABELS_WITH_IONOSPHERIC_DATA);
            content.push_str("             hhmmss  s  .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns\n");
        } else {
            content.push_str(TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA);
            content.push_str("             hhmmss  s  .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s\n");
        }

        write!(fmt, "{}", content)?;

        for track in self.tracks.iter() {
            writeln!(fmt, "{}", track)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_code() {
        assert_eq!(Code::default(), Code::C1);
        assert_eq!(Code::from_str("C2").unwrap(), Code::C2);
        assert_eq!(Code::from_str("P1").unwrap(), Code::P1);
        assert_eq!(Code::from_str("P2").unwrap(), Code::P2);
        assert_eq!(Code::from_str("E5").unwrap(), Code::E5);
    }
}
