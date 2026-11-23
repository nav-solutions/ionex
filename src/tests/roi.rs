use std::str::FromStr;

use crate::{
    prelude::{Epoch, IONEX, Point, Rect, coord},
    tests::init_logger,
};

#[test]
fn worldwide_map() {
    init_logger();

    let mut ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    // returned from file name directly
    assert!(
        ionex.is_worldwide_map(),
        "Worldwide map from standardized file name!"
    );

    // destroy and verify this still works
    ionex.attributes = None;
    assert!(ionex.is_worldwide_map(), "Guessed worldwide map !");

    // reduce to ROI
    let roi = Rect::new(coord!(x: -30.0, y: -30.0), coord!(x: 30.0, y: 30.0));

    let reduced = ionex.to_regional_ionex(roi.into()).unwrap();

    assert!(
        reduced.is_regional_map(),
        "Worldwide map reduced to Regional ROI"
    );

    // test new map
    let bounding_rect = reduced.bounding_rect_degrees();
    assert_eq!(bounding_rect, roi);

    // axes orientation is preserved
    assert_eq!(reduced.header.grid.latitude.start, 30.0);
    assert_eq!(reduced.header.grid.latitude.end, -30.0);
    assert_eq!(reduced.header.grid.longitude.start, -30.0);
    assert_eq!(reduced.header.grid.longitude.end, 30.0);

    // 25 maps of 25 latitudes x 13 longitudes
    assert_eq!(reduced.record.iter().count(), 25 * 25 * 13);

    for (k, _) in reduced.record.iter() {
        assert!(k.latitude_ddeg() >= -30.0 && k.latitude_ddeg() <= 30.0);
        assert!(k.longitude_ddeg() >= -30.0 && k.longitude_ddeg() <= 30.0);
    }

    // same values within the ROI
    let t = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();
    let point = Point::new(1.44, 13.6);
    assert_eq!(
        reduced.tec_at(t, point).unwrap(),
        ionex.tec_at(t, point).unwrap()
    );
    assert!(reduced.tec_at(t, Point::new(31.0, 0.0)).is_err());

    // the ROI is extended to the grid
    let roi = Rect::new(coord!(x: -29.0, y: -29.0), coord!(x: 29.0, y: 29.0));
    let reduced = ionex.to_regional_ionex(roi.into()).unwrap();
    assert_eq!(
        reduced.bounding_rect_degrees(),
        Rect::new(coord!(x: -30.0, y: -30.0), coord!(x: 30.0, y: 30.0))
    );

    // and saturated to the map
    let roi = Rect::new(coord!(x: 170.0, y: 80.0), coord!(x: 200.0, y: 90.0));
    let reduced = ionex.to_regional_ionex(roi.into()).unwrap();
    assert_eq!(
        reduced.bounding_rect_degrees(),
        Rect::new(coord!(x: 170.0, y: 80.0), coord!(x: 180.0, y: 87.5))
    );

    // formatting reciprocity
    reduced.to_file("regional-roi.txt").unwrap();
    let parsed = IONEX::from_file("regional-roi.txt").unwrap();
    assert_eq!(parsed.header.grid, reduced.header.grid);
    assert_eq!(parsed.record, reduced.record);
    let _ = std::fs::remove_file("regional-roi.txt");
}
