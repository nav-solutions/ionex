use crate::{
    prelude::{coord, Axis, Duration, MappingFunction, Rect, Version, IONEX},
    tests::{
        init_logger,
        toolkit::{generic_comparison, generic_test, TestPoint},
    },
};

use std::fs::File;
use std::io::BufWriter;

#[test]
fn unitary_spatial() {
    init_logger();

    let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    // dummy case
    let stretched = ionex
        .spatially_stretched(Axis::All, 1.0)
        .unwrap_or_else(|e| {
            panic!("unexpected error: {}", e);
        });

    assert_eq!(stretched, ionex);
}

#[test]
fn spatial_planar_downscaling() {
    init_logger();

    // 2D IONEX
    let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    for factor in [0.1, 0.2, 0.25, 0.33, 0.45, 0.5, 0.6, 0.7, 0.8, 0.9] {
        let bounding_rect = ionex.bounding_rect_degrees();
        let dimensions_deg = (bounding_rect.width(), bounding_rect.height());

        // Verify that z-axis does not modify anything (2D case)
        let stretched = ionex
            .spatially_stretched(Axis::Altitude, factor)
            .unwrap_or_else(|e| {
                panic!("unexpected error: {}", e);
            });

        assert_eq!(stretched.record, ionex.record, "z-axis on 2D IONEX");
        assert_eq!(
            stretched.bounding_rect_degrees(),
            bounding_rect,
            "z-axis on 2D IONEX"
        );

        let stretched = ionex
            .spatially_stretched(Axis::All, factor)
            .unwrap_or_else(|e| {
                panic!("unexpected error: {}", e);
            });

        let new_dimensions_deg = (
            stretched.bounding_rect_degrees().width(),
            stretched.bounding_rect_degrees().height(),
        );

        assert_eq!(
            new_dimensions_deg.0,
            dimensions_deg.0 * factor,
            "incorrect post-stretching dimensions"
        );
        assert_eq!(
            new_dimensions_deg.1,
            dimensions_deg.1 * factor,
            "incorrect post-stretching dimensions"
        );
    }
}

#[test]
fn spatial_planar_upscaling() {
    init_logger();

    // 2D IONEX
    let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    for factor in [1.1, 1.2, 1.25, 1.5, 1.75, 2.0, 3.0, 4.0] {
        let bounding_rect = ionex.bounding_rect_degrees();
        let dimensions_deg = (bounding_rect.width(), bounding_rect.height());

        // Verify that z-axis does not modify anything (2D case)
        let stretched = ionex
            .spatially_stretched(Axis::Altitude, factor)
            .unwrap_or_else(|e| {
                panic!("unexpected error: {}", e);
            });

        assert_eq!(stretched.record, ionex.record, "z-axis on 2D IONEX");
        assert_eq!(
            stretched.bounding_rect_degrees(),
            bounding_rect,
            "z-axis on 2D IONEX"
        );

        let stretched = ionex
            .spatially_stretched(Axis::All, factor)
            .unwrap_or_else(|e| {
                panic!("unexpected error: {}", e);
            });

        let new_dimensions_deg = (
            stretched.bounding_rect_degrees().width(),
            stretched.bounding_rect_degrees().height(),
        );

        assert_eq!(
            new_dimensions_deg.0,
            dimensions_deg.0 * factor,
            "incorrect post-stretching dimensions"
        );
        assert_eq!(
            new_dimensions_deg.1,
            dimensions_deg.1 * factor,
            "incorrect post-stretching dimensions"
        );
    }
}
