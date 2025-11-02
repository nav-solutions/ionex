use thiserror::Error;

use gnss_rs::{
    constellation::ParsingError as ConstellationParsingError, sv::ParsingError as SVParsingError,
};

use std::io::Error as IoError;

/// Errors that may rise during parsing process.
#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("I/O input error: {0}")]
    IoError(#[from] IoError),

    #[error("header line too short (invalid)")]
    HeaderLineTooShort,

    #[error("empty epoch")]
    EmptyEpoch,

    #[error("faulty epoch description")]
    EpochDescriptionError,

    #[error("failed to parse map index from \"{0}\"")]
    MapIndexParsing(String),

    #[error("invalid grid definition")]
    InvalidGridDefinition,

    #[error("failed to parse number of maps")]
    NumberofMaps,

    #[error("failed to parse number of stations")]
    NumberofStations,

    #[error("failed to parse number of satellites")]
    NumberofSatellites,

    #[error("failed to parse elevation cutoff")]
    ElevationCutoff,

    #[error("failed to parse sampling period")]
    SamplingPeriod,

    #[error("error when parsing a coordinates")]
    CoordinatesParsing,

    #[error("invalid epoch format")]
    EpochFormat,

    #[error("epoch parsing")]
    EpochParsing,

    #[error("revision number parsing")]
    VersionParsing,

    #[error("constellation parsing error: {0}")]
    ConstellationParsing(#[from] ConstellationParsingError),

    #[error("satellite parsing error: {0}")]
    SVParsing(#[from] SVParsingError),

    #[error("header coordinates parsing")]
    Coordinates,

    #[error("invalid reference system")]
    ReferenceSystem,

    #[error("mapping function parsing error")]
    MappingFunction,

    #[error("datetime parsing error")]
    DatetimeParsing,

    #[error("map index parsing")]
    MapIndex,

    #[error("map grid specs parsing")]
    GridSpecs,

    #[error("error when parsing grid coordinates")]
    GridCoordinates,

    #[error("unknown earth observation satellite")]
    UnknownEarthObservationSat,

    #[error("unknown theoretical model")]
    UnknownTheoreticalModel,

    #[error("scaling parsing issue")]
    ExponentScaling,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("strech factor must be positive finite number")]
    InvalidStretchFactor,

    #[error("both regions do not describe the same spatial ROI")]
    SpatialMismatch,

    #[error("invalid temporal interpolation instant")]
    InvalidTemporalPoint,
}

/// Errors that may rise during Formatting process
#[derive(Error, Debug)]
pub enum FormattingError {
    #[error("I/O output error: {0}")]
    IoError(#[from] IoError),

    #[error("missing grid definition")]
    NoGridDefinition,
}
