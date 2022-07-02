use std::str::FromStr;
use rinex::{Constellation, Sv};
use cggtts::{Track, track::CommonViewClass, track::GlonassChannel};

#[cfg(test)]
mod track {
    use super::*;
    #[test]
    fn constructor() {
        let track = Track::default();
        assert_eq!(track.duration.as_secs(), 780);
        assert_eq!(track.elevation, 0.0);
        assert_eq!(track.azimuth, 0.0);
        assert_eq!(track.refsv, 0.0);
        assert_eq!(track.srsv, 0.0);
        assert_eq!(track.refsys, 0.0);
        assert_eq!(track.srsys, 0.0);
        assert_eq!(track.ionospheric, None);
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.space_vehicule_combination(), true);
    }

    #[test]
    fn parser() {
        let content = 
"G99 99 59506 043400 0780 099 0099 +9999999999 +99999 +9999989126   +283   25 999 9999 +999 9999 +999 00 00 L1C 6F";
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.space_vehicule, Sv {
            constellation: Constellation::GPS,
            prn: 99
        });
        assert_eq!(track.class, CommonViewClass::Single);
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.duration, std::time::Duration::from_secs(780));
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert!((track.dsg - 2.5E-9).abs() < 1E-6);
        assert!((track.srsys - 2.83E-11).abs() < 1E-6);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");
        let dumped = track.to_string();
        assert_eq!(content.to_owned(), dumped); 
        
        let content =
"G99 99 59563 001400 0780 099 0099 +9999999999 +99999       +1588  +1027   27 999 9999 +999 9999 +999 00 00 L1C EA";
//SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.space_vehicule, Sv {
            constellation: Constellation::GPS,
            prn: 99
        });
        assert_eq!(track.class, CommonViewClass::Single);
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.duration, std::time::Duration::from_secs(780));
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");
        let dumped = track.to_string();
        assert_eq!(content.to_owned(), dumped); 

        let content =
"G99 99 59563 232200 0780 099 0099 +9999999999 +99999       +1529   -507   23 999 9999 +999 9999 +999 00 00 L1C D9";
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.class, CommonViewClass::Single);
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.duration, std::time::Duration::from_secs(780));
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");
        let dumped = track.to_string();
        assert_eq!(content.to_owned(), dumped); 

        let content =
"G99 99 59567 001400 0780 099 0099 +9999999999 +99999       +1561   -151   27 999 9999 +999 9999 +999 00 00 L1C D4";
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.space_vehicule, Sv {
            constellation: Constellation::GPS,
            prn: 99
        });
        assert_eq!(track.class, CommonViewClass::Single);
        //assert_eq!(track.trktime 043400)
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.duration, std::time::Duration::from_secs(780));
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");
        let dumped = track.to_string();
        assert_eq!(content.to_owned(), dumped); 
    }
/*
    #[test]
    fn parser_ionospheric() {
        let content =
"R24 FF 57000 000600 780 347 394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P";
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.space_vehicule, Some(
            Sv {
                constellation: Constellation::Glonass,
                prn: 24,
            }
        ));
        assert_eq!(track.class, CommonViewClass::Combination(Constellation::GPS));
        //assert_eq!(track.trktime 043400)
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.duration, std::time::Duration::from_secs(780));
        assert_eq!(track.has_ionospheric_data(), false);
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");
    }
*/
/*
SAT CL MJD STTIME TRKL ELV AZTH REFSV SRSV REFSYS SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK
             hhmmss s .1dg .1dg .1ns .1ps/s .1ns .1ps/s .1ns .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns
R05 FF 57000 000600 780 70 2325 +22617 +6 165 -3 53 2 646 +606 131 -9 131 -9 37 +1 0 L3P 8C
R17 FF 57000 000600 780 539 1217 -1407831 -36 154 -54 20 2 100 -8 24 +0 24 0 13 +4 0 L3P 7A
R16 FF 57000 000600 780 370 3022 +308130 -18 246 -28 29 2 134 -22 63 +4 63 4 21 1 0 L3P 80*/
}
