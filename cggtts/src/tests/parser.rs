mod test {
    use crate::tests::toolkit::{cmp_dut_model, random_name};
    use crate::CGGTTS;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    #[test]
    fn single_frequency_files() {
        let resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../data")
            .join("standard");
        for entry in std::fs::read_dir(resources).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }
            let fp = path.to_string_lossy().to_string();
            println!("parsing \"{}\"", fp);

            let cggtts = CGGTTS::from_file(&fp);
            assert!(
                cggtts.is_ok(),
                "failed to parse {} - {:?}",
                fp,
                cggtts.err()
            );

            let cggtts = cggtts.unwrap();

            // dump into file
            let filename = random_name(8);
            let mut fd = File::create(&filename).unwrap();
            write!(fd, "{}", cggtts).unwrap();

            // remove generated file
            let _ = std::fs::remove_file(&filename);
        }
    }
    #[test]
    fn dual_frequency_files() {
        let resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../data")
            .join("advanced");
        for entry in std::fs::read_dir(resources).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }
            let fp = path.to_string_lossy().to_string();
            println!("parsing \"{}\"", fp);
            let cggtts = CGGTTS::from_file(&fp);
            assert!(
                cggtts.is_ok(),
                "failed to parse {} - {:?}",
                fp,
                cggtts.err()
            );
        }
    }
}
