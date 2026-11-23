//! Ionospheric Pierce Point (IPP) and single layer model.

use geo::Point;

use crate::error::Error;

/// Ionospheric Pierce Point ([IPP]): the location where the line of sight
/// between a receiver and a satellite pierces the ionosphere, modeled as a thin shell
/// (single layer model) at a fixed altitude above a spherical Earth.
/// This is the model used by 2D IONEX maps: the shell altitude is the fixed
/// altitude of the map, and the Earth radius is the base radius of the [crate::prelude::Header].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IPP {
    /// Coordinates of the pierce point on the shell
    /// (x: longitude, y: latitude in decimal degrees)
    pub coordinates: Point<f64>,

    /// Altitude of the shell, in kilometers
    pub altitude_km: f64,

    /// Zenith angle of the line of sight at the pierce point, in radians
    pub zenith_angle_rad: f64,
}

impl IPP {
    /// Determines the [IPP] from the receiver position (x: longitude, y: latitude in decimal
    /// degrees), the elevation and azimuth of the line of sight (in degrees), using a spherical
    /// Earth of given radius and a shell at given altitude (both in kilometers).
    /// The receiver altitude is neglected.
    ///
    /// Returns [Error::InvalidElevation] for negative elevations.
    ///
    /// ```
    /// use ionex::prelude::{IPP, Point};
    ///
    /// // receiver at (0°, 0°), satellite at zenith
    /// let ipp = IPP::new(Point::new(0.0, 0.0), 90.0, 0.0, 6371.0, 350.0).unwrap();
    /// assert!(ipp.coordinates.x().abs() < 1E-9);
    /// assert!(ipp.coordinates.y().abs() < 1E-9);
    /// assert!((ipp.obliquity_factor() - 1.0).abs() < 1E-9);
    ///
    /// // 30° elevation, looking North
    /// let ipp = IPP::new(Point::new(0.0, 0.0), 30.0, 0.0, 6371.0, 350.0).unwrap();
    /// assert!(ipp.coordinates.y() > 4.8 && ipp.coordinates.y() < 4.9);
    /// assert!((ipp.obliquity_factor() - 1.7513).abs() < 1E-3);
    /// ```
    pub fn new(
        receiver: Point<f64>,
        elevation_deg: f64,
        azimuth_deg: f64,
        earth_radius_km: f64,
        shell_altitude_km: f64,
    ) -> Result<Self, Error> {
        if !(0.0..=90.0).contains(&elevation_deg) {
            return Err(Error::InvalidElevation);
        }

        let (elevation, azimuth) = (elevation_deg.to_radians(), azimuth_deg.to_radians());
        let (lat, long) = (receiver.y().to_radians(), receiver.x().to_radians());

        // zenith angle at the pierce point
        let sin_z = earth_radius_km / (earth_radius_km + shell_altitude_km) * elevation.cos();
        let zenith_angle_rad = sin_z.asin();

        // Earth central angle between the receiver and the pierce point
        let psi = std::f64::consts::FRAC_PI_2 - elevation - zenith_angle_rad;

        // destination point along the great circle
        let ipp_lat = (lat.sin() * psi.cos() + lat.cos() * psi.sin() * azimuth.cos()).asin();

        let ipp_long = long
            + (azimuth.sin() * psi.sin() * lat.cos()).atan2(psi.cos() - lat.sin() * ipp_lat.sin());

        // wrap to [-180, 180[
        let ipp_long = (ipp_long.to_degrees() + 180.0).rem_euclid(360.0) - 180.0;

        Ok(Self {
            coordinates: Point::new(ipp_long, ipp_lat.to_degrees()),
            altitude_km: shell_altitude_km,
            zenith_angle_rad,
        })
    }

    /// Returns the obliquity factor (also called slant factor, or single layer
    /// mapping function) F = 1 / cos(z'), that converts vertical TEC at this [IPP]
    /// to slant TEC along the line of sight: STEC = F * VTEC.
    pub fn obliquity_factor(&self) -> f64 {
        1.0 / self.zenith_angle_rad.cos()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pierce_points() {
        let (radius, altitude) = (6371.0, 350.0);
        let origin = Point::new(0.0, 0.0);

        // symmetry: North / South / East / West at the equator
        let north = IPP::new(origin, 30.0, 0.0, radius, altitude).unwrap();
        let south = IPP::new(origin, 30.0, 180.0, radius, altitude).unwrap();
        let east = IPP::new(origin, 30.0, 90.0, radius, altitude).unwrap();
        let west = IPP::new(origin, 30.0, 270.0, radius, altitude).unwrap();

        let psi = north.coordinates.y();
        assert!(psi > 4.8 && psi < 4.9, "{}", psi);
        assert!(north.coordinates.x().abs() < 1E-9);

        assert!((south.coordinates.y() + psi).abs() < 1E-9);
        assert!((east.coordinates.x() - psi).abs() < 1E-9);
        assert!((west.coordinates.x() + psi).abs() < 1E-9);
        assert!(east.coordinates.y().abs() < 1E-9);

        // same obliquity factor for all azimuths
        for ipp in [south, east, west] {
            assert!((ipp.obliquity_factor() - north.obliquity_factor()).abs() < 1E-12);
        }

        // obliquity factor increases as elevation decreases
        let mut prev = 1.0;
        for elevation in [80.0, 60.0, 40.0, 20.0, 10.0, 5.0, 0.0] {
            let ipp = IPP::new(origin, elevation, 0.0, radius, altitude).unwrap();
            assert!(ipp.obliquity_factor() > prev);
            prev = ipp.obliquity_factor();
        }

        // horizon (350 km shell): 1/sqrt(1 - (6371/6721)²) = 3.14
        assert!((prev - 3.1398).abs() < 1E-4, "{}", prev);

        // antimeridian wrapping
        let ipp = IPP::new(Point::new(179.0, 0.0), 20.0, 90.0, radius, altitude).unwrap();
        assert!(ipp.coordinates.x() < -170.0, "{:?}", ipp.coordinates);

        assert!(IPP::new(origin, -1.0, 0.0, radius, altitude).is_err());
        assert!(IPP::new(origin, 91.0, 0.0, radius, altitude).is_err());
    }
}
