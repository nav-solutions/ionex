use crate::{error::ParsingError, linspace::Linspace};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

/// [GridSpecs] as found in IONEX descriptor
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub(crate) struct GridSpecs {
    /// Latitude (in decimal degrees) of following longitude segment
    pub latitude_ddeg: f64,

    /// Altitude (in km) of following longitude segment
    pub altitude_km: f64,

    /// Longitude [Linspace]
    pub longitude_space: Linspace,
}

impl std::str::FromStr for GridSpecs {
    type Err = ParsingError;

    /// Parses [GridSpecs] from standardized line
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.len() != 60 {
            return Err(ParsingError::InvalidGridDefinition);
        }

        let (latitude_ddeg, rem) = line[2..].split_at(6);

        let latitude_ddeg = latitude_ddeg
            .trim()
            .parse::<f64>()
            .map_err(|_| ParsingError::GridCoordinates)?;

        let (longitude_start_ddeg, rem) = rem.split_at(6);

        let longitude_start_ddeg = longitude_start_ddeg
            .trim()
            .parse::<f64>()
            .map_err(|_| ParsingError::GridCoordinates)?;

        let (longitude_end_ddeg, rem) = rem.split_at(6);

        let longitude_end_ddeg = longitude_end_ddeg
            .trim()
            .parse::<f64>()
            .map_err(|_| ParsingError::GridCoordinates)?;

        let (longitude_spacing_ddeg, rem) = rem.split_at(6);

        let longitude_spacing_ddeg = longitude_spacing_ddeg
            .trim()
            .parse::<f64>()
            .map_err(|_| ParsingError::GridCoordinates)?;

        let (altitude_km, _) = rem.split_at(6);

        let altitude_km = altitude_km
            .trim()
            .parse::<f64>()
            .map_err(|_| ParsingError::GridCoordinates)?;

        let longitude_space = Linspace::new(
            longitude_start_ddeg,
            longitude_end_ddeg,
            longitude_spacing_ddeg,
        )?;

        Ok(Self {
            latitude_ddeg,
            altitude_km,
            longitude_space,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn grid_specs_parsing() {
        for (lat_ddeg, long1_ddeg, long2_ddeg, dlon_ddeg, alt_km, content) in [
            (
                87.5,
                -180.0,
                180.0,
                5.0,
                450.0,
                "     2.5-180.0 180.0   5.0 350.0                            LAT/LON1/LON2/DLON/H",
            ),
            (
                87.5,
                -180.0,
                180.0,
                5.0,
                450.0,
                "    87.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H",
            ),
            (
                -2.5,
                -180.0,
                180.0,
                5.0,
                250.0,
                "    -2.5-180.0 180.0   5.0 250.0                            LAT/LON1/LON2/DLON/H",
            ),
        ] {
            let grid_specs = GridSpecs::from_str(content).unwrap_or_else(|e| {
                panic!("Failed to parse grid specs from \"{}\": {}", content, e);
            });

            assert_eq!(grid_specs.latitude_ddeg, lat_ddeg);
            assert_eq!(grid_specs.altitude_km, alt_km);
            assert_eq!(grid_specs.longitude_space.start, long1_ddeg);
            assert_eq!(grid_specs.longitude_space.end, long2_ddeg);
            assert_eq!(grid_specs.longitude_space.spacing, dlon_ddeg);
        }
    }
}
