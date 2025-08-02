use crate::prelude::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [MappingFunction] used in the determination of the TEC map.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MappingFunction {
    /// cos-1(z)
    CosZ,

    /// Qfactor
    QFactor,
}

impl std::str::FromStr for MappingFunction {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "q" => Ok(Self::QFactor),
            "cos" | "cosine" => Ok(Self::CosZ),
            _ => Err(ParsingError::IonexMappingFunction),
        }
    }
}

impl std::fmt::Display for MappingFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CosZ => write!(f, "Cos-1(z)"),
            Self::QFactor => write!(f, "Q-factor"),
        }
    }
}
