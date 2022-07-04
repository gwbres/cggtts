use rinex::sv::Sv;
use rinex::constellation::Constellation;

use cggtts::Rcvr;
use cggtts::Cggtts;
use cggtts::track::GlonassChannel;
use cggtts::{CalibratedDelay, Delay, TimeSystem};

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_standard_cggtts() {
        let cggtts = Cggtts::from_file(
            &(env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/standard/GZSY8259.568"));
        assert_eq!(cggtts.is_ok(), true);
        let cggtts = cggtts.unwrap();
        assert_eq!(cggtts.rev_date.format("%Y-%m-%d").to_string(), String::from("2014-02-20"));
        assert_eq!(cggtts.rcvr, Some(Rcvr{
            manufacturer: String::from("GORGYTIMING"),
            recv_type: String::from("SYREF25"),
            serial_number: String::from("18259999"),
            year: 2018,
            release: String::from("v00"),
        }));
        assert_eq!(cggtts.lab, Some(String::from("SY82")));
        assert_eq!(cggtts.nb_channels, 12);
        assert_eq!(cggtts.ims, None);
        assert_eq!(cggtts.time_reference, 
            TimeSystem::Unknown(String::from("REF(SY82)")));
        assert_eq!(cggtts.reference_frame, Some(String::from("ITRF")));
        assert!((cggtts.coordinates.x - 4314143.824).abs() < 1E-6);
        assert!((cggtts.coordinates.y - 452633.241).abs() < 1E-6);
        assert!((cggtts.coordinates.z - 4660711.385).abs() < 1E-6);
        assert_eq!(cggtts.comments, None);
        assert_eq!(cggtts.tracks.len(), 32);
        let first = cggtts.tracks.first();
        assert_eq!(cggtts.delay.value(), 0.0);
        assert_eq!(first.is_some(), true);
        let first = first.unwrap();
        assert_eq!(first.space_vehicule, Sv {
            constellation: Constellation::GPS,
            prn: 99,
        });

        let _dumped = cggtts.to_string();
        let _compare = std::fs::read_to_string(
            &(env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/standard/GZSY8259.568")).unwrap();
    }
    #[test]
    fn parse_standard_data() {
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/standard");
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let cggtts = Cggtts::from_file(&path.to_str().unwrap());
                assert_eq!(
                    cggtts.is_err(),
                    false,
                    "Cggtts::from_file() failed for {:#?} with {:#?}",
                    path,
                    cggtts);
                let cggtts = cggtts.unwrap();
                assert_eq!(cggtts.has_ionospheric_data(), false);
            }
        }
    }
    #[test]
    fn parse_advanced_data() {
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/advanced");
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let cggtts = Cggtts::from_file(&path.to_str().unwrap());
                assert_eq!(
                    cggtts.is_err(),
                    false,
                    "Cggtts::from_file() failed for {:#?} with {:#?}",
                    path,
                    cggtts);
                let cggtts = cggtts.unwrap();
                assert_eq!(cggtts.has_ionospheric_data(), true);
            }
        }
    }
    #[test]
    fn test_advanced_cggtts() {
        let cggtts = Cggtts::from_file(
            &(env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/advanced/RZSY8257.000"));
        assert_eq!(cggtts.is_ok(), true);
        let cggtts = cggtts.unwrap();
        assert_eq!(cggtts.rev_date.format("%Y-%m-%d").to_string(), String::from("2014-02-20"));
        assert_eq!(cggtts.rcvr, None);
        assert_eq!(cggtts.lab, Some(String::from("ABC")));
        assert_eq!(cggtts.nb_channels, 12);
        assert_eq!(cggtts.ims, None);
        assert_eq!(cggtts.time_reference,
            TimeSystem::UTCk(String::from("ABC"), None));
        assert_eq!(cggtts.reference_frame, 
            Some(String::from(
                "ITRF, PZ-90->ITRF Dx = 0.0 m, Dy = 0.0 m, Dz = 0.0 m, ds = 0.0, Rx = 0.0, Ry = 0.0, Rz = 0.000000"
            )));
        assert!((cggtts.coordinates.x - 4027881.79).abs() < 1E-6);
        assert!((cggtts.coordinates.y - 306998.67).abs() < 1E-6);
        assert!((cggtts.coordinates.z - 4919499.36).abs() < 1E-6);
        assert_eq!(cggtts.comments, None);
        assert_eq!(cggtts.delay.value(), 53.9 + 237.0 + 149.6);
        /*assert_eq!(cggtts.delay.calib_delay, CalibratedDelay {
            cal_id: None,
            channel: Channel::L1,
            delay: Delay::Internal(53.9),
        });*/
        
        assert_eq!(cggtts.tracks.len(), 4);
        let first = cggtts.tracks.first();
        assert_eq!(first.is_some(), true);
        let first = first.unwrap();
        assert_eq!(first.space_vehicule, Sv {
            constellation: Constellation::Glonass,
            prn: 24,
        });
        assert_eq!(first.fr, GlonassChannel::Channel(2));

        let _dumped = cggtts.to_string();
        let _compare = std::fs::read_to_string(
            &(env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/advanced/RZSY8257.000")).unwrap();
    }
}
