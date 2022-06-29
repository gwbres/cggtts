#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    /// Tests default constructor 
    fn cggtts_test_default() {
        let cggtts = Cggtts::new();
        assert_eq!(cggtts.lab, "Unknown"); // default
        assert_eq!(cggtts.nb_channels, 0); // default
        assert_eq!(cggtts.frame, "?"); // default ..
        assert_eq!(cggtts.reference, "Unknown"); // default..
        assert_eq!(cggtts.coordinates, (0.0,0.0,0.0)); // empty..
        assert_eq!(cggtts.rev_date,
            chrono::NaiveDate::parse_from_str(LATEST_REV_DATE, "%Y-%m-%d")
            .unwrap());
        assert_eq!(cggtts.date, chrono::Utc::today().naive_utc());
        assert_eq!(cggtts.tot_dly.is_none(), true);
        assert_eq!(cggtts.int_dly.is_none(), true);
        assert_eq!(cggtts.sys_dly.is_none(), true);
        assert_eq!(cggtts.cab_dly, 0.0);
        assert_eq!(cggtts.ref_dly, 0.0);
        println!("{:#?}", cggtts.total_delay());
        assert_eq!(cggtts.total_delay().values.len(), 1); // single freq Cggts by default
        assert_eq!(cggtts.total_delay().values[0], 0.0); // not specified
        println!("{:#?}", cggtts)
    }

    #[test]
    /// Tests basic usage 
    fn cggtts_basic_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        //cggtts.set_total_delay(300E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        //assert_eq!(cggtts.get_system_delay().is_none(), true); // not provided
        //assert_eq!(cggtts.get_cable_delay().is_none(), true); // not provided
        //assert_eq!(cggtts.get_reference_delay().is_none(), true); // not provided
        //assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        //assert_eq!(cggtts.get_total_delay().unwrap(), 300E-9); // basic usage
        println!("{:#?}", cggtts)
    }

    #[test]
    /// Test normal / intermediate usage
    fn cgggts_intermediate_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        //cggtts.set_reference_delay(100E-9);
        //cggtts.set_system_delay(150E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        //assert_eq!(cggtts.get_cable_delay().is_some(), false); // not provided
        //assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_system_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        //assert_eq!(cggtts.get_total_delay().unwrap(), 250E-9); // intermediate usage
        println!("{:#?}", cggtts)
    }

    #[test]
    /// Test advanced usage
    fn cgggts_advanced_use_case() {
        let mut cggtts = Cggtts::new();
        cggtts.set_lab_agency("TestLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_cable_delay(300E-9);
        //cggtts.set_reference_delay(100E-9);
        //cggtts.set_internal_delay(25E-9);
        assert_eq!(cggtts.get_lab_agency(), "TestLab");
        assert_eq!(cggtts.get_nb_channels(), 10);
        assert_eq!(cggtts.get_antenna_coordinates(), (1.0,2.0,3.0));
        //assert_eq!(cggtts.get_system_delay().is_some(), false); // not provided: we have granularity
        //assert_eq!(cggtts.get_cable_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_internal_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_reference_delay().is_some(), true); // provided
        //assert_eq!(cggtts.get_total_delay().is_ok(), true); // enough information
        /*assert!(
            approx_eq!(f64,
                cggtts.get_total_delay().unwrap(),
                425E-9, // advanced usage
                epsilon = 1E-9
            )
        );*/
        println!("{:#?}", cggtts)
    }
    
    #[test]
    /// Tests standard file parsing
    fn cggtts_test_from_standard_data() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/standard");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                let cggtts = Cggtts::from_file(&fp);
                assert_eq!(
                    cggtts.is_err(),
                    false,
                    "Cggtts::from_file() failed for {:#?} with {:#?}",
                    path,
                    cggtts);
                println!("File \"{:?}\" {:#?}", &path, cggtts.unwrap())
            }
        }
    }
    #[test]
    /// Tests advanced file parsing
    fn cggtts_test_from_ionospheric_data() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/ionospheric");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                let cggtts = Cggtts::from_file(&fp);
                assert_eq!(
                    cggtts.is_err(), 
                    false,
                    "Cggtts::from_file() failed for {:#?} with {:#?}",
                    path, 
                    cggtts);
                println!("File \"{:?}\" {:#?}", &path, cggtts.unwrap())
            }
        }
    }

    #[test]
    /// Tests basci `Cggtts` to file
    fn default_cggtts_to_file() {
        let cggtts = Cggtts::default();
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD0").is_err(), false)
    }

    #[test]
    /// Tests customized `Cggtts` to file
    fn basic_cggtts_to_file() {
        let mut cggtts = Cggtts::default();

        // identify receiver hw
        let rcvr = Rcvr {
            manufacturer: String::from("SomeManufacturer"),
            recv_type: String::from("SomeKind"), 
            serial_number: String::from("XXXXXX"), 
            year: 2021, 
            software_number: String::from("v00"),
        };
        cggtts.set_rcvr_infos(rcvr);

        // add some more infos
        cggtts.set_lab_agency("MyLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_time_reference("UTC(k)");

        // define a total delay
        let delay = CalibratedDelay {
            constellation: track::Constellation::Glonass,
            values: vec![100E-9_f64],
            codes: vec![String::from("C1")], 
            report: String::from("NA"),
        };
        cggtts.set_total_delay(delay);
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD1").is_err(), false)
    }
    
    #[test]
    /// Tests customized `Cggtts` to file
    fn dual_frequency_cggtts_to_file() {
        let mut cggtts = Cggtts::default();

        // identify receiver hw
        let rcvr = Rcvr {
            manufacturer: String::from("SomeManufacturer"),
            recv_type: String::from("SomeKind"), 
            serial_number: String::from("XXXXXX"), 
            year: 2021, 
            software_number: String::from("v00"),
        };
        cggtts.set_rcvr_infos(rcvr);

        // add some more infos
        cggtts.set_lab_agency("MyLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_time_reference("UTC(k)");

        // set a total delay
        let total_delay = CalibratedDelay {
            constellation: track::Constellation::Glonass,
            values: vec![100E-9, 150E-9],
            codes: vec![String::from("C1"),String::from("C2")], 
            report: String::from("NA"),
        };
        cggtts.set_total_delay(total_delay);
        println!("{:#?}",cggtts);
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD2").is_err(), false)
    }
    
    #[test]
    /// Tests customized `Cggtts` to file (B)
    fn cggtts_with_system_delay_to_file() {
        let mut cggtts = Cggtts::default();

        // identify receiver hw
        let rcvr = Rcvr {
            manufacturer: String::from("SomeManufacturer"),
            recv_type: String::from("SomeKind"), 
            serial_number: String::from("XXXXXX"), 
            year: 2021, 
            software_number: String::from("v00"),
        };
        cggtts.set_rcvr_infos(rcvr);

        // add some more infos
        cggtts.set_lab_agency("MyLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_time_reference("UTC(k)");

        // define a total delay
        let delay = CalibratedDelay {
            constellation: track::Constellation::Glonass,
            values: vec![100E-9_f64],
            codes: vec![String::from("C2")], 
            report: String::from("NA"),
        };
        cggtts.set_system_delay(delay);
        cggtts.set_cable_delay(50E-9);
        cggtts.set_ref_delay(100E-9);
        let total_delay = cggtts.total_delay();
        assert_eq!(total_delay.values.len(), 1); // single freq
        assert_eq!(total_delay.values[0], 100E-9+50E-9); // single freq
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD3").is_err(), false)
    }
    
    #[test]
    /// Tests customized `Cggtts` to file (C)
    fn cggtts_with_internal_delay_to_file() {
        let mut cggtts = Cggtts::default();

        // identify receiver hw
        let rcvr = Rcvr {
            manufacturer: String::from("SomeManufacturer"),
            recv_type: String::from("SomeKind"), 
            serial_number: String::from("XXXXXX"), 
            year: 2021, 
            software_number: String::from("v00"),
        };
        cggtts.set_rcvr_infos(rcvr);

        // add some more infos
        cggtts.set_lab_agency("MyLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_time_reference("UTC(k)");

        // define a total delay
        let delay = CalibratedDelay {
            constellation: track::Constellation::GPS,
            values: vec![25E-9_f64],
            codes: vec![String::from("C1")], 
            report: String::from("NA"),
        };
        cggtts.set_internal_delay(delay);
        cggtts.set_cable_delay(100E-9);
        cggtts.set_ref_delay(50E-9);
        let total_delay = cggtts.total_delay();
        assert_eq!(total_delay.values.len(), 1); // single freq
        assert_eq!(cggtts.total_delay().values[0], 25E-9+25E-9+100E-9); 
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD4").is_err(), false)
    }
    
    #[test]
    /// Another test..
    fn cggtts_with_ionospheric_parameters () {
        let mut cggtts = Cggtts::default();

        // identify receiver hw
        let rcvr = Rcvr {
            manufacturer: String::from("SomeManuf1"),
            recv_type: String::from("SomeKind1"), 
            serial_number: String::from("XXXXXX"), 
            year: 2021, 
            software_number: String::from("v01"),
        };
        cggtts.set_rcvr_infos(rcvr);

        // IMS infos
        let ims = Rcvr {
            manufacturer: String::from("SomeManuf2"),
            recv_type: String::from("SomeKind2"), 
            serial_number: String::from("YYYY"), 
            year: 2022,
            software_number: String::from("v02"),
        };
        cggtts.set_ims_infos(ims);

        // add some more infos
        cggtts.set_lab_agency("MyLab");
        cggtts.set_nb_channels(10);
        cggtts.set_antenna_coordinates((1.0,2.0,3.0));
        cggtts.set_time_reference("UTC(USNO)");

        // define a delay
        let delay = CalibratedDelay {
            constellation: track::Constellation::GPS,
            values: vec![25E-9_f64],
            codes: vec![String::from("C1")], 
            report: String::from("NA"),
        };
        cggtts.set_internal_delay(delay);
        cggtts.set_cable_delay(100E-9);
        cggtts.set_ref_delay(50E-9);

        // add some measurements
        let mut track = track::CggttsTrack::default();
        track.set_satellite_id(0x01);
        cggtts.tracks.push(track);
        let mut track = track::CggttsTrack::default();
        track.set_satellite_id(0x11);
        cggtts.tracks.push(track);

        let total_delay = cggtts.total_delay();
        assert_eq!(total_delay.values.len(), 1); // single freq
        assert_eq!(cggtts.total_delay().values[0], 25E-9+25E-9+100E-9); 
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD5").is_err(), false)
    }
    #[test]
    /// Tests CRC calculation method
    fn test_crc_calc() {
        let content = vec![
            "R24 FF 57000 000600  780 347 394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P"
        ];
        let expected = vec![0x0F];
        for i in 0..content.len() {
            let ck = calc_crc(content[i])
                .unwrap();
            let expect = expected[i];
            assert_eq!(ck,expect,"Failed for \"{}\", expect \"{}\" but \"{}\" locally computed",content[i],expect,ck)
        }
    }
}
