use crate::prelude::*;

#[test]
fn filename_conventions() {
    for testfile in ["CKMG0020.22I.gz", "CKMG0080.09I.gz", "CKMG0090.21I.gz"] {
        let name = format!("data/IONEX/V1/{}", testfile);
        let ionex = IONEX::from_gzip_file(&name).unwrap();
        assert_eq!(ionex.generate_standardized_filename(), testfile);
    }
}
