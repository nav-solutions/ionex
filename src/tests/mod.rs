//! integrated tests
pub mod toolkit;

mod antex;
mod compression;
mod filename;
pub mod formatting;
mod parsing;

#[cfg(feature = "flate2")]
mod production;

#[cfg(feature = "log")]
use log::LevelFilter;

#[cfg(feature = "log")]
use std::sync::Once;

#[cfg(feature = "log")]
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
