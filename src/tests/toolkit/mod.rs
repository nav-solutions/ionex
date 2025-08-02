use crate::prelude::{Header, IONEX};

// use rand::{distributions::Alphanumeric, Rng};

// /// Random name generator
// pub fn random_name(size: usize) -> String {
//     rand::thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(size)
//         .map(char::from)
//         .collect()
// }

/// Verifies two [Header]s are "strictly" identical
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
        } else {
            panic!("{:?} does not exist in model", dut_k);
        }
    }

    for (model_k, model_v) in model.record.iter() {
        if let Some(dut_v) = dut.record.get(&model_k) {
        } else {
            panic!("DUT is missing entry at {:?}", model_k);
        }
    }
}
