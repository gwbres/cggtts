use crate::crc::{calc_crc, Error as CrcError};
use format_num::NumberFormat;
use thiserror::Error;

mod glonass;
use glonass::GlonassChannel;

mod iono;
use iono::IonosphericData;

mod scheduler;
pub use scheduler::TrackScheduler;

mod class;
pub use class::CommonViewClass;

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch, Unit};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const TRACK_WITH_IONOSPHERIC: usize = 24;
const TRACK_WITHOUT_IONOSPHERIC: usize = 21;

/// A `Track` is a `Cggtts` measurement
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    /// Common View Class (Single/Multi channel)
    pub class: CommonViewClass,
    /// Epoch of this track
    pub epoch: Epoch,
    /// Tracking duration
    pub duration: Duration,
    /// SV tracked during this realization
    pub sv: SV,
    /// Elevation at track midpoint, expressed in degrees
    pub elevation: f64,
    /// Azimuth at track midpoint, expressed in degrees
    pub azimuth: f64,
    /// Track data
    pub data: TrackData,
    /// Optionnal Ionospheric compensation terms
    pub iono: Option<IonosphericData>,
    /// Glonass Channel Frequency [1:24], O for other GNSS
    pub fr: GlonassChannel,
    /// Hardware / receiver channel [0:99], 0 if Unknown
    pub hc: u8,
    /// Carrier frequency standard 3 letter code,
    /// refer to RINEX specifications for meaning
    pub frc: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("invalid track format")]
    InvalidFormat,
    #[error("invalid sttime field format")]
    InvalidTrkTimeFormat,
    #[error("unknown common view class")]
    UnknownClass,
    #[error("failed to parse sv")]
    SVParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to parse \"{0}\" field")]
    FieldParsing(String),
    #[error("missing \"{0}\" field")]
    MissingField(String),
    #[error("checksum error")]
    CrcError(#[from] crate::crc::Error),
}

/// Track (clock) data
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrackData {
    /// REFSV field
    pub refsv: f64,
    /// SRSV field
    pub srsv: f64,
    /// REFSYS field
    pub refsys: f64,
    /// SRSYS field
    pub srsys: f64,
    /// Data signma (`DSG`) : RMS residuals to linear fit from solution B in section 2.3.3
    pub dsg: f64,
    /// Issue of Ephemeris (`IOE`),
    /// Three-digit decimal code indicating the ephemeris used for the computation.
    /// As no IOE is associated with the GLONASS navigation messages, the values 1-96 have to be
    /// used to indicate the date of the ephemeris used, given by the number of the quarter of an hour in
    /// the day, starting at 1=00h00m00s.
    /// For BeiDou, IOE will report the integer hour in the date of the ephemeris (Time of Clock).
    pub ioe: u16,
    /// Modeled tropospheric delay corresponding to the solution C in section 2.3.3
    pub mdtr: f64,
    /// Slope of the modeled tropospheric delay corresponding to the solution C in section 2.3.3
    pub smdt: f64,
    /// Modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    pub mdio: f64,
    /// Slope of the modeled ionospheric delay corresponding to the solution D in section 2.3.3.
    pub smdi: f64,
}

impl Track {
    /// Builds new CGGTTS track from single SV realization.
    /// For Glonass vehicles, prefer [Self::new_glonass_sv]
    pub fn new_sv(
        sv: SV,
        epoch: Epoch,
        duration: Duration,
        class: CommonViewClass,
        elevation: f64,
        azimuth: f64,
        data: TrackData,
        iono: Option<IonosphericData>,
        rcvr_channel: u8,
        frc: &str,
    ) -> Self {
        Self {
            epoch,
            class,
            sv,
            duration,
            elevation,
            azimuth,
            data,
            iono,
            fr: GlonassChannel::Unknown,
            hc: rcvr_channel,
            frc: frc.to_string(),
        }
    }
    /// Builds new CGGTTS track from single Glonass SV realization
    pub fn new_glonass_sv(
        sv: SV,
        epoch: Epoch,
        duration: Duration,
        class: CommonViewClass,
        elevation: f64,
        azimuth: f64,
        data: TrackData,
        iono: Option<IonosphericData>,
        rcvr_channel: u8,
        glo_channel: GlonassChannel,
        frc: &str,
    ) -> Self {
        Self {
            sv,
            epoch,
            duration,
            class,
            elevation,
            azimuth,
            data,
            iono,
            fr: glo_channel,
            hc: rcvr_channel,
            frc: frc.to_string(),
        }
    }
    /// Builds new CGGTTS track resulting from a melting pot realization
    pub fn new_melting_pot(
        epoch: Epoch,
        duration: Duration,
        class: CommonViewClass,
        elevation: f64,
        azimuth: f64,
        data: TrackData,
        iono: Option<IonosphericData>,
        rcvr_channel: u8,
        glo_channel: GlonassChannel,
        frc: &str,
    ) -> Self {
        Self {
            sv: SV {
                constellation: Constellation::GPS,
                prn: 99,
            },
            epoch,
            class,
            duration,
            elevation,
            azimuth,
            data,
            iono,
            fr: GlonassChannel::Unknown,
            hc: rcvr_channel,
            frc: frc.to_string(),
        }
    }
    /// Builds a new CGGTTS track from Glonass melting pot realization
    pub fn new_glonass_melting_pot(
        epoch: Epoch,
        duration: Duration,
        class: CommonViewClass,
        elevation: f64,
        azimuth: f64,
        data: TrackData,
        iono: Option<IonosphericData>,
        glo_channel: GlonassChannel,
        rcvr_channel: u8,
        frc: &str,
    ) -> Self {
        Self {
            epoch,
            class,
            duration,
            sv: SV {
                constellation: Constellation::Glonass,
                prn: 99,
            },
            elevation,
            azimuth,
            data,
            iono,
            fr: glo_channel,
            hc: rcvr_channel,
            frc: frc.to_string(),
        }
    }
    /// Track is a melting pot is only one SV was tracked during its realization
    pub fn melting_pot(&self) -> bool {
        self.sv.prn == 99
    }
    /// Returns true if Self was measured against given `GNSS` Constellation
    pub fn uses_constellation(&self, c: Constellation) -> bool {
        self.sv.constellation == c
    }

    /// Returns True if Self follows BIPM specifications / requirements,
    /// in terms of tracking pursuit
    pub fn follows_bipm_specs(&self) -> bool {
        self.duration == Duration::from_seconds(780.0)
    }

    /// Returns a `Track` with desired unique space vehicule
    pub fn with_sv(&self, sv: SV) -> Self {
        let mut t = self.clone();
        t.sv = sv.clone();
        t
    }

    /// Returns a track with desired elevation angle in Degrees
    pub fn with_elevation(&self, elevation: f64) -> Self {
        let mut t = self.clone();
        t.elevation = elevation;
        t
    }

    /// Returns a `Track` with given azimuth angle in Degrees, at tracking midpoint
    pub fn with_azimuth(&self, azimuth: f64) -> Self {
        let mut t = self.clone();
        t.azimuth = azimuth;
        t
    }

    /// Returns a `Track` with desired Frequency carrier code
    pub fn with_carrier_code(&self, code: &str) -> Self {
        let mut t = self.clone();
        t.frc = code.to_string();
        t
    }

    /// Returns true if Self comes with Ionospheric parameter estimates
    pub fn has_ionospheric_data(&self) -> bool {
        self.iono.is_some()
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = String::new();
        let num = NumberFormat::new();
        let mjd = self.epoch.to_mjd_utc_days();
        let (_, _, _, h, m, s, _) = self.epoch.to_gregorian_utc();
        string.push_str(&format!(
            "{} {:X} {} {:02}{:02}:{:02}",
            self.sv,
            self.class,
            mjd.floor() as u32,
            h,
            m,
            s
        ));
        string.push_str(&format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} ",
            num.format("04d", self.duration.to_seconds() as f64),
            num.format("03d", self.elevation * 10.0),
            num.format("04d", self.azimuth * 10.0),
            num.format("+11d", self.data.refsv * 1E10),
            num.format("+4d", self.data.srsv * 1E13),
            num.format("+11d", self.data.refsys * 1E10),
            num.format("+6d", self.data.srsys * 1E13),
            num.format("4d", self.data.dsg * 1E10),
            num.format("03d", self.data.ioe),
            num.format("04d", self.data.mdtr * 1E10),
            num.format("+04d", self.data.smdt * 1E13),
            num.format("04d", self.data.mdio * 1E10),
            num.format("+04d", self.data.smdi * 1E13),
        ));
        if let Some(iono) = self.iono {
            string.push_str(&format!(
                "{} {} {} ",
                num.format("11d", iono.msio * 1E10),
                num.format("+6d", iono.smsi * 1E13),
                num.format("04d", iono.isg * 1E10),
            ));
        }

        string.push_str(&format!("{:02} {:02X} {}", self.fr, self.hc, self.frc));

        if let Ok(crc) = calc_crc(&string) {
            string.push_str(&format!(" {:2X}", crc + 32))
        }
        fmt.write_str(&string)
    }
}

fn parse_data(items: &mut std::str::SplitAsciiWhitespace<'_>) -> Result<TrackData, Error> {
    let refsv = items
        .next()
        .ok_or(Error::MissingField(String::from("REFSV")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("REFSV")))?
        * 0.1E-9;

    let srsv = items
        .next()
        .ok_or(Error::MissingField(String::from("SRSV")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SRSV")))?
        * 0.1E-12;

    let refsys = items
        .next()
        .ok_or(Error::MissingField(String::from("REFSYS")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("REFSYS")))?
        * 0.1E-9;

    let srsys = items
        .next()
        .ok_or(Error::MissingField(String::from("SRSYS")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SRSYS")))?
        * 0.1E-12;

    let dsg = items
        .next()
        .ok_or(Error::MissingField(String::from("DSG")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("DSG")))?
        * 0.1E-9;

    let ioe = items
        .next()
        .ok_or(Error::MissingField(String::from("IOE")))?
        .parse::<u16>()
        .map_err(|_| Error::FieldParsing(String::from("IOE")))?;

    let mdtr = items
        .next()
        .ok_or(Error::MissingField(String::from("MDTR")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("MDTR")))?
        * 0.1E-9;

    let smdt = items
        .next()
        .ok_or(Error::MissingField(String::from("SMDT")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SMDT")))?
        * 0.1E-12;

    let mdio = items
        .next()
        .ok_or(Error::MissingField(String::from("MDIO")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("MDIO")))?
        * 0.1E-9;

    let smdi = items
        .next()
        .ok_or(Error::MissingField(String::from("SMDI")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SMDI")))?
        * 0.1E-12;

    Ok(TrackData {
        refsv,
        srsv,
        refsys,
        srsys,
        dsg,
        ioe,
        mdtr,
        smdt,
        mdio,
        smdi,
    })
}

fn parse_without_iono(
    items: &mut std::str::SplitAsciiWhitespace<'_>,
) -> Result<(TrackData, Option<IonosphericData>), Error> {
    let data = parse_data(items)?;
    Ok((data, None))
}

fn parse_with_iono(
    items: &mut std::str::SplitAsciiWhitespace<'_>,
) -> Result<(TrackData, Option<IonosphericData>), Error> {
    let data = parse_data(items)?;

    let msio = items
        .next()
        .ok_or(Error::MissingField(String::from("MSIO")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("MSIO")))?
        * 0.1E-9;

    let smsi = items
        .next()
        .ok_or(Error::MissingField(String::from("SMSI")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SMSI")))?
        * 0.1E-12;

    let isg = items
        .next()
        .ok_or(Error::MissingField(String::from("ISG")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("ISG")))?
        * 0.1E-9;

    Ok((data, Some(IonosphericData { msio, smsi, isg })))
}

impl std::str::FromStr for Track {
    type Err = Error;
    /*
     * Builds a Track from given str description
     */
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let cleanedup = String::from(line.trim());
        let mut epoch = Epoch::default();
        let mut items = cleanedup.split_ascii_whitespace();

        let sv = SV::from_str(
            items
                .next()
                .ok_or(Error::MissingField(String::from("SV")))?,
        )?;

        let class = CommonViewClass::from_str(
            items
                .next()
                .ok_or(Error::MissingField(String::from("CL")))?
                .trim(),
        )?;

        let mjd = items
            .next()
            .ok_or(Error::MissingField(String::from("MJD")))?
            .parse::<i32>()
            .map_err(|_| Error::FieldParsing(String::from("MJD")))?;

        let trk_sttime = items
            .next()
            .ok_or(Error::MissingField(String::from("STTIME")))?;

        if trk_sttime.len() < 6 {
            return Err(Error::InvalidTrkTimeFormat);
        }

        let h = trk_sttime[0..2]
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("STTIME:%H")))?;

        let m = trk_sttime[2..4]
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("STTIME:%M")))?;

        let s = trk_sttime[4..6]
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("STTIME:%S")))?;

        let mut epoch = Epoch::from_mjd_utc(mjd as f64);
        epoch = epoch + (h as f64) * Unit::Hour;
        epoch = epoch + (m as f64) * Unit::Minute;
        epoch = epoch + (s as f64) * Unit::Second;

        let nb_items = items.clone().count();

        let duration = Duration::from_seconds(
            items
                .next()
                .ok_or(Error::MissingField(String::from("STTIME")))?
                .parse::<f64>()
                .map_err(|_| Error::FieldParsing(String::from("STTIME")))?,
        );

        let elevation = items
            .next()
            .ok_or(Error::MissingField(String::from("ELV")))?
            .parse::<f64>()
            .map_err(|_| Error::FieldParsing(String::from("ELV")))?
            * 0.1;

        let azimuth = items
            .next()
            .ok_or(Error::MissingField(String::from("AZTH")))?
            .parse::<f64>()
            .map_err(|_| Error::FieldParsing(String::from("AZTH")))?
            * 0.1;

        //let (data, iono, hc, frc, ck) = match items.count() {
        let (data, iono) = match nb_items {
            TRACK_WITH_IONOSPHERIC => parse_with_iono(&mut items)?,
            TRACK_WITHOUT_IONOSPHERIC => parse_without_iono(&mut items)?,
            _ => {
                return Err(Error::InvalidFormat);
            },
        };

        let fr = GlonassChannel::from_str(
            items
                .next()
                .ok_or(Error::MissingField(String::from("fr")))?,
        )?;

        let hc = items
            .next()
            .ok_or(Error::MissingField(String::from("hc")))?
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("hc")))?;

        let frc: String = items
            .next()
            .ok_or(Error::MissingField(String::from("frc")))?
            .parse()
            .map_err(|_| Error::FieldParsing(String::from("frc")))?;

        // checksum
        let ck = items
            .next()
            .ok_or(Error::MissingField(String::from("ck")))?
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("ck")))?;

        // let cksum = calc_crc(&line.split_at(end_pos - 1).0)?;

        // verification
        /*if cksum != ck {
            println!("GOT {} EXPECT {}", ck, cksum);
            return Err(Error::ChecksumError(cksum, ck))
        }*/

        Ok(Track {
            sv,
            class,
            epoch,
            duration,
            elevation,
            azimuth,
            data,
            iono,
            fr,
            hc,
            frc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::GlonassChannel;
    #[test]
    fn test_glonass_channel() {
        let c = GlonassChannel::Unknown;
        assert_eq!(c.to_string(), "00");
        let c = GlonassChannel::Channel(1);
        assert_eq!(c.to_string(), "01");
        let c = GlonassChannel::Channel(10);
        assert_eq!(c.to_string(), "0A");
        assert_eq!(c, GlonassChannel::Channel(10));
        assert_eq!(c != GlonassChannel::Unknown, true);
        assert_eq!(GlonassChannel::default(), GlonassChannel::Unknown);
    }
}
