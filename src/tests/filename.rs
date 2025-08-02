use crate::prelude::*;

use std::path::Path;

#[test]
fn filename_conventions() {
    for testfile in [
        "CKMG0020.22I.gz",
        "CKMG0080.09I.gz",
        "CKMG0090.21I.gz",
        "jplg0010.17i.gz",
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join(testfile);

        let name = format!("data/IONEX/V1/{}", testfile);

        let ionex = IONEX::from_gzip_file(&name).unwrap_or_else(|e| {
            panic!("Failed to parse IONEX file \"{}\"", name);
        });

        let expected = testfile.to_uppercase();
        assert_eq!(ionex.standardized_filename(), expected);
    }
}
