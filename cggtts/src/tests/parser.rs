mod test {

    use hifitime::Unit;

    use crate::{
        header::{CalibrationID, Code, Coordinates, Delay},
        prelude::{Constellation, Epoch, Hardware, ReferenceTime, CGGTTS, SV},
        tests::toolkit::random_name,
        track::CommonViewClass,
    };
    use std::{
        fs::read_dir,
        io::Write,
        path::{Path, PathBuf},
    };

    #[test]
    fn single_frequency_files() {
        let dir: PathBuf = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../data")
            .join("single");

        for entry in read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }

            let cggtts = CGGTTS::from_file(&path)
                .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));

            // test filename convention
            let stem = path.file_name().unwrap();
            let _stem = stem.to_string_lossy();
            // assert_eq!(cggtts.filename(), stem, "bad filename convention");

            // dump into file
            // let filename = random_name(8);
            // let mut fd = File::create(&filename).unwrap();
            // write!(fd, "{}", cggtts).unwrap();

            // // parse back
            // let parsed = CGGTTS::from_file(&filename);
            // assert!(
            //     parsed.is_ok(),
            //     "failed to parse back generated file: {}",
            //     parsed.err().unwrap()
            // );

            // println!("running testbench on \"{}\"", filename);
            // //TODO: hifitime pb
            // // cmp_dut_model(&parsed.unwrap(), &cggtts);

            // // remove generated file
            // let _ = std::fs::remove_file(&filename);
        }
    }

    #[test]
    fn dual_frequency_files() {
        let dir = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../data")
            .join("dual");

        for entry in read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }

            let cggtts = CGGTTS::from_file(&path)
                .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));

            let file_name = random_name(8);

            cggtts.to_file(&file_name).unwrap();
        }
    }

    #[test]
    fn gzsy8259_568() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("single")
            .join("GZSY8259.568");

        let cggtts = CGGTTS::from_file(&path);
        let cggtts = cggtts.unwrap();

        assert_eq!(
            cggtts.header.release_date,
            Epoch::from_gregorian_utc_at_midnight(2014, 02, 20),
        );

        assert!(cggtts.is_gps_cggtts());

        assert_eq!(
            cggtts.header.receiver,
            Some(
                Hardware::default()
                    .with_manufacturer("GORGYTIMING")
                    .with_model("SYREF25")
                    .with_serial_number("18259999")
                    .with_release_year(2018)
                    .with_release_version("v00")
            ),
        );

        assert_eq!(cggtts.header.station, "SY82");
        assert_eq!(cggtts.header.nb_channels, 12);
        assert!(cggtts.header.ims_hardware.is_none());

        assert_eq!(
            cggtts.header.reference_time,
            ReferenceTime::Custom(String::from("REF(SY82)"))
        );

        assert_eq!(cggtts.header.reference_frame, Some(String::from("ITRF")));
        assert!((cggtts.header.apc_coordinates.x - 4314143.824).abs() < 1E-6);
        assert!((cggtts.header.apc_coordinates.y - 452633.241).abs() < 1E-6);
        assert!((cggtts.header.apc_coordinates.z - 4660711.385).abs() < 1E-6);

        assert_eq!(cggtts.header.comments, None);

        assert_eq!(cggtts.tracks.len(), 32);

        let first = cggtts.tracks.first();
        //assert_eq!(cggtts.delay.value(), 0.0);
        assert!(first.is_some());
        let first = first.unwrap();
        assert_eq!(
            first.sv,
            SV {
                constellation: Constellation::GPS,
                prn: 99,
            }
        );

        assert_eq!(
            cggtts.standardized_file_name(Some("SY"), Some("82")),
            String::from("GSSY8259.568")
        );

        let tracks: Vec<_> = cggtts.tracks_iter().collect();
        assert_eq!(tracks.len(), 32);

        // let _dumped = cggtts.to_string();
        // let _compare = std::fs::read_to_string(
        //     env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/single/GZSY8259.568",
        // )
        // .unwrap();
    }

    #[test]
    fn ezgtr60_258() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("dual")
            .join("EZGTR60.258");

        let fullpath = path.to_string_lossy().to_string();

        let cggtts = CGGTTS::from_file(&fullpath).unwrap();

        assert!(cggtts.header.receiver.is_none());
        assert!(cggtts.header.ims_hardware.is_none());
        assert!(cggtts.is_galileo_cggtts());

        assert_eq!(
            cggtts.header.delay.calibration_id,
            Some(CalibrationID {
                process_id: 1015,
                year: 2021,
            })
        );

        let mut tests_passed = 0;

        for (nth, track) in cggtts.tracks_iter().enumerate() {
            match nth {
                0 => {
                    assert_eq!(track.sv.to_string(), "E03");
                    assert_eq!(track.class, CommonViewClass::MultiChannel);
                    assert_eq!(track.epoch.to_mjd_utc(Unit::Day).floor() as u16, 60258);
                    assert_eq!(track.duration.to_seconds() as u16, 780);
                    assert!(track.follows_bipm_tracking());
                    assert!((track.elevation_deg - 13.9).abs() < 0.01);
                    assert!((track.azimuth_deg - 54.8).abs() < 0.01);
                    assert!((track.data.refsv - 723788.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsv - 14.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.refsys - -302.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsys - -14.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.dsg - 2.0 * 0.1E-9).abs() < 1E-10);
                    assert_eq!(track.data.ioe, 76);

                    assert!((track.data.mdtr - 325.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdt - -36.0 * 0.1E-12).abs() < 1E-12);
                    assert!((track.data.mdio - 32.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdi - -3.0 * 0.1E-12).abs() < 1E-12);

                    let iono = track.iono.unwrap();
                    assert!((iono.msio - 20.0 * 0.1E-9).abs() < 1E-10);
                    assert!((iono.smsi - 20.0 * 0.1E-12).abs() < 1E-12);
                    assert!((iono.isg - 3.0 * 0.1E-9).abs() < 1E-10);

                    assert_eq!(track.frc, "E1");
                    tests_passed += 1;
                },
                _ => {},
            }
        }
        assert_eq!(tests_passed, 1);

        // test filename generator
        assert_eq!(
            cggtts.standardized_file_name(Some("GT"), Some("R")),
            "EZGTR60.258"
        );

        // format (dump) then parse back
    }

    #[test]
    fn gzgtr5_60_258() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("dual")
            .join("GZGTR560.258");

        let cggtts = CGGTTS::from_file(&path).unwrap();

        assert!(cggtts.header.receiver.is_none());
        assert!(cggtts.header.ims_hardware.is_none());
        assert!(cggtts.is_gps_cggtts());

        assert_eq!(
            cggtts.header.delay.calibration_id,
            Some(CalibrationID {
                process_id: 1015,
                year: 2021,
            })
        );

        let mut tests_passed = 0;

        for (nth, track) in cggtts.tracks_iter().enumerate() {
            match nth {
                0 => {
                    assert_eq!(track.sv.to_string(), "G08");
                    assert_eq!(track.class, CommonViewClass::MultiChannel);
                    assert_eq!(track.epoch.to_mjd_utc(Unit::Day).floor() as u16, 60258);
                    assert_eq!(track.duration.to_seconds() as u16, 780);
                    assert!(track.follows_bipm_tracking());
                    assert!((track.elevation_deg - 24.5).abs() < 0.01);
                    assert!((track.azimuth_deg - 295.4).abs() < 0.01);
                    assert!((track.data.refsv - 1513042.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsv - 28.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.refsys - -280.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsys - 10.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.dsg - 3.0 * 0.1E-9).abs() < 1E-10);
                    assert_eq!(track.data.ioe, 42);

                    assert!((track.data.mdtr - 192.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdt - -49.0 * 0.1E-12).abs() < 1E-12);
                    assert!((track.data.mdio - 99.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdi - -14.0 * 0.1E-12).abs() < 1E-12);

                    let iono = track.iono.unwrap();
                    assert!((iono.msio - 57.0 * 0.1E-9).abs() < 1E-10);
                    assert!((iono.smsi - -29.0 * 0.1E-12).abs() < 1E-12);
                    assert!((iono.isg - 5.0 * 0.1E-9).abs() < 1E-10);

                    assert_eq!(track.frc, "L1C");
                    tests_passed += 1;
                },
                _ => {},
            }
        }
        assert_eq!(tests_passed, 1);

        // test filename generator
        assert_eq!(
            cggtts.standardized_file_name(Some("GT"), Some("R5")),
            "GZGTR560.258"
        );

        // format (dump) then parse back
    }
}
