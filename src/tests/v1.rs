use crate::prelude::{Duration, MappingFunction, ReferenceSystem, Version, IONEX};

#[test]
fn parse_ckmg0020() {
    let ionex = IONEX::from_gzip_file("../data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    assert!(ionex.is_2d());

    assert_eq!(ionex.header.version, Version::new(1, 0));

    assert_eq!(ionex.header.program.unwrap(), "BIMINX V5.3");
    assert_eq!(ionex.header.run_by.unwrap(), "AIUB");
    assert_eq!(ionex.header.date.unwrap(), "07-JAN-22 07:51");

    assert!(ionex.header.doi.is_none());
    assert!(ionex.header.license.is_none());

    assert_eq!(ionex.header.number_of_maps, 25);
    assert_eq!(
        ionex.header.epoch_of_first_map.to_string().as_str(),
        "2022-01-02T00:00:00 UTC"
    );
    assert_eq!(
        ionex.header.epoch_of_last_map.to_string().as_str(),
        "2022-01-03T00:00:00 UTC"
    );

    // assert_eq!(ionex.header.reference_system, ReferenceSystem::);

    assert_eq!(ionex.header.mapf, MappingFunction::None);
    assert_eq!(ionex.header.base_radius_km, 6371.0);

    assert_eq!(ionex.header.sampling_period, Duration::from_hours(1.0));

    assert_eq!(ionex.header.grid.latitude.start, 87.5);
    assert_eq!(ionex.header.grid.latitude.end, -87.5);
    assert_eq!(ionex.header.grid.latitude.spacing, -2.5);

    assert_eq!(ionex.header.grid.longitude.start, -180.0);
    assert_eq!(ionex.header.grid.longitude.end, 180.0);
    assert_eq!(ionex.header.grid.longitude.spacing, 5.0);

    assert_eq!(ionex.header.grid.altitude.start, 350.0);
    assert_eq!(ionex.header.grid.altitude.end, 350.0);
    assert_eq!(ionex.header.grid.altitude.spacing, 0.0);

    assert_eq!(ionex.header.elevation_cutoff, 0.0);

    assert_eq!(ionex.header.comments.len(), 2);

    assert_eq!(
        ionex.header.comments[0],
        "CODE'S KLOBUCHAR-STYLE IONOSPHERE MODEL FOR DAY 002, 2022"
    );
    assert_eq!(
        ionex.header.comments[1],
        "TEC/RMS values in 0.1 TECU; 9999, if no value available"
    );
}
