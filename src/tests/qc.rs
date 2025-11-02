use crate::{
    prelude::{coord, Duration, MappingFunction, Rect, Version, IONEX},
    tests::{
        init_logger,
        toolkit::{generic_comparison, generic_test, TestPoint},
    },
};

use gnss_qc_traits::Merge;

use std::fs::File;
use std::io::BufWriter;

#[test]
fn v1_self_merge() {
    init_logger();

    let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    let merged = ionex.merge(&ionex).unwrap_or_else(|e| {
        panic!("self::merge(self) should be feasible! {}", e);
    });

    assert!(merged.is_merged(), "merged file not declared as merged!");

    // reciprocal
    let fd = File::create("merged.txt").unwrap();
    let mut writer = BufWriter::new(fd);

    merged.format(&mut writer).unwrap_or_else(|e| {
        panic!("failed to format V1 file: {}", e);
    });

    // parse back
    let mut parsed = IONEX::from_file("merged.txt").unwrap_or_else(|e| {
        panic!("failed to parse back CKMG V1: {}", e);
    });

    assert!(parsed.is_merged(), "merged file not declared as merged!");

    // remove special comment and run strict equality test
    parsed
        .header
        .comments
        .retain(|comment| !comment.eq("FILE MERGE"));

    generic_comparison(&parsed, &ionex);
}

#[test]
fn v1_files_merge() {
    init_logger();

    let file_a = IONEX::from_gzip_file("data/IONEX/V1/CKMG0090.21I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse V1 file: {}", e);
    });
    let file_b = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse V1 file: {}", e);
    });

    let merged = file_a.merge(&file_b).unwrap_or_else(|e| {
        panic!("failed to merge both files: {}", e);
    });

    // testbench
    assert!(merged.is_merged(), "merged file not declared as merged!");
    assert_eq!(
        merged.header.epoch_of_first_map.to_string(),
        "2021-01-09T00:00:00 UTC"
    );
    assert_eq!(
        merged.header.epoch_of_last_map.to_string(),
        "2022-01-03T00:00:00 UTC"
    );
    assert!(merged.is_worldwide_map());

    // Dump to file
    let fd = File::create("merged.txt").unwrap();
    let mut writer = BufWriter::new(fd);

    merged.format(&mut writer).unwrap_or_else(|e| {
        panic!("failed to format V1 file: {}", e);
    });

    // parse back
    let parsed = IONEX::from_file("merged.txt").unwrap_or_else(|e| {
        panic!("failed to parse back CKMG V1: {}", e);
    });

    // testbench
    assert!(parsed.is_merged(), "merged file not declared as merged!");
    assert_eq!(
        parsed.header.epoch_of_first_map.to_string(),
        "2021-01-09T00:00:00 UTC"
    );
    assert_eq!(
        parsed.header.epoch_of_last_map.to_string(),
        "2022-01-03T00:00:00 UTC"
    );
    assert!(parsed.is_worldwide_map());
}
