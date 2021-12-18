//! CGGTTS Track
//! https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf
//! TODO @ GBR <!> pb sur checksum - on ne peut pas utiliser '112' comme dit dans la doc
//! TODO @ GBR |1-x| prochainement implemente' dans syref pour srsys ??
//////////////////////////////////////////////////////////////////////////////

use thiserror::Error;
use std::str::FromStr;

use rinex;

/*
 * Common view class
 */
#[derive(Clone, Debug, PartialEq)]
enum CommonViewClassType {
    SingleFile,
    MultiFiles,
}

/*
 * Constellation Code denomination
 * see RINEX demoninations 
 */
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
enum ConstellationRinexCode {
    GPS_GLO_QZ_SBA_L1C,
    GPS_GLO_L1P,
    GAL_E1,
    QZSS_L1C,
    BEIDOU_B1i,
    GPS_C1_P1C2_P2,
    GAL_E1E5a,
    BEIDOU_BliB2i,
    GLO_C1_P1C2_P2,
    GZSS_C1C5,
    NonSupportedCode,
}

#[derive(Error, Debug)]
pub enum ConstellationRinexCodeError {
    #[error("unknown rinex constellation code '{0}'")]
    UnknownCode(String),
}

const TRACK_WITH_IONOSPHERIC_DATA_LENGTH: usize = 24;
const TRACK_WITHOUT_IONOSPHERIC_DATA_LENGTH: usize = 21;

/*
 * CGGTTS Track 
 */
#[derive(Debug, Clone)]
pub struct CggttsTrack {
    constellation: rinex::Constellation,
    sat_id: u8,
    class: CommonViewClassType,
    trktime: chrono::NaiveTime, // Tracking start date (hh:mm:ss)
    duration: u32, // Tracking duration [in secs]
    elevation: f64, // Elevation (angle) at Tracking midpoint [in degrees]
    azimuth: f64, // Azimuth (angle) at Tracking midpoint [in degrees]
    refsv: f64,
    srsv: f64,
    refsys: f64,
    srsys: f64,
    // DSG [Data Sigma]
    // Root-mean-square of the residuals to linear fit from solution B in section 2.3.3
    dsg: f64,
    // IOE [Issue of Ephemeris]
    // Three-digit decimal code indicating the ephemeris used for the computation.
    // As no IOE is associated with the GLONASS navigation messages, the values 1-96 have to be
    // used to indicate the date of the ephemeris used, given by the number of the quarter of an hour in
    // the day, starting at 1=00h00m00s. For BeiDou, IOE will report the integer hour in the date of the
    // ephemeris (Time of Clock).
    ioe: u16,
    mdtr: f64, // Modeled tropospheric delay corresponding to the solution C in section 2.3.3
    smdt: f64, // Slope of the modeled tropospheric delay corresponding to the solution C in section 2.3.3
    mdio: f64, // Modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    smdi: f64, // Slope of the modelled ionospheric delay corresponding to the solution D in section 2.3.3.
    msio: Option<f64>, // Measured ionospheric delay corresponding to the solution E in section 2.3.3.
    smsi: Option<f64>, // Slope of the measured ionospheric delay corresponding to the solution E in section 2.3.3.
    isg: Option<f64>, // [Ionospheric Sigma] Root-mean-square of the residuals corresponding to the solution E in section2.3.3
    fr: u8, // Glonass Channel Frequency [1:24], O for other GNSS
    hc: u8, // Receiver Hardware Channel [0:99], 0 if Unknown
    frc: ConstellationRinexCode 
}

/*
 * Error wrapper
 */
#[derive(Error, Debug)]
pub enum Error {
    #[error("nb of white spaces does not match expected CGGTTS format")]
    FormatError,
    #[error("failed to parse int number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse track time")]
    ChronoParseError(#[from] chrono::ParseError),
    #[error("unknown gnss constellation")]
    ConstellationError(#[from] rinex::ConstellationError),
    #[error("unknown constellation rinex code \"{0}\"")]
    ConstellationRinexCodeError(#[from] ConstellationRinexCodeError),
    #[error("failed to parse common view class")]
    CommonViewClassError(#[from] std::str::Utf8Error),
    #[error("checksum error - expecting \"{0}\" - got \"{1}\"")]
    ChecksumError(u8, u8),
}

impl CggttsTrack {
    pub fn new (line: &str) -> Result<CggttsTrack, Error> {
        let cleaned_up = String::from(line.trim());
        let items: Vec<&str> = cleaned_up.split_ascii_whitespace().collect();
        let constellation = rinex::Constellation::from_str(items.get(0).unwrap_or(&""))?; 
        let (_, sat_id) = items.get(0).unwrap_or(&"").split_at(1);
        let class = CommonViewClassType::from_str(items.get(1).unwrap_or(&""))?;
        let trktime = chrono::NaiveTime::parse_from_str(items.get(3).unwrap_or(&""), "%H%M%S")?;
        let duration = u32::from_str_radix(items.get(4).unwrap_or(&""), 10)?;
        let elevation = f64::from_str(items.get(5).unwrap_or(&""))? * 0.1;
        let azimuth = f64::from_str(items.get(6).unwrap_or(&""))? * 0.1;
        let refsv = f64::from_str(items.get(7).unwrap_or(&""))? * 0.1E-9;
        let srsv = f64::from_str(items.get(8).unwrap_or(&""))? * 0.1E-12;
        let refsys = f64::from_str(items.get(9).unwrap_or(&""))? * 0.1E-9;
        let srsys = f64::from_str(items.get(10).unwrap_or(&""))? * 0.1E-12;
        let dsg = f64::from_str(items.get(11).unwrap_or(&""))? * 0.1E-9;
        let ioe = u16::from_str_radix(items.get(12).unwrap_or(&""), 10)?;
        let mdtr = f64::from_str(items.get(13).unwrap_or(&""))? * 0.1E-9;
        let smdt = f64::from_str(items.get(14).unwrap_or(&""))? * 0.1E-12;
        let mdio = f64::from_str(items.get(15).unwrap_or(&""))? * 0.1E-9;
        let smdi = f64::from_str(items.get(16).unwrap_or(&""))? * 0.1E-12;

        // TODO see next release
        //if refsys > 100E-3 { 
        //    refsys = 1.0 - refsys
        //}

        let (msio, smsi, isg, fr, hc, frc, ck): (Option<f64>,Option<f64>,Option<f64>,u8,u8,ConstellationRinexCode,u8) = match items.len() {
            TRACK_WITHOUT_IONOSPHERIC_DATA_LENGTH => {
                (None,None,None,
                u8::from_str_radix(items.get(17).unwrap_or(&""), 16)?, 
                u8::from_str(items.get(18).unwrap_or(&""))?,
                ConstellationRinexCode::from_str(items.get(19).unwrap_or(&""))?,
                u8::from_str_radix(items.get(20).unwrap_or(&""), 16)?)
            },
            TRACK_WITH_IONOSPHERIC_DATA_LENGTH => {
                (Some(f64::from_str(items.get(17).unwrap_or(&""))? * 0.1E-9), 
                Some(f64::from_str(items.get(18).unwrap_or(&""))? * 0.1E-12), 
                Some(f64::from_str(items.get(19).unwrap_or(&""))? * 0.1E-9),
                u8::from_str_radix(items.get(20).unwrap_or(&""), 16)?, 
                u8::from_str_radix(items.get(21).unwrap_or(&""), 16)?,
                ConstellationRinexCode::from_str(items.get(22).unwrap_or(&""))?,
                u8::from_str_radix(items.get(23).unwrap_or(&""), 16)?)
            },
            _ => return Err(Error::FormatError),
        };
        // checksum field
        let bytes = String::from(line.trim()).into_bytes();
        let mut chksum: u8 = 0;
        let last_payload_item = items.get(items.len()-2)
            .unwrap(); // already matching
        let end_pos = line.rfind(last_payload_item)
            .unwrap();
        for i in 0..end_pos+1 { // CK
            chksum = chksum.wrapping_add(bytes[i])
        }
        // checksum verification
        if chksum != ck {
        //    return Err(Error::ChecksumError(chksum, ck))
        }

        Ok(CggttsTrack {
            constellation,
            sat_id: u8::from_str_radix(sat_id, 10).unwrap_or(0),
            class,
            trktime,
            duration,
            elevation,
            azimuth,
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
            msio,
            smsi,
            isg,
            fr,
            hc,
            frc
        })
    }

    /* returns track (refsys, srsys) duplet */
    pub fn get_refsys_srsys (&self) -> (f64, f64) { (self.refsys, self.srsys) }

    /* returns track start time */ 
    pub fn get_track_start_time (&self) -> chrono::NaiveTime { self.trktime }

    /* returns true if track does have ionospheric data */
    pub fn has_ionospheric_data (&self) -> bool { self.msio.is_some() && self.smsi.is_some() && self.isg.is_some() }

    /* returns ionospheric data triplet (msio, smsi, isg) */
    pub fn get_ionospheric_data (&self) -> Option<(f64, f64, f64)> {
        if self.has_ionospheric_data() {
            Some((self.msio.unwrap(),self.smsi.unwrap(),self.isg.unwrap()))
        } else {
            None
        }
    }
}

// custom display Formatter
impl std::fmt::Display for CggttsTrack {
    // custom diplay formatter
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f, "Constellation: {:?} | SAT #{}\nCommon View Class: '{:?}'\nStart Time: {} | Duration: {}\nElevation: {} | Azimuth: {}\nREFSV: {} | SRSV: {} | REFSYS: {} SRSYS: {}\nDSG: {} | IOE: {}\nMDTR: {} | SMDT: {} | MDIO: {} | SMDI: {} | MSIO: {:#?} | SMSI: {:#?} | ISG: {:#?}\nFR: {} | HC: {}",
            self.constellation, self.sat_id, self.class,
            self.trktime, self.duration,
            self.elevation, self.azimuth, self.refsv, self.srsv,
            self.refsys, self.srsys,
            self.dsg, self.ioe,
            self.mdtr, self.smdt, self.mdio, self.smdi, self.msio, self.smsi, self.isg,
            self.fr, self.hc, //self.frc
        )
    }
}

impl std::str::FromStr for CommonViewClassType {
    type Err = std::str::Utf8Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("FF") {
            Ok(CommonViewClassType::MultiFiles)
        } else {
            Ok(CommonViewClassType::SingleFile)
        }
    }
}

impl std::str::FromStr for ConstellationRinexCode {
    type Err = ConstellationRinexCodeError;   
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("L1C") {
            Ok(ConstellationRinexCode::GPS_GLO_QZ_SBA_L1C)
        } else if s.eq("L1P") {
            Ok(ConstellationRinexCode::GPS_GLO_L1P)
        } else if s.eq("E1") {
            Ok(ConstellationRinexCode::GPS_GLO_L1P)
        } else if s.eq("L1C") {
            Ok(ConstellationRinexCode::GAL_E1)
        } else if s.eq("B1i") {
            Ok(ConstellationRinexCode::QZSS_L1C)
        } else if s.eq("L3P") {
            Ok(ConstellationRinexCode::BEIDOU_B1i)
        } else if s.eq("L3E") {
            Ok(ConstellationRinexCode::GPS_C1_P1C2_P2)
        } else if s.eq("L3B") {
            Ok(ConstellationRinexCode::GAL_E1E5a)
        } else if s.eq("L3P") {
            Ok(ConstellationRinexCode::BEIDOU_BliB2i)
        } else if s.eq("L3Q") {
            Ok(ConstellationRinexCode::GZSS_C1C5)
        } else {
            Err(ConstellationRinexCodeError::UnknownCode(s.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
/*
    #[test]
    /// Tests CGGTTS track parser against test data
    fn cggtts_track_parser() -> std::io::Result<()> {   
        let test_resources = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + "/data");
        for entry in std::fs::read_dir(test_resources).unwrap() {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                let name = path.to_str().unwrap_or("");
                let content: String = std::fs::read_to_string(name).unwrap_or(String::from("")).parse().unwrap_or(String::from(""));
                let lines: Vec<&str> = content.split("\n").collect();
                for line in 0..lines.len() {
                    let line_content = lines.get(line).unwrap_or(&"");
                    if line > 18 && line_content.len() > 0 { 
                        match CggttsTrack::new(line_content) {
                            Ok(_) => {},
                            Err(e) => panic!("CggttsTrack::new() failed with \"{}\" - parsing file \"{}\" line #{} \"{}\"", e, name, line+1, line_content.trim())
                        }
                    }
                }
            }
            Ok(())
        }
    }
*/
}
