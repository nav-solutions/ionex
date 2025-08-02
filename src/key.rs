use crate::prelude::{Epoch, QuantizedCoordinates};

/// [Key] allows efficient IONEX data storage.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    /// [Epoch] of the attached TEC estimation.
    pub epoch: Epoch,

    /// [QuantizedCoordinates] of the attached TEC estimate.
    pub coordinates: QuantizedCoordinates,
}
