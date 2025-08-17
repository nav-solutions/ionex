use crate::quantized::Quantized;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [QuantizedCoordinates] used in map discretization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QuantizedCoordinates {
    /// Quantized latitude
    lat_ddeg: Quantized,

    /// Quantized longitude
    long_ddeg: Quantized,

    /// Quantized altitude
    alt_km: Quantized,
}

impl QuantizedCoordinates {
    /// Builds new [QuantizedCoordinates] from coordinates in decimal degrees
    /// and altitude in kilometers.
    pub fn from_decimal_degrees(lat: f64, long: f64, alt_km: f64) -> Self {
        Self {
            lat_ddeg: Quantized::auto_scaled(lat),
            long_ddeg: Quantized::auto_scaled(long),
            alt_km: Quantized::auto_scaled(alt_km),
        }
    }

    /// Builds new [QuantizedCoordinates] from coordinates in decimal degrees,
    /// altitude in kilometers, and using desired quantization scaling.
    #[cfg(test)]
    pub fn new(
        lat_ddeg: f64,
        lat_exponent: i8,
        long_ddeg: f64,
        long_exponent: i8,
        alt_km: f64,
        alt_exponent: i8,
    ) -> Self {
        Self {
            lat_ddeg: Quantized::new(lat_ddeg, lat_exponent),
            long_ddeg: Quantized::new(long_ddeg, long_exponent),
            alt_km: Quantized::new(alt_km, alt_exponent),
        }
    }

    /// Builds new [QuantizedCoordinates] from [Quantized] coordinates
    pub(crate) fn from_quantized(
        lat_ddeg: Quantized,
        long_ddeg: Quantized,
        alt_km: Quantized,
    ) -> Self {
        Self {
            lat_ddeg,
            long_ddeg,
            alt_km,
        }
    }

    /// Returns latitude in degrees
    pub fn latitude_ddeg(&self) -> f64 {
        self.lat_ddeg.real_value()
    }

    // /// Returns latitude in radians
    // pub fn latitude_rad(&self) -> f64 {
    //     self.latitude_ddeg().to_radians()
    // }

    /// Returns longitude in degrees
    pub fn longitude_ddeg(&self) -> f64 {
        self.long_ddeg.real_value()
    }

    // /// Returns longitude in radians
    // pub fn longitude_rad(&self) -> f64 {
    //     self.longitude_ddeg().to_radians()
    // }

    /// Returns longitude in kilometers
    pub fn altitude_km(&self) -> f64 {
        self.alt_km.real_value()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quantized_coords() {
        let coords = QuantizedCoordinates::new(1.0, 1, 2.0, 1, 3.0, 1);
        assert_eq!(coords.latitude_ddeg(), 1.0);
        assert_eq!(coords.longitude_ddeg(), 2.0);
        assert_eq!(coords.altitude_km(), 3.0);

        let coords = QuantizedCoordinates::new(1.5, 1, 2.0, 1, 3.12, 2);
        assert_eq!(coords.latitude_ddeg(), 1.5);
        assert_eq!(coords.longitude_ddeg(), 2.0);
        assert_eq!(coords.altitude_km(), 3.12);
    }
}
