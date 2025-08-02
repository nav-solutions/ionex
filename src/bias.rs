use crate::prelude::SV;

/// Possible DCB source.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BiasSource {
    /// Referenced to a satellite vehicle ([SV])
    Satellite(SV),

    /// Referenced to a ground station
    Station(String),
}

impl BiasSource {
    pub fn as_satellit(&self) -> Option<SV> {
        match self {
            Self::Satellite(sv) => Some(*sv),
            _ => None,
        }
    }

    pub fn as_ground_station(&self) -> Option<String> {
        match self {
            Self::Station(station) => Some(station.to_string()),
            _ => None,
        }
    }
}
