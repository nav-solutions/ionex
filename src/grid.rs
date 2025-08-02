use crate::linspace::Linspace;

#[cfg(feature = "serde")]
use serde::Serialize;

/// [Grid] used to describe latitude, longitude
/// and altitude linar spaces, defining the entire map.
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Grid {
    /// Latitude [Linspace]
    pub latitude: Linspace,

    /// Longitude [Linspace]
    pub longitude: Linspace,

    /// Altitude [Linspace]
    pub altitude: Linspace,
}

impl Grid {
    /// Returns true if self is compatible with a 3D TEC map.
    pub fn is_3d_grid(&self) -> bool {
        !self.is_2d_grid()
    }

    /// Returns true if self is not compatible with a 3D TEC map.
    /// That means the altitude is a single point with null width.
    pub fn is_2d_grid(&self) -> bool {
        self.altitude.is_single_point()
    }

    /// Defines a new [Grid] with updated latitude space
    pub fn with_latitude_space(mut self, linspace: Linspace) -> Self {
        self.latitude = linspace;
        self
    }

    /// Defines a new [Grid] with updated longitude space
    pub fn with_longitude_space(mut self, linspace: Linspace) -> Self {
        self.longitude = linspace;
        self
    }

    /// Defines a new [Grid] with updated altitude space
    pub fn with_altitude_space(mut self, linspace: Linspace) -> Self {
        self.altitude = linspace;
        self
    }
}
