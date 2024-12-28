#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

// #[cfg(feature = "tracker")]
// #[cfg_attr(docsrs, doc(cfg(feature = "tracker")))]
// pub mod tracker;

extern crate gnss_rs as gnss;

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch, TimeScale};
use itertools::Itertools;
use strum_macros::EnumString;

use std::{fs::File, io::BufReader, path::Path};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::{
    delay::SystemDelay,
    hardware::Hardware,
    reference_time::ReferenceTime,
    track::{CommonViewClass, Track},
    version::Version,
};

mod crc;
mod cv;
mod formatting;
mod hardware;
mod parsing;
mod reference_time;
mod version;

#[cfg(test)]
mod tests;

pub mod delay;
pub mod errors;
pub mod track;

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

    pub use crate::{
        cv::CommonViewPeriod,
        hardware::Hardware,
        reference_time::ReferenceTime,
        track::{CommonViewClass, IonosphericData, Track, TrackData},
        version::Version,
        CGGTTS,
    };

    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::prelude::{Duration, Epoch, TimeScale};

    // #[cfg(feature = "scheduler")]
    // #[cfg_attr(docsrs, doc(cfg(feature = "scheduler")))]
    // pub use tracker::{FitData, SVTracker};
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use errors::ParsingError;

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
    /// Antenna phase center coordinates in meters ECEF (as [Coordinates]).
    pub apc_coordinates: Coordinates,
    /// Short readable comments (if any)
    pub comments: Option<String>,
    /// Measurement [SystemDelay]
    pub delay: SystemDelay,
    /// [Track]s describe the result of track fitting,
    /// in chronological order.
    pub tracks: Vec<Track>,
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
            delay: SystemDelay::default(),
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

    /// Adds readable comments to this [CGGTTS].
    /// Try to keep it short, because it will eventually be
    /// wrapped in a single line.
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

    /// Returns new [CGGTTS] with desired APC coordinates in ECEF.
    pub fn with_apc_coordinates(&self, apc: Coordinates) -> Self {
        let mut c = self.clone();
        c.apc_coordinates = apc;
        c
    }

    /// Returns new [CGGTTS] with [TimeScale::UTC] reference system time.
    pub fn with_utc_reference_time(&self) -> Self {
        let mut c = self.clone();
        c.reference_time = TimeScale::UTC.into();
        c
    }

    /// Returns new [CGGTTS] with [TimeScale::TAI] reference system time.
    pub fn with_tai_reference_time(&self) -> Self {
        let mut c = self.clone();
        c.reference_time = TimeScale::TAI.into();
        c
    }

    /// Returns new [CGGTTS] with desired [ReferenceTime] system
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParsingError> {
        let fd = File::open(path)?;
        let mut reader = BufReader::new(fd);
        Self::parse(&mut reader)
    }

    /// Parse [CGGTTS] from gzip compressed local path.
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn from_gzip_file<P: AsRef<Path>>(path: P) -> Result<Self, ParsingError> {
        let fd = File::open(path)?;
        let reader = GzDecoder::new(fd);

        let mut reader = BufReader::new(reader);
        Self::parse(&mut reader)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_code() {
        assert_eq!(Code::default(), Code::C1);
        assert_eq!(Code::from_str("C2").unwrap(), Code::C2);
        assert_eq!(Code::from_str("P1").unwrap(), Code::P1);
        assert_eq!(Code::from_str("P2").unwrap(), Code::P2);
        assert_eq!(Code::from_str("E5").unwrap(), Code::E5);
    }
}
