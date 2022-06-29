#[cfg(test)]
mod test {
    use super::*;
    /*
    #[test]
    fn from_standard_data() {
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/standard");
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
    fn from_advanced_data() {
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data/ionospheric");
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
    fn default_cggtts_to_file() {
        let cggtts = Cggtts::default();
        assert_eq!(cggtts.to_file("data/output/GZXXXXDD.DD0").is_err(), false)
    }*/
}
