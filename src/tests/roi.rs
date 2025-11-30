use crate::{
    prelude::{IONEX, Rect, coord},
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
}
