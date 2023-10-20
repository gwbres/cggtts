mod test {
    use crate::Cggtts;
    use std::path::PathBuf;
    #[test]
    fn test_standard_pool() {
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
            let cggtts = Cggtts::from_file(&fp);
            assert!(
                cggtts.is_ok(),
                "failed to parse {} - {:?}",
                fp,
                cggtts.err()
            );
        }
    }
    #[test]
    fn test_advanced_pool() {
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
            let cggtts = Cggtts::from_file(&fp);
            assert!(
                cggtts.is_ok(),
                "failed to parse {} - {:?}",
                fp,
                cggtts.err()
            );
        }
    }
}
