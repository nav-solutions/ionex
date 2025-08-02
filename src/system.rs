use std::str::FromStr;

use crate::prelude::{Constellation, ParsingError};

#[cfg(doc)]
use crate::prelude::Header;

/// [ReferenceSystem] describes either reference constellations
/// or theoretical models used in this evaluation.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReferenceSystem {
    /// Reference Constellation.
    /// When `Mixed` this generally means GPS + Glonass.
    /// When GNSS constellation was used, TEC maps
    /// include electron content through the ionosphere
    /// and plasmasphere, up to altitude 20000 km.
    Constellation(Constellation),

    /// Evaluated using [OtherSystem].
    Other(OtherSystem),

    /// [TheoreticalModel].
    /// When a theoretical model is used,
    /// its parameters are given in the [Header] section.
    Model(TheoreticalModel),
}

/// [OtherSystem] that may serve the TEC map evaluation process.
#[derive(Default, Copy, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OtherSystem {
    /// BENt
    BENt,

    /// ENVisat is an ESA Earth Observation satellite
    #[default]
    ENVisat,

    /// European Remote Sensing Satellite (ESA).
    /// ERS-1 or ERS-2 were Earth observation satellites.
    /// Now replaced by ENVisat.
    ERS,

    /// IRI: Earth Observation Application group
    IRI,
}

impl std::str::FromStr for OtherSystem {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ben" => Ok(Self::BENt),
            "env" => Ok(Self::ENVisat),
            "ers" => Ok(Self::ERS),
            "iri" => Ok(Self::IRI),
            _ => Err(ParsingError::UnknownEarthObservationSat),
        }
    }
}

impl std::fmt::Display for OtherSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

/// Map resulting of a theoretical model.
#[derive(Default, Copy, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TheoreticalModel {
    /// Mixed / combined models.
    #[default]
    MIX,

    /// NNS transit
    NNS,

    /// TOP means TOPex.
    /// TOPex/TEC represents the ionosphere electron content
    /// measured over sea surface at altitudes below
    /// satellite orbits (1336 km).
    TOP,
}

impl std::str::FromStr for TheoreticalModel {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mix" => Ok(Self::MIX),
            "nns" => Ok(Self::NNS),
            "top" => Ok(Self::TOP),
            _ => Err(ParsingError::UnknownTheoreticalModel),
        }
    }
}

impl std::fmt::Display for TheroreicalModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl Default for RefSystem {
    fn default() -> Self {
        Self::Constellation(Constellation::default())
    }
}

impl std::fmt::Display for RefSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Constellation(c) => c.fmt(f),
            Self::OtherSystem(s) => s.fmt(f),
            Self::TheoreticalModel(m) => m.fmt(f),
        }
    }
}

impl FromStr for RefSystem {
    type Err = ParsingError;
    fn from_str(system: &str) -> Result<Self, Self::Err> {
        if let Ok(gnss) = Constellation::from_str(system) {
            Ok(Self::Constellation(gnss))
        } else if system.eq("GNSS") {
            Ok(Self::Constellation(Constellation::Mixed))
        } else if let Ok(other) = OtherSystem::from_str(system) {
            Ok(Self::OtherSystem(other))
        } else if let Ok(m) = TheoreticalModel::from_str(system) {
            Ok(Self::Model(m))
        } else {
            Err(ParsingError::ReferenceSystem)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_refsystem() {
        assert_eq!(
            ReferenceSystem::default(),
            ReferenceSystem::Constellation(Default::default())
        );
    }
}
