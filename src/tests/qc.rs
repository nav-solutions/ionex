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

    // reciprocal
    let fd = File::create("merged.txt").unwrap();
    let mut writer = BufWriter::new(fd);

    merged.format(&mut writer).unwrap_or_else(|e| {
        panic!("failed to format V1 file: {}", e);
    });

    // parse back
    let parsed = IONEX::from_file("merged.txt").unwrap_or_else(|e| {
        panic!("failed to parse back CKMG V1: {}", e);
    });

    // full reciprocity
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

    // verifications
    let fd = File::create("merged.txt").unwrap();
    let mut writer = BufWriter::new(fd);

    merged.format(&mut writer).unwrap_or_else(|e| {
        panic!("failed to format V1 file: {}", e);
    });

    // parse back
    let parsed = IONEX::from_file("ckmg-v1.txt").unwrap_or_else(|e| {
        panic!("failed to parse back CKMG V1: {}", e);
    });

    // TODO
}
