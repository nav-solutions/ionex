use std::str::FromStr;

use crate::{
    error::Error,
    linspace::Linspace,
    prelude::{
        Duration, Epoch, Grid, Header, IONEX, Key, Point, Record, TEC, TemporalInterpolation,
    },
};

fn ckmg() -> IONEX {
    IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap()
}

fn record_tecu(ionex: &IONEX, epoch: Epoch, lat: f64, long: f64) -> f64 {
    let key = Key::from_decimal_degrees_km(epoch, lat, long, ionex.header.grid.altitude.start);
    ionex.record.get(&key).unwrap().tecu()
}

#[test]
fn ckmg_spatial_interpolation() {
    let ionex = ckmg();
    let noon = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();

    // grid nodes are preserved, for all strategies
    for (lat, long) in [(42.5, 0.0), (87.5, -180.0), (-87.5, 180.0), (0.0, 0.0)] {
        let expected = record_tecu(&ionex, noon, lat, long);

        for method in [
            TemporalInterpolation::NearestMap,
            TemporalInterpolation::Linear,
            TemporalInterpolation::RotatedMaps,
        ] {
            let tec = ionex
                .tec_at_with(noon, Point::new(long, lat), method)
                .unwrap();
            assert_eq!(tec.tecu(), expected, "({}, {}) {:?}", lat, long, method);
        }
    }

    // middle of an edge: average of the two nodes
    let (a, b) = (
        record_tecu(&ionex, noon, 42.5, 0.0),
        record_tecu(&ionex, noon, 42.5, 5.0),
    );
    let tec = ionex.tec_at(noon, Point::new(2.5, 42.5)).unwrap();
    assert!((tec.tecu() - (a + b) / 2.0).abs() < 1E-9);

    // center of a cell: average of the 4 corners
    let (c, d) = (
        record_tecu(&ionex, noon, 45.0, 0.0),
        record_tecu(&ionex, noon, 45.0, 5.0),
    );
    let tec = ionex.tec_at(noon, Point::new(2.5, 43.75)).unwrap();
    assert!((tec.tecu() - (a + b + c + d) / 4.0).abs() < 1E-9);

    // the interpolated value is always bounded by the 4 corners
    let tec = ionex.tec_at(noon, Point::new(1.44, 43.6)).unwrap().tecu();
    let (min, max) = (a.min(b).min(c).min(d), a.max(b).max(c).max(d));
    assert!(
        tec >= min - 1E-9 && tec <= max + 1E-9,
        "{} not in [{}, {}] ({} {} {} {})",
        tec,
        min,
        max,
        a,
        b,
        c,
        d
    );

    // longitude wrapping on worldwide maps
    let wrapped = ionex.tec_at(noon, Point::new(361.44, 43.6)).unwrap().tecu();
    assert_eq!(wrapped, tec);

    // outside
    assert!(matches!(
        ionex.tec_at(noon, Point::new(0.0, 89.0)),
        Err(Error::OutsideSpatialBoundaries)
    ));

    // 2D map: altitude must match
    assert!(
        ionex
            .volumetric_tec_at(noon, Point::new(0.0, 0.0), 350.0, Default::default())
            .is_ok()
    );
    assert!(
        ionex
            .volumetric_tec_at(noon, Point::new(0.0, 0.0), 450.0, Default::default())
            .is_err()
    );
}

#[test]
fn ckmg_temporal_interpolation() {
    let ionex = ckmg();

    let t0 = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();
    let t1 = Epoch::from_str("2022-01-02T13:00:00 UTC").unwrap();
    let (lat, long) = (42.5, 0.0);
    let point = Point::new(long, lat);

    let (e0, e1) = (
        record_tecu(&ionex, t0, lat, long),
        record_tecu(&ionex, t1, lat, long),
    );

    // linear
    for (minutes, w) in [(15.0, 0.25), (30.0, 0.5), (45.0, 0.75)] {
        let t = t0 + Duration::from_seconds(minutes * 60.0);
        let tec = ionex
            .tec_at_with(t, point, TemporalInterpolation::Linear)
            .unwrap();
        assert!((tec.tecu() - ((1.0 - w) * e0 + w * e1)).abs() < 1E-9);
    }

    // nearest
    let t = t0 + Duration::from_seconds(20.0 * 60.0);
    let tec = ionex
        .tec_at_with(t, point, TemporalInterpolation::NearestMap)
        .unwrap();
    assert_eq!(tec.tecu(), e0);

    let t = t0 + Duration::from_seconds(40.0 * 60.0);
    let tec = ionex
        .tec_at_with(t, point, TemporalInterpolation::NearestMap)
        .unwrap();
    assert_eq!(tec.tecu(), e1);

    // rotated maps: at t0 + 20', the maps are rotated by +5° and -10° respectively,
    // which are grid nodes
    let t = t0 + Duration::from_seconds(20.0 * 60.0);
    let (r0, r1) = (
        record_tecu(&ionex, t0, lat, long + 5.0),
        record_tecu(&ionex, t1, lat, long - 10.0),
    );
    let w = 1.0 / 3.0;
    let tec = ionex
        .tec_at_with(t, point, TemporalInterpolation::RotatedMaps)
        .unwrap();
    assert!((tec.tecu() - ((1.0 - w) * r0 + w * r1)).abs() < 1E-9);

    // rotated maps across the antimeridian
    let point = Point::new(180.0, lat);
    let t = t0 + Duration::from_seconds(20.0 * 60.0);
    let (r0, r1) = (
        record_tecu(&ionex, t0, lat, -175.0),
        record_tecu(&ionex, t1, lat, 170.0),
    );
    let tec = ionex
        .tec_at_with(t, point, TemporalInterpolation::RotatedMaps)
        .unwrap();
    assert!((tec.tecu() - ((1.0 - w) * r0 + w * r1)).abs() < 1E-6);

    // continuity of the default strategy
    let mut prev = ionex.tec_at(t0, Point::new(1.44, 43.6)).unwrap().tecu();
    for seconds in (60..3600).step_by(60) {
        let t = t0 + Duration::from_seconds(seconds as f64);
        let tec = ionex.tec_at(t, Point::new(1.44, 43.6)).unwrap().tecu();
        assert!((tec - prev).abs() < 1.0, "discontinuity at {}", t);
        prev = tec;
    }

    // outside of temporal axis
    let t = Epoch::from_str("2022-01-01T23:59:59 UTC").unwrap();
    assert!(matches!(
        ionex.tec_at(t, Point::new(0.0, 0.0)),
        Err(Error::OutsideTemporalBoundaries)
    ));
}

#[test]
fn ckmg_roi_interpolation() {
    let ionex = ckmg();
    let t = Epoch::from_str("2022-01-02T12:30:00 UTC").unwrap();

    // unitary roi: grid quantum interpolated in time
    let cell = ionex.unitary_roi_at(t, Point::new(1.44, 43.6)).unwrap();
    assert_eq!(cell.epoch, t);
    assert_eq!(cell.south_west.point, Point::new(0.0, 42.5));

    let expected = ionex
        .tec_at_with(t, Point::new(1.44, 43.6), TemporalInterpolation::Linear)
        .unwrap();

    let tec = cell.spatial_tec_interp(Point::new(1.44, 43.6)).unwrap();
    assert!((tec.tecu() - expected.tecu()).abs() < 1E-9);
}

/// Synthetic 3D IONEX, where the TEC is f(lat, long, alt, t) = 1 + 0.1 long + 0.2 lat + 0.01 alt + t(h)
/// which is reproduced exactly by the (tri)linear interpolation.
fn synthetic_3d() -> (IONEX, impl Fn(f64, f64, f64, f64) -> f64) {
    let f = |lat: f64, long: f64, alt: f64, hours: f64| {
        1.0 + 0.1 * long + 0.2 * lat + 0.01 * alt + hours
    };

    let t0 = Epoch::from_str("2022-01-01T00:00:00 UTC").unwrap();

    let grid = Grid {
        latitude: Linspace::from((10.0, -10.0, -2.5)),
        longitude: Linspace::from((-20.0, 20.0, 5.0)),
        altitude: Linspace::from((100.0, 500.0, 100.0)),
    };

    let mut record = Record::default();

    for hours in [0.0, 1.0, 2.0] {
        let epoch = t0 + Duration::from_seconds(hours * 3600.0);

        for lat in grid.latitude.quantize() {
            for long in grid.longitude.quantize() {
                for alt in grid.altitude.quantize() {
                    let (lat, long, alt) = (lat.real_value(), long.real_value(), alt.real_value());

                    record.insert(
                        Key::from_decimal_degrees_km(epoch, lat, long, alt),
                        TEC::from_tecu(f(lat, long, alt, hours)),
                    );
                }
            }
        }
    }

    let mut header = Header::default();
    header.map_dimension = 3;
    header.grid = grid;
    header.epoch_of_first_map = t0;
    header.epoch_of_last_map = t0 + Duration::from_seconds(7200.0);

    (IONEX::new(header, record), f)
}

#[test]
fn synthetic_volumetric_interpolation() {
    let (ionex, f) = synthetic_3d();
    let t0 = Epoch::from_str("2022-01-01T00:00:00 UTC").unwrap();

    assert!(ionex.is_3d());

    // altitude must be specified
    assert!(matches!(
        ionex.tec_at(t0, Point::new(0.0, 0.0)),
        Err(Error::AltitudeRequired)
    ));

    for (lat, long, alt, hours) in [
        (0.0, 0.0, 100.0, 0.0),
        (1.3, -7.2, 350.0, 0.0),
        (-9.9, 19.9, 499.0, 0.5),
        (10.0, -20.0, 500.0, 1.25),
        (3.0, 4.0, 222.0, 2.0),
    ] {
        let t = t0 + Duration::from_seconds(hours * 3600.0);

        let tec = ionex
            .volumetric_tec_at(t, Point::new(long, lat), alt, TemporalInterpolation::Linear)
            .unwrap();

        let expected = f(lat, long, alt, hours);
        assert!(
            (tec.tecu() - expected).abs() < 1E-9,
            "(lat={}, long={}, alt={}, t={}h): {} instead of {}",
            lat,
            long,
            alt,
            hours,
            tec.tecu(),
            expected
        );
    }

    // outside of the volume
    assert!(
        ionex
            .volumetric_tec_at(t0, Point::new(0.0, 0.0), 600.0, Default::default())
            .is_err()
    );

    // regional map: rotation leaves the map near the borders
    let t = t0 + Duration::from_seconds(1800.0);
    assert!(
        ionex
            .volumetric_tec_at(
                t,
                Point::new(19.0, 0.0),
                200.0,
                TemporalInterpolation::RotatedMaps
            )
            .is_err()
    );
}

#[test]
fn jplg_rms_interpolation() {
    let ionex = IONEX::from_gzip_file("data/IONEX/V1/jplg0010.17i.gz").unwrap();
    let t0 = ionex.record.first_epoch().unwrap();
    let alt = ionex.header.grid.altitude.start;

    let rms = |lat: f64, long: f64| {
        let key = Key::from_decimal_degrees_km(t0, lat, long, alt);
        ionex.record.get(&key).unwrap().root_mean_square().unwrap()
    };

    // grid node
    let tec = ionex.tec_at(t0, Point::new(0.0, 42.5)).unwrap();
    assert_eq!(tec.root_mean_square(), Some(rms(42.5, 0.0)));

    // center of a cell
    let expected = (rms(42.5, 0.0) + rms(42.5, 5.0) + rms(45.0, 0.0) + rms(45.0, 5.0)) / 4.0;
    let tec = ionex.tec_at(t0, Point::new(2.5, 43.75)).unwrap();
    assert!((tec.root_mean_square().unwrap() - expected).abs() < 1E-9);
}

#[test]
fn synthetic_3d_formatting() {
    let (ionex, _) = synthetic_3d();

    // computed values (not parsed): the formatter must apply the header scaling
    ionex.to_file("synthetic-3d.txt").unwrap();
    let parsed = IONEX::from_file("synthetic-3d.txt").unwrap();
    let _ = std::fs::remove_file("synthetic-3d.txt");

    assert!(parsed.is_3d());
    assert_eq!(parsed.header.grid, ionex.header.grid);
    assert_eq!(parsed.record.iter().count(), ionex.record.iter().count());

    for (k, v) in ionex.record.iter() {
        let parsed = parsed.record.get(k).unwrap();
        // limited to the file precision (0.1 TECu)
        assert!((parsed.tecu() - v.tecu()).abs() <= 0.05 + 1E-9);
    }
}
