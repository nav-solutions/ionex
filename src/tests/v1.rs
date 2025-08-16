use crate::{
    prelude::{Duration, MappingFunction, ReferenceSystem, Version, IONEX},
    tests::{
        init_logger,
        toolkit::{generic_test, TestPoint},
    },
};

use std::fs::File;
use std::io::{BufWriter, Write};

#[test]
fn parse_ckmg0020() {
    init_logger();

    let ionex = IONEX::from_gzip_file("../data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
        panic!("Failed to parse CKMG0020: {}", e);
    });

    assert!(ionex.is_2d());
    assert!(!ionex.is_3d());

    generic_test(
        &ionex,
        vec![
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: -180.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: -175.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: -170.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: 0.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: -5.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: 5.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 85.0,
                long_ddeg: -180.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 85.0,
                long_ddeg: 180.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 42.5,
                long_ddeg: 125.0,
                alt_km: 350.0,
                tecu: 10.6,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 42.5,
                long_ddeg: 130.0,
                alt_km: 350.0,
                tecu: 10.8,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 42.5,
                long_ddeg: 135.0,
                alt_km: 350.0,
                tecu: 10.8,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 42.5,
                long_ddeg: 140.0,
                alt_km: 350.0,
                tecu: 10.7,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T00:00:00 UTC",
                lat_ddeg: 42.5,
                long_ddeg: 145.0,
                alt_km: 350.0,
                tecu: 10.4,
                rms: None,
            },
            TestPoint {
                epoch_str: "2022-01-02T05:00:00 UTC",
                lat_ddeg: 87.5,
                long_ddeg: -180.0,
                alt_km: 350.0,
                tecu: 9.2,
                rms: None,
            },
            // TestPoint {
            //     epoch_str: "2022-01-02T05:00:00 UTC",
            //     lat_ddeg: 25.0,
            //     long_ddeg: -180.0,
            //     alt_km: 350.0,
            //     tecu: 23.8,
            //     rms: None,
            // },
            // TestPoint {
            //     epoch_str: "2022-01-02T05:00:00 UTC",
            //     lat_ddeg: -82.5,
            //     long_ddeg: -180.0,
            //     alt_km: 350.0,
            //     tecu: 13.2,
            //     rms: None,
            // },
            // TestPoint {
            //     epoch_str: "2022-01-02T05:00:00 UTC",
            //     lat_ddeg: -87.5,
            //     long_ddeg: -180.0,
            //     alt_km: 350.0,
            //     tecu: 13.2,
            //     rms: None,
            // },
            // TestPoint {
            //     epoch_str: "2022-01-02T05:00:00 UTC",
            //     lat_ddeg: -87.5,
            //     long_ddeg: 175.5,
            //     alt_km: 350.0,
            //     tecu: 13.6,
            //     rms: None,
            // },
            // TestPoint {
            //     epoch_str: "2022-01-02T06:00:00 UTC",
            //     lat_ddeg: 87.5,
            //     long_ddeg: -180.0,
            //     alt_km: 350.0,
            //     tecu: 9.2,
            //     rms: None,
            // },
        ],
        0.1,
        1.0,
    );

    // check no RMS value
    for (k, tec) in ionex.record.iter() {
        // verify RMS map
        assert!(tec.root_mean_square().is_none());

        // fixed altitude (2D)
        assert_eq!(k.altitude_km(), 350.0);
    }

    assert_eq!(ionex.header.version, Version::new(1, 0));

    assert_eq!(ionex.header.program, Some("BIMINX V5.3".to_string()));
    assert_eq!(ionex.header.run_by, Some("AIUB".to_string()));
    assert_eq!(ionex.header.date, Some("07-JAN-22 07:51".to_string()));

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

    assert_eq!(ionex.header.base_radius_km, 6371.0);
    assert_eq!(ionex.header.mapf, MappingFunction::None);

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
    assert_eq!(ionex.header.exponent, -1);

    assert_eq!(ionex.header.comments.len(), 2);

    assert_eq!(
        ionex.header.comments[0],
        "CODE'S KLOBUCHAR-STYLE IONOSPHERE MODEL FOR DAY 002, 2022"
    );

    assert_eq!(
        ionex.header.comments[1],
        "TEC/RMS values in 0.1 TECU; 9999, if no value available"
    );

    // dump as file
    let mut fd = File::create("ckmg-v1.txt").unwrap();
    let mut writer = BufWriter::new(fd);

    ionex.format(&mut writer).unwrap_or_else(|e| {
        panic!("failed to format CKMG V1: {}", e);
    });

    // parse back
    let parsed = IONEX::from_file("ckmg-v1.txt").unwrap_or_else(|e| {
        panic!("failed to parse back CKMG V1: {}", e);
    });
}

#[test]
fn parse_jplg() {
    init_logger();

    let ionex = IONEX::from_gzip_file("../data/IONEX/V1/jplg0010.17i.gz").unwrap_or_else(|e| {
        panic!("Failed to parse V1/jplg0010.17i.gz: {}", e);
    });

    assert!(ionex.is_2d());
    assert!(!ionex.is_3d());

    generic_test(
        &ionex,
        vec![TestPoint {
            epoch_str: "2017-01-01T00:00:00 UTC",
            lat_ddeg: 87.5,
            long_ddeg: -180.0,
            alt_km: 450.0,
            tecu: 3.3,
            rms: None,
        }],
        0.1,
        1.0,
    );

    for (k, tec) in ionex.record.iter() {
        // verify RMS map
        assert!(tec.root_mean_square().is_some());

        // fixed altitude (2D)
        assert_eq!(k.altitude_km(), 450.0);
    }

    assert_eq!(ionex.header.version, Version::new(1, 0));

    assert_eq!(ionex.header.program, Some("GIM V3.0".to_string()));
    assert_eq!(ionex.header.run_by, Some("JPL - GNISD".to_string()));
    assert_eq!(ionex.header.date, Some("04-jan-2017 02:12".to_string()));

    assert!(ionex.header.doi.is_none());
    assert!(ionex.header.license.is_none());

    assert_eq!(ionex.header.number_of_maps, 13);

    assert_eq!(
        ionex.header.epoch_of_first_map.to_string().as_str(),
        "2017-01-01T00:00:00 UTC"
    );

    assert_eq!(
        ionex.header.epoch_of_last_map.to_string().as_str(),
        "2017-01-02T00:00:00 UTC"
    );

    assert_eq!(ionex.header.num_stations, 170);
    assert_eq!(ionex.header.num_satellites, 31);

    assert_eq!(ionex.header.base_radius_km, 6371.0);
    assert_eq!(ionex.header.mapf, MappingFunction::None);

    assert_eq!(ionex.header.sampling_period, Duration::from_hours(1.0));

    assert_eq!(ionex.header.grid.latitude.start, 87.5);
    assert_eq!(ionex.header.grid.latitude.end, -87.5);
    assert_eq!(ionex.header.grid.latitude.spacing, -2.5);

    assert_eq!(ionex.header.grid.longitude.start, -180.0);
    assert_eq!(ionex.header.grid.longitude.end, 180.0);
    assert_eq!(ionex.header.grid.longitude.spacing, 5.0);

    assert_eq!(ionex.header.grid.altitude.start, 450.0);
    assert_eq!(ionex.header.grid.altitude.end, 450.0);
    assert_eq!(ionex.header.grid.altitude.spacing, 0.0);

    assert_eq!(ionex.header.elevation_cutoff, 10.0);
    assert_eq!(ionex.header.exponent, -1);

    assert_eq!(ionex.header.comments.len(), 2);

    assert_eq!(
        ionex.header.comments[0],
        "CODE'S KLOBUCHAR-STYLE IONOSPHERE MODEL FOR DAY 002, 2022"
    );

    assert_eq!(
        ionex.header.comments[1],
        "TEC/RMS values in 0.1 TECU; 9999, if no value available"
    );

    // dump as file
    let mut fd = File::create("jplg-v1.txt").unwrap();
    let mut writer = BufWriter::new(fd);
    ionex.format(&mut writer).unwrap_or_else(|e| {
        panic!("failed to format JPLG V1: {}", e);
    });

    // parse back
    let parsed = IONEX::from_file("jplg-v1.txt").unwrap_or_else(|e| {
        panic!("failed to parse back JPLG V1: {}", e);
    });
}
