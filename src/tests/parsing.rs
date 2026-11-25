use crate::prelude::IONEX;

#[test]
fn repo_parsing() {
    let prefix = "data/IONEX/V1";

    for file in std::fs::read_dir(prefix).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        let fullpath = &path.to_str().unwrap();

        let filename = file.file_name().to_str().unwrap().to_string();

        if filename.starts_with('.') {
            continue;
        }

        println!("parsing \"{}\"", fullpath);

        if filename.ends_with(".gz") {
            IONEX::from_gzip_file(fullpath).unwrap_or_else(|e| {
                panic!("Failed to parse \"{}\": {}", fullpath, e);
            });
        } else {
            IONEX::from_file(fullpath).unwrap_or_else(|e| {
                panic!("Failed to parse \"{}\": {}", fullpath, e);
            });
        }
    }
}

#[test]
#[cfg(feature = "flate2")]
fn record_comments_roundtrip() {
    use std::io::{BufReader, BufWriter};

    let mut ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap();

    ionex.comments = vec![
        "first record comment".to_string(),
        "second comment with numbers 12 34 56".to_string(),
    ];

    let mut buf = Vec::<u8>::new();
    {
        let mut writer = BufWriter::new(&mut buf);
        ionex.format(&mut writer).unwrap();
    }

    let parsed = IONEX::parse(&mut BufReader::new(&buf[..])).unwrap();

    // comments are formatted with the standard COMMENT marker
    // and must be identified as such (never as data)
    assert_eq!(parsed.comments, ionex.comments);
    assert_eq!(parsed.record, ionex.record);
}
