pub mod toolkit;
// pub mod formatting;

mod filename;
mod parsing;
mod qc;
mod roi;
mod stretching;

mod v1;

use log::LevelFilter;
use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(feature = "log")]
pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .init();
    });
}

// /// Verifies this IONEX is constant (TEC map) or panics otherwise.
// pub fn ionex_is_constant(ionex: &IONEX, constant_tecu: f64) {
//     for (k, v) in ionex.record.iter() {
//         assert_eq!(v.tecu(), constant_tecu);
//     }
// }
//
// /// Verifues this IONEX is null (TEC map only) or panics otherwise.
// pub fn ionex_is_null(ionex: &IONEX) {
//     ionex_is_constant(ionex, 0.0)
// }
