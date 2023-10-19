use cggtts::{
    ionospheric::IonosphericData, track::CommonViewClass, track::GlonassChannel, track::Track,
};
use rinex::{constellation::Constellation, sv::Sv};
use std::str::FromStr;

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
"G99 99 59568 001000 0780 099 0099 +9999999999 +99999       +1536   +181   26 999 9999 +999 9999 +999 00 00 L1C D3";
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(
            track.space_vehicule,
            Sv {
                constellation: Constellation::GPS,
                prn: 99
            }
        );
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
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(
            track.space_vehicule,
            Sv {
                constellation: Constellation::GPS,
                prn: 99
            }
        );
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
        assert_eq!(
            track.space_vehicule,
            Sv {
                constellation: Constellation::GPS,
                prn: 99
            }
        );
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
    #[test]
    fn parser_ionospheric() {
        let content =
"R24 FF 57000 000600 0780 347 0394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P EF";
        let track = Track::from_str(content);
        assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.class, CommonViewClass::Multiple);
        assert_eq!(track.follows_bipm_specs(), true);
        assert_eq!(track.duration, std::time::Duration::from_secs(780));
        assert_eq!(track.has_ionospheric_data(), true);
        let iono = track.ionospheric.unwrap();
        assert_eq!(iono.msio, 23.0E-10);
        assert_eq!(iono.smsi, -1.0E-13);
        assert_eq!(iono.isg, 29.0E-10);
        assert_eq!(track.elevation, 34.7);
        assert!((track.azimuth - 39.4).abs() < 1E-6);
        assert_eq!(track.fr, GlonassChannel::Channel(2));
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L3P");
    }
    #[test]
    fn test_ionospheric_data() {
        let data: IonosphericData = (1E-9, 1E-13, 1E-10).into();
        assert_eq!(data.msio, 1E-9);
        assert_eq!(data.smsi, 1E-13);
        assert_eq!(data.isg, 1E-10);
        let (msio, smsi, isg): (f64, f64, f64) = data.into();
        assert_eq!(msio, data.msio);
        assert_eq!(smsi, data.smsi);
        assert_eq!(isg, data.isg);
    }
}
