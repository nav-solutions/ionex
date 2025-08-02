//! integrated tests
pub mod toolkit;

mod filename;
pub mod formatting;
mod parsing;

use log::LevelFilter;
use std::sync::Once;

use crate::prelude::IONEX;

static INIT: Once = Once::new();

#[cfg(feature = "log")]
pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .init();
    });
}

/// Verifies this IONEX is constant (TEC map) or panics otherwise.
pub fn ionex_is_constant(ionex: &IONEX, constant: f64) {
    for (k, v) in ionex.record.iter() {}
}

/// Verifues this IONEX is null (TEC map only) or panics otherwise.
pub fn ionex_is_null(ionex: &IONEX) {
    ionex_is_constant(ionex, 0.0)
}
