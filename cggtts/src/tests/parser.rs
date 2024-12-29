mod test {
    use crate::{
        header::{CalibrationID, Code, Coordinates, Delay},
        prelude::{Constellation, Epoch, Hardware, ReferenceTime, CGGTTS, SV},
        tests::toolkit::random_name,
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

        let fullpath = path.to_string_lossy().to_string();
        let cggtts = CGGTTS::from_file(&fullpath);
        assert!(cggtts.is_ok());

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
            cggtts.standardized_file_name(),
            String::from("GSSY1859.568")
        );

        let tracks: Vec<_> = cggtts.tracks().collect();
        assert_eq!(tracks.len(), 32);

        // let _dumped = cggtts.to_string();
        // let _compare = std::fs::read_to_string(
        //     env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/single/GZSY8259.568",
        // )
        // .unwrap();
    }

    #[test]
    fn rzsy8257_000() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("dual")
            .join("RZSY8257.000");

        let fullpath = path.to_string_lossy().to_string();

        let cggtts = CGGTTS::from_file(&fullpath).unwrap();

        assert!(cggtts.header.receiver.is_none());
        assert!(cggtts.header.ims_hardware.is_none());
        assert!(cggtts.is_glonass_cggtts());

        assert_eq!(
            cggtts.header.apc_coordinates,
            Coordinates {
                x: 4027881.79,
                y: 306998.67,
                z: 4919499.36,
            }
        );

        assert!(cggtts.header.comments.is_none());
        assert_eq!(cggtts.header.station, "ABC");
        assert_eq!(cggtts.header.nb_channels, 12);

        assert_eq!(cggtts.header.delay.antenna_cable_delay, 237.0);
        assert_eq!(cggtts.header.delay.local_ref_delay, 149.6);
        assert_eq!(cggtts.header.delay.freq_dependent_delays.len(), 2);

        assert_eq!(
            cggtts.header.delay.freq_dependent_delays[0],
            (Code::C1, Delay::Internal(53.9))
        );

        let delay_nanos = cggtts
            .header
            .delay
            .total_frequency_dependent_delay_nanos(&Code::C1)
            .unwrap();

        let err_nanos = (delay_nanos - (53.9 + 237.0 + 149.6)).abs();
        assert!(err_nanos < 1.0);

        let delay_nanos = cggtts
            .header
            .delay
            .total_frequency_dependent_delay_nanos(&Code::C2)
            .unwrap();

        let err_nanos = (delay_nanos - (49.8 + 237.0 + 149.6)).abs();
        assert!(err_nanos < 1.0);

        assert!(cggtts.header.delay.calibration_id.is_none());

        let tracks: Vec<_> = cggtts.tracks().collect();
        assert_eq!(tracks.len(), 4);
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
    }
}
