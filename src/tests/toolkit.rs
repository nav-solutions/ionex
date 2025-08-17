use log::info;
use std::str::FromStr;

use crate::{
    coordinates::QuantizedCoordinates,
    prelude::{Epoch, Header, Key, IONEX, TEC},
    quantized::Quantized,
};

/// Verifies two [Header]s are strictly identical
pub fn generic_header_comparison(dut: &Header, model: &Header) {
    assert_eq!(dut.version, model.version);
    assert_eq!(dut.comments, model.comments);
    assert_eq!(dut.program, model.program);
    assert_eq!(dut.run_by, model.run_by);
    assert_eq!(dut.license, model.license);
    assert_eq!(dut.doi, model.doi);
    assert_eq!(dut.date, model.date);
    assert_eq!(dut.description, model.description);
    assert_eq!(dut.sampling_period, model.sampling_period);
    assert_eq!(dut.epoch_of_first_map, model.epoch_of_first_map);
    assert_eq!(dut.epoch_of_last_map, model.epoch_of_last_map);
    assert_eq!(dut.reference_system, model.reference_system);
    assert_eq!(dut.base_radius_km, model.base_radius_km);
    assert_eq!(dut.elevation_cutoff, model.elevation_cutoff);
    assert_eq!(dut.grid, model.grid);
}

/// Verifies two [IONEX] files are strictly identical,
/// otherwise panics.
pub fn generic_comparison(dut: &IONEX, model: &IONEX) {
    generic_header_comparison(&dut.header, &model.header);

    assert_eq!(dut.comments, model.comments);

    for (dut_k, dut_v) in dut.record.iter() {
        if let Some(model_v) = model.record.get(&dut_k) {
            assert_eq!(dut_v, model_v);
        } else {
            panic!("{:?} does not exist in model", dut_k);
        }
    }

    for (model_k, model_v) in model.record.iter() {
        if let Some(dut_v) = dut.record.get(&model_k) {
            assert_eq!(dut_v, model_v);
        } else {
            panic!("DUT is missing entry at {:?}", model_k);
        }
    }
}

/// Helper to test one value at one coordinates
pub struct TestPoint<'a> {
    pub epoch_str: &'a str,
    pub lat_ddeg: f64,
    pub long_ddeg: f64,
    pub alt_km: f64,
    pub tecu: f64,
    pub rms: Option<f64>,
}

impl<'a> TestPoint<'a> {
    pub fn quantize(&self) -> QuantizedCoordinates {
        QuantizedCoordinates::from_decimal_degrees(self.lat_ddeg, self.long_ddeg, self.alt_km)
    }
}

/// Verifies all test points in a [IONEX].
pub fn generic_test(
    dut: &IONEX,
    test_points: &Vec<TestPoint>,
    angles_err_deg: f64,
    alt_err_m: f64,
) {
    let mut total_tests = 0;

    for test_point in test_points.iter() {
        let mut found = false;
        let mut tec = TEC::default();
        let epoch = Epoch::from_str(test_point.epoch_str).unwrap();

        let coordinates = test_point.quantize();
        let key = Key { epoch, coordinates };

        for (k, v) in dut.record.iter() {
            let latitude_err = (k.latitude_ddeg() - test_point.lat_ddeg).abs();
            let longitude_err = (k.longitude_ddeg() - test_point.long_ddeg).abs();
            let alt_err = (k.altitude_km() - test_point.alt_km).abs() * 1.0E3;

            if latitude_err < angles_err_deg
                && longitude_err < angles_err_deg
                && alt_err < alt_err_m
            {
                if k.altitude_km() == test_point.alt_km {
                    tec = *v;
                    found = true;
                }
            }
        }

        if !found {
            panic!(
                "missing data @{} (lat={}°, long={}°, z={}km)",
                test_point.epoch_str, test_point.lat_ddeg, test_point.long_ddeg, test_point.alt_km,
            );
        }

        let tecu = tec.tecu();
        let error = (tecu - test_point.tecu).abs();

        assert!(
            error < 1.0E-5,
            "({} lat={}° long={}° alt={}km) - invalid TECu: {} but {} is expected",
            test_point.epoch_str,
            test_point.lat_ddeg,
            test_point.long_ddeg,
            test_point.alt_km,
            tecu,
            test_point.tecu
        );

        total_tests += 1;
    }

    info!("tested {} coordinates", total_tests);
}

#[test]
fn test_helpers() {
    for (lat_ddeg, long_ddeg, alt_km) in [
        (87.5, -180.0, 350.0),
        (87.5, 180.0, 350.0),
        (87.5, 180.5, 300.0),
        (-10.5, 180.5, 300.0),
        (-30.5, 180.5, 300.0),
        (10.5, 150.5, 300.0),
        (30.5, 151.5, 300.0),
    ] {
        let testpoint = TestPoint {
            epoch_str: "2022-01-02T00:00:00 UTC",
            lat_ddeg,
            long_ddeg,
            alt_km,
            tecu: 92.0,
            rms: None,
        };

        let quantized = testpoint.quantize();

        assert_eq!(quantized.latitude_ddeg(), lat_ddeg);
        assert_eq!(quantized.longitude_ddeg(), long_ddeg);
        assert_eq!(quantized.altitude_km(), alt_km);
    }
}
