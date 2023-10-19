use thiserror::Error;
use crate::ionospheric;
use format_num::NumberFormat;
use crate::crc::{calc_crc, CrcError};

mod glonass;
use glonass::GlonassChannel;

mod iono;
use iono::IonosphericData;

mod scheduler;
pub use scheduler::TrackScheduler;

use hifitime::{Duration, Epoch};
use gnss::prelude::{Constellation, SV};

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
    pub fr: Option<GlonassChannel>,
    /// Optionnal receiver channel [0:99], 0 if Unknown
    pub hc: Option<u8>,
    /// Carrier frequency standard 3 letter code,
    /// refer to RINEX specifications for meaning
    pub frc: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("track data format mismatch")]
    InvalidFormat(String),
    #[error("failed to parse sv")]
    SvError(#[from] rinex::Error),
    #[error("crc calc() failed over non utf8 data: \"{0}\"")]
    NonAsciiData(#[from] CrcError),
    #[error("checksum error - expecting \"{0}\" - got \"{1}\"")]
    ChecksumError(u8, u8),
    #[error("failed to parse azimuth angle")]
    AzimuthParsing,
    #[error("missing azimuth field")]
    AzimuthMissing,
    #[error("failed to parse elevation angle")]
    ElevationParsing,
    #[error("missing elevation field")]
    ElevationMissing,
    #[error("missing REFSV field")]
    REFSvMissing,
    #[error("failed to parse REFSV")]
    REFSVParsing,
    #[error("missing SRSV field")]
    SRSvMissing,
    #[error("failed to parse SRSV")]
    SRSVParsing,
    #[error("missing REFSYS field")]
    REFSysMissing,
    #[error("failed to parse REFSYS")]
    REFSYSParsing,
    #[error("missing SRSYS field")]
    SRSysMissing,
    #[error("failed to parse SRSYS")]
    SRSYSParsing,
    #[error("missing DSG field")]
    DSGMissing,
    #[error("failed to parse DSG")]
    DSGParsing,
    #[error("failed to parse IOE")]
    IOEParsing,
    #[error("missing IOE field")]
    IOEMissing,
    #[error("failed to parse IOE")]
    IOEParsing,
    #[error("missing MDTR field")]
    MDTRMissing,
    #[error("failed to parse MDTR")]
    MDTRParsing,
    #[error("missing SMDT field")]
    SMDTMissing,
    #[error("failed to parse SMDT")]
    SMDTParsing,
    #[error("missing MDIO field")]
    MDIOMissing,
    #[error("failed to parse MDIO")]
    MDIOParsing,
    #[error("missing SMDI field")]
    SMDIMissing,
    #[error("failed to parse SMDI")]
    SMDIParsing,
    #[error("missing HC field")]
    HcMissing,
    #[error("failed to parse HC")]
    HcParsing,
    #[error("missing FR field")]
    FRMissing,
    #[error("failed to parse FR")]
    FRParsing,
}

/// Track (clock) data
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct TrackData {
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
        rcvr_channel: Option<u8>,
        frc: Carrier,
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
            fr: None,
            hc: recvr_channel,
            frc,
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
        recvr_channel: Option<u8>,
        glo_channel: u8,
        frc: Carrier,
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
            recvr_channel,
            glo_channel,
            frc,
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
        rcvr_channel: Option<u8>,
        frc: Carrier,
    ) -> Self {
        Self {
            epoch,
            class,
            duration,
            elevation,
            azimuth,
            data,
            iono,
            fr: glo_channel,
            hc: rcvr_channel,
            frc
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
        glo_channel: u16,
        rcvr_channel: Option<u8>,
        frc: Carrier,
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
            frc,
        }
    }

    /// Returns a new `Track` with given Ionospheric parameters,
    /// if parameters were previously assigned, they get overwritten)
    pub fn with_ionospheric_data(&self, data: IonosphericData) -> Self {
        let mut t = self.clone();
        t.class = CommonViewClass::MultiChannel; // always when Iono provided
        t.ionospheric = Some(data);
        t
    }

    /// Returns a `Track` with desired duration
    pub fn with_duration(&self, duration: std::time::Duration) -> Self {
        let mut t = self.clone();
        t.duration = duration;
        t
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
        self.duration.as_secs() == scheduler::BIPM_RECOMMENDED_TRACKING.as_secs()
    }

    /// Returns a `Track` with desired unique space vehicule
    pub fn with_sv(&self, sv: Sv) -> Self {
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
        self.ionospheric.is_some()
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = String::new();
        let num = NumberFormat::new();
        string.push_str(&format!(
            "{} {} {} {} ",
            self.sv,
            self.class,
            julianday::ModifiedJulianDay::from(self.date).inner(),
            self.trktime.format("%H%M%S")
        ));
        string.push_str(&format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} ",
            num.format("04d", self.data.duration.as_secs() as f64),
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

fn parse_data(items: std::str::Lines<'_>) -> Result<TrackData, Error> {
    let refsv = items.next()
        .ok_or(Error::REFSvMissing)?
        .parse::<f64>()
        .map_err(|_| Error::REFSVParsing)? * 0.1E-9;
    
    let srsv = items.next()
        .ok_or(Error::SRSvMissing)?
        .parse::<f64>()
        .map_err(|_| Error::SRSVParsing)? * 0.1E-12;

    let refsys = items.next()
        .ok_or(Error::REFSysMissing)?
        .parse::<f64>()
        .map_err(|_| Error::REFSYSParsing)? * 0.1E-9;

    let srsys = items.next()
        .ok_or(Error::SRSysMissing)?
        .parse::<f64>()
        .map_err(|_| Error::SRSYSParsing)? * 0.1E-12;

    let dsg = items.next()
        .ok_or(Error::DSGMissing)?
        .parse::<f64>()
        .map_err(|_| Error::DSGParsing)? * 0.1E-9;

    let ioe = items.next()
        .ok_or(Error::IOEMissing)?
        .parse::<u16>()
        .map_err(|_| Error::IOEParsing)?;

    let mdtr = items.next()
        .ok_or(Error::MDTRMissing)?
        .parse::<f64>()
        .map_err(|_| Error::MDTRParsing)? * 0.1E-9;
    
    let smdt = items.next()
        .ok_or(Error::SMDTMissing)?
        .parse::<f64>()
        .map_err(|_| Error::SMDTParsing)? * 0.1E-12;

    let mdio = items.next()
        .ok_or(Error::MDIOMissing)?
        .parse::<f64>()
        .map_err(|_| Error::MDIOParsing)? * 0.1E-9;
    
    let smdi = items.next()
        .ok_or(Error::SMDIMissing)?
        .parse::<f64>()
        .map_err(|_| Error::SMDIParsing)? * 0.1E-12;

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

fn parse_without_iono(lines: std::str::Lines<'_>) -> Result<(TrackData, Option<IonosphericData>), Error> {
    let data = parse_data(lines)?; 
    (data, None)
}

fn parse_with_iono(lines: std::str::Lines<'_>) -> Result<(TrackData, Option<IonosphericData>), Error> {
    let data = parse_data(lines)?; 
    
    let msio = items.next()
        .ok_or(Error::MSIOMissing)?
        .parse::<f64>()
        .map_err(|_| Error::MSIOParsing) * 0.1E-9;

    let smsi = items.next()
        .ok_or(Error::SMSIMissing)?
        .parse::<f64>()
        .map_err(|_| Error::SMSIParsing) * 0.1E-12;

    let isg = items.next()
        .ok_or(Error::ISGMissing)?
        .parse::<f64>()
        .map_err(|_| Error::ISGParsing) * 0.1E-9;

    (data, Some(IonosphericData {
        msio,
        smsi,
        isg,
    }))
}

impl std::str::FromStr for Track {
    type Err = Error;
    /* 
     * Builds a Track from given str description
     */
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let cleanedup = String::from(line.trim());
        let items = cleanedup
            .split_ascii_whitespace();
        
        let mut epoch = Epoch::default();

        let sv = SV::from_str(items.next()
            .ok_or(Error::SvMissing)?
            .trim())?;

        let class = CommonViewClass::from_str(
            items.next()
                .ok_or(Error::CvClassMissing)?
                .trim())?;

        let mjd = items.next()
            .ok_or(Error::MjdMissing)?
            .parse::<i32>()
            .map_err(|_| Error::MjdParsing)?;
        
        let trk_sttime = items.next()
            .ok_or(Error::TrkStartTimeMissing)?;

        if trk_sttime.len() < 6 {
            return Err(Error(TrkStartTimeFormat));
        }

        let y = trk_sttime[0..2]
            .parse::<u8>()?;
        let m = trk_sttime[2..4]
            .parse::<u8>()?;
        let d = trk_sttime[4..6]
            .parse::<u8>()?;

        let duration = Duration::from_seconds(
            items.next()
                .ok_or(Error::TrkDurationMissing)?
                .parse::<f64>()
                .map_err(|_| Error::TrkDurationParsing)?);

        let elevation = items.next()
            .ok_or(Error::ElevationMissing)?
            .parse::<f64>()
            .map_err(|_| Error::ElevationParsing)? * 0.1;
        
        let azimuth = items.next()
            .ok_or(Error::AzimuthMissing)?
            .parse::<f64>()
            .map_err(|_| Error::AzimuthParsing)? * 0.1;

        let azimuth = items.next()
            .ok_or(Error::AzimuthMissing)?
            .parse::<f64>()
            .map_err(|_| Error::AzimuthParsing)? * 0.1;

        //let (data, iono, hc, frc, ck) = match items.count() {
        let (data, iono) = match items.count() {
            TRACK_WITH_IONOSPHERIC => {
                parse_with_iono(&mut lines)?
            },
            TRACK_WITHOUT_IONOSPHERIC => {
                parse_without_iono(&mut lines)?
            },
            _ => {
                return Err(Error::InvalidFormat);
            }
        };

        let fr = GlonassChannel::from_str(items.next()
            .ok_or(Error::GlonassChannelMissing)?)?;

        let hc = items.next()
            .ok_or(Error::HcMissing)?
            .parse::<u8>()
            .map_err(|_| Error::HcParsing)?;

        
        // checksum
        let end_pos = line.rfind(ck).unwrap(); // already matching
        let _cksum = calc_crc(&line.split_at(end_pos - 1).0)?;
        // verification
        /*if cksum != ck {
            println!("GOT {} EXPECT {}", ck, cksum);
            return Err(Error::ChecksumError(cksum, ck))
        }*/

        Ok(Track {
            class: {
                if class.eq("FF") {
                    CommonViewClass::Multiple
                } else {
                    CommonViewClass::Single
                }
            },
            sv,
            epoch,
            trktime,
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
