use cggtts::Rcvr;
use cggtts::Cggtts;
use rinex::sv::Sv;
use rinex::constellation::Constellation;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_standard_cggtts() {
        let cggtts = Cggtts::from_file(
            &(env!("CARGO_MANIFEST_DIR").to_owned() + "/data/standard/GZSY8259.568"));
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
        assert_eq!(cggtts.time_reference, Some(String::from("REF(SY82)")));
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

        let dumped = cggtts.to_string();
        let compare = std::fs::read_to_string(
            &(env!("CARGO_MANIFEST_DIR").to_owned() + "/data/standard/GZSY8259.568")).unwrap();
        println!("{:#?}", dumped);
    }
    #[test]
    fn parse_standard_data() {
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/standard");
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
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/advanced");
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
}
