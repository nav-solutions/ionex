use crate::{coordinates::QuantizedCoordinates, prelude::Epoch};

/// [Key] allows efficient IONEX data storage.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    /// [Epoch] of the attached TEC estimation.
    pub epoch: Epoch,

    /// [QuantizedCoordinates] of the attached TEC estimate.
    pub(crate) coordinates: QuantizedCoordinates,
}

impl Key {
    /// Creates a new index [Key] from datetime as [Epoch],
    /// latitude and longitude in decimal degrees and altitude
    /// in kilometers.
    pub fn from_decimal_degrees_km(
        epoch: Epoch,
        lat_ddeg: f64,
        long_ddeg: f64,
        alt_km: f64,
    ) -> Self {
        Self {
            epoch,
            coordinates: QuantizedCoordinates::from_decimal_degrees(lat_ddeg, long_ddeg, alt_km),
        }
    }

    /// Creates a new index [Key] from datetime as [Epoch],
    /// latitude and longitude angles in radians, altitude in kilometers.
    pub fn from_radians_km(epoch: Epoch, lat_rad: f64, long_rad: f64, alt_km: f64) -> Self {
        Self {
            epoch,
            coordinates: QuantizedCoordinates::from_decimal_degrees(
                lat_rad.to_degrees(),
                long_rad.to_degrees(),
                alt_km,
            ),
        }
    }

    /// Returns latitude angle in decimal degrees
    pub fn latitude_ddeg(&self) -> f64 {
        self.coordinates.latitude_ddeg()
    }

    /// Returns longitude angle in decimal degrees
    pub fn longitude_ddeg(&self) -> f64 {
        self.coordinates.longitude_ddeg()
    }

    /// Returns altitude in kilometers
    pub fn altitude_km(&self) -> f64 {
        self.coordinates.altitude_km()
    }
}
