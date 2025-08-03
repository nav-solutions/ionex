use std::str::FromStr;

use crate::prelude::{Epoch, Header, Key, Quantized, QuantizedCoordinates, IONEX};

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

/// Verifies that all data points do not have RMS value (otherwise panics)
pub fn check_no_root_mean_square(dut: &IONEX) {
    for (k, v) in dut.record.iter() {
        assert!(
            v.root_mean_square().is_none(),
            "unexpected Root Mean Square at {}(lat={},long={},z={})",
            k.epoch,
            k.coordinates.latitude_ddeg(),
            k.coordinates.longitude_ddeg(),
            k.coordinates.altitude_km(),
        );
    }
}

/// Verifies all test points in a [IONEX].
pub fn generic_test(dut: &IONEX, test_points: Vec<TestPoint>) {
    for test_point in test_points.iter() {
        let epoch = Epoch::from_str(test_point.epoch_str).unwrap();

        let coordinates = QuantizedCoordinates::from_quantized(
            Quantized::new_auto_scaled(test_point.lat_ddeg),
            Quantized::new_auto_scaled(test_point.long_ddeg),
            Quantized::new_auto_scaled(test_point.alt_km),
        );

        let key = Key { epoch, coordinates };

        let tec = dut.record.get(&key).expect(&format!(
            "missing data @{} (lat={};long={};z={})",
            test_point.epoch_str, test_point.lat_ddeg, test_point.long_ddeg, test_point.alt_km
        ));

        let tecu = tec.tecu();

        let error = (tecu - test_point.tecu).abs();

        assert!(
            error < 1.0E-5,
            "invalid tec value: {} but {} is expected",
            tecu,
            test_point.tecu
        );
    }
}
