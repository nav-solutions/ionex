use thiserror::Error;

use gnss_rs::{
    constellation::ParsingError as ConstellationParsingError, cospar::Error as CosparParsingError,
    domes::Error as DOMESParsingError, sv::ParsingError as SVParsingError,
};

use hifitime::{HifitimeError, ParsingError as HifitimeParsingError};

use std::io::Error as IoError;

/// Errors that may rise during the parsing process.
#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("header line too short (invalid)")]
    HeaderLineTooShort,

    #[error("empty epoch")]
    EmptyEpoch,

    #[error("faulty epoch description")]
    EpochDescriptionError,

    #[error("failed to parse map index from \"{0}\"")]
    MapIndexParsing(String),

    #[error("bad grid definition")]
    BadGridDefinition(#[from] crate::linspace::Error),

    #[error("failed to parse {0} coordinates from \"{1}\"")]
    CoordinatesParsing(String, String),

    #[error("number of sat")]
    NumSat,

    #[error("invalid epoch format")]
    EpochFormat,

    #[error("epoch parsing")]
    EpochParsing,

    #[error("ionex revision parsing")]
    VersionParsing,

    #[error("constellation parsing")]
    ConstellationParsing(#[from] ConstellationParsingError),

    #[error("sv parsing")]
    SVParsing(#[from] SVParsingError),

    #[error("header coordinates parsing")]
    Coordinates,

    #[error("invalid reference system")]
    ReferenceSystem,

    #[error("mapping function parsing error")]
    MappingFunction,

    #[error("hifitime parsing")]
    HifitimeParsing(#[from] HifitimeParsingError),

    #[error("map index parsing")]
    MapIndex,

    #[error("map grid specs parsing")]
    GridSpecs,

    #[error("invalid map grid specs")]
    BadGridSpecs,

    #[error("error when parsing grid coordinates")]
    GridCoordinates,

    #[error("earth observation satellite")]
    EarthObservationSat,

    #[error("model parsing issue")]
    ModelParsing,

    #[error("scaling parsing issue")]
    ExponentScaling,
}

/// Errors that may rise in Formatting process
#[derive(Error, Debug)]
pub enum FormattingError {
    #[error("i/o: output error")]
    OutputError(#[from] IoError),

    #[error("missing grid definition")]
    NoGridDefinition,
}
