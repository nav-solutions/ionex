use crate::prelude::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [MappingFunction] used in the determination of the TEC map.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MappingFunction {
    /// No mapping function being used
    #[default]
    None,

    /// Model is 1/cos(z)
    CosZ,

    /// Qfactor
    QFactor,
}

impl std::str::FromStr for MappingFunction {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "QFAC" => Ok(Self::QFactor),
            "NONE" => Ok(Self::None),
            "COSZ" | "cosine" => Ok(Self::CosZ),
            _ => Err(ParsingError::MappingFunction),
        }
    }
}

impl std::fmt::Display for MappingFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CosZ => write!(f, "COSZ"),
            Self::QFactor => write!(f, "QFAC"),
            Self::None => write!(f, "NONE"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn mapping_function() {
        for (content, value) in [
            ("COSZ", MappingFunction::CosZ),
            ("QFAC", MappingFunction::QFactor),
            ("NONE", MappingFunction::None),
        ] {
            let parsed = MappingFunction::from_str(content).unwrap_or_else(|e| {
                panic!("Failed to parse mapf: \"{}\" - {}", content, e);
            });

            assert_eq!(parsed, value);

            let formatted = parsed.to_string();
            assert_eq!(formatted, content);
        }
    }
}
