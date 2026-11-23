use std::str::FromStr;

use crate::prelude::{Duration, Epoch, IONEX, Point, Rect, TemporalInterpolation, coord};

fn ckmg() -> IONEX {
    IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap()
}

#[test]
fn temporal_resampling() {
    let ionex = ckmg();

    // reduce the scope to speed up the test
    let roi = Rect::new(coord!(x: -10.0, y: 35.0), coord!(x: 30.0, y: 60.0));
    let ionex = ionex.to_regional_ionex(roi.into()).unwrap();

    // upsampling
    let upsampled = ionex
        .temporally_resampled(Duration::from_seconds(900.0), TemporalInterpolation::Linear)
        .unwrap();

    assert_eq!(upsampled.header.number_of_maps, 97);
    assert_eq!(
        upsampled.header.sampling_period,
        Duration::from_seconds(900.0)
    );
    assert_eq!(
        upsampled.header.epoch_of_first_map,
        ionex.header.epoch_of_first_map
    );
    assert_eq!(
        upsampled.header.epoch_of_last_map,
        ionex.header.epoch_of_last_map
    );

    // original maps are preserved
    for (k, v) in ionex.record.iter() {
        let resampled = upsampled.record.get(k).unwrap();
        assert!((resampled.tecu() - v.tecu()).abs() < 1E-9);
    }

    // new maps are interpolated
    let t = Epoch::from_str("2022-01-02T12:15:00 UTC").unwrap();
    let point = Point::new(5.0, 45.0);
    assert!(
        (upsampled.tec_at(t, point).unwrap().tecu()
            - ionex
                .tec_at_with(t, point, TemporalInterpolation::Linear)
                .unwrap()
                .tecu())
        .abs()
            < 1E-9
    );

    // downsampling
    let downsampled = ionex
        .temporally_resampled(
            Duration::from_seconds(7200.0),
            TemporalInterpolation::Linear,
        )
        .unwrap();

    assert_eq!(downsampled.header.number_of_maps, 13);

    for (k, v) in downsampled.record.iter() {
        assert_eq!(ionex.record.get(k).unwrap().tecu(), v.tecu());
    }

    // stretching API
    let mut stretched = ionex.clone();
    stretched.temporal_stretching_mut(4.0).unwrap();
    assert_eq!(stretched, upsampled);

    assert!(
        ionex
            .temporally_resampled(Duration::ZERO, TemporalInterpolation::Linear)
            .is_err()
    );
}

#[test]
fn spatial_resampling() {
    let ionex = ckmg();

    let roi = Rect::new(coord!(x: -10.0, y: 35.0), coord!(x: 30.0, y: 60.0));
    let regional = ionex.to_regional_ionex(roi.into()).unwrap();

    let upsampled = regional.spatially_resampled(0.5, 1.0).unwrap();

    assert_eq!(upsampled.header.grid.latitude.spacing, -0.5);
    assert_eq!(upsampled.header.grid.longitude.spacing, 1.0);
    assert_eq!(
        upsampled.bounding_rect_degrees(),
        regional.bounding_rect_degrees()
    );

    // 25 maps of 51 x 41 nodes
    assert_eq!(upsampled.record.iter().count(), 25 * 51 * 41);

    // original nodes are preserved
    for (k, v) in regional.record.iter() {
        assert_eq!(upsampled.record.get(k).unwrap().tecu(), v.tecu());
    }

    // back to the original grid
    let downsampled = upsampled.spatially_resampled(2.5, 5.0).unwrap();
    assert_eq!(downsampled.header.grid, regional.header.grid);
    assert_eq!(downsampled.record, regional.record);

    // invalid spacing
    assert!(regional.spatially_resampled(0.7, 1.0).is_err());

    // formatting reciprocity
    upsampled.to_file("spatially-resampled.txt").unwrap();
    let parsed = IONEX::from_file("spatially-resampled.txt").unwrap();
    let _ = std::fs::remove_file("spatially-resampled.txt");

    assert_eq!(parsed.header.grid, upsampled.header.grid);

    for (k, v) in upsampled.record.iter() {
        let parsed = parsed.record.get(k).unwrap();
        // formatting is limited to the file precision (0.1 TECu with usual exponent)
        assert!((parsed.tecu() - v.tecu()).abs() <= 0.05 + 1E-9);
    }
}
