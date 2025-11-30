use geo::{Contains, GeodesicArea, Geometry, Point, Rect};

use crate::prelude::{Epoch, TEC};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MapPoint {
    /// [Point]
    pub point: Point<f64>,

    /// TEC
    pub tec: TEC,
}

/// [MapCell] describing a region that we can then interpolate.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MapCell {
    /// Epoch of observation
    pub epoch: Epoch,

    /// North East [MapPoint]
    pub north_east: MapPoint,

    /// North West [MapPoint]
    pub north_west: MapPoint,

    /// South East [MapPoint]
    pub south_east: MapPoint,

    /// South West [MapPoint]
    pub south_west: MapPoint,
}

impl MapCell {
    /// Define a new [MapCell] from 4 (latitude_ddeg, longitude_ddeg) cardinal tuples and
    /// associated TEC values.
    pub fn from_lat_long_degrees(
        epoch: Epoch,
        northeast_ddeg: (f64, f64),
        northeast_tec: TEC,
        northwest_ddeg: (f64, f64),
        northwest_tec: TEC,
        southeast_ddeg: (f64, f64),
        southeast_tec: TEC,
        southwest_ddeg: (f64, f64),
        southwest_tec: TEC,
    ) -> Self {
        Self {
            epoch,
            north_east: MapPoint {
                point: Point::new(northeast_ddeg.0, northeast_ddeg.1),
                tec: northeast_tec,
            },
            north_west: MapPoint {
                point: Point::new(northwest_ddeg.0, northwest_ddeg.1),
                tec: northwest_tec,
            },
            south_east: MapPoint {
                point: Point::new(southeast_ddeg.0, southeast_ddeg.1),
                tec: southeast_tec,
            },
            south_west: MapPoint {
                point: Point::new(southwest_ddeg.0, southwest_ddeg.1),
                tec: southwest_tec,
            },
        }
    }

    /// Define a new [MapCell] from 4 (latitude_rad, longitude_rad) cardinal tuples and
    /// associated TEC values.
    pub fn from_lat_long_radians(
        epoch: Epoch,
        northeast_rad: (f64, f64),
        northeast_tec: TEC,
        northwest_rad: (f64, f64),
        northwest_tec: TEC,
        southeast_rad: (f64, f64),
        southeast_tec: TEC,
        southwest_rad: (f64, f64),
        southwest_tec: TEC,
    ) -> Self {
        Self {
            epoch,
            north_east: MapPoint {
                point: Point::new(northeast_rad.0.to_degrees(), northeast_rad.1.to_degrees()),
                tec: northeast_tec,
            },
            north_west: MapPoint {
                point: Point::new(northwest_rad.0.to_degrees(), northwest_rad.1.to_degrees()),
                tec: northwest_tec,
            },
            south_east: MapPoint {
                point: Point::new(southeast_rad.0.to_degrees(), southeast_rad.1.to_degrees()),
                tec: southeast_tec,
            },
            south_west: MapPoint {
                point: Point::new(southwest_rad.0.to_degrees(), southwest_rad.1.to_degrees()),
                tec: southwest_tec,
            },
        }
    }

    /// Define a new [MapCell] from all 4 [MapPoint]s describing each corner at this [Epoch].
    pub fn from_cardinal_points(
        epoch: Epoch,
        north_east: MapPoint,
        north_west: MapPoint,
        south_east: MapPoint,
        south_west: MapPoint,
    ) -> Self {
        Self {
            epoch,
            north_east,
            north_west,
            south_east,
            south_west,
        }
    }

    /// Defines a unitary [MapCell] ((0,0), (0,1), (1,0), (1,1)) with associated TEC values,
    /// where
    /// - (x/long=0, y/lat=0) is the SW corner
    /// - (x/long=1, y/lat=0) is the SE corner
    /// - (x/long=0, y/lat=1) is the NW corner
    /// - (x/long=1, y/lat=1) is the NE corner
    ///
    /// ```
    /// use ionex::prelude::{MapCell, Epoch, TEC};
    ///
    /// let epoch = Epoch::default();
    /// let tec = TEC::from_tecu(1.0);
    /// let cell = MapCell::from_unitary_tec(epoch, tec, tec, tec, tec);
    ///
    /// // 1.0° unitary ECEF span
    /// assert_eq!(cell.latitude_longitude_span_degrees(), (1.0, 1.0));
    ///
    /// // ECEF!
    /// assert!((cell.geodesic_perimeter() - 443770.0).abs() < 1.0);
    /// assert!((cell.geodesic_area() - 12308778361.0).abs() < 1.0);
    /// ```
    pub fn from_unitary_tec(
        epoch: Epoch,
        northeast_tec: TEC,
        northwest_tec: TEC,
        southeast_tec: TEC,
        southwest_tec: TEC,
    ) -> Self {
        Self::from_lat_long_degrees(
            epoch,
            (1.0, 1.0),
            northeast_tec,
            (0.0, 1.0),
            northwest_tec,
            (1.0, 0.0),
            southeast_tec,
            (0.0, 0.0),
            southwest_tec,
        )
    }

    /// Defines a unitary [MapCell] ((0,0), (0,1), (1,0), (1,1)) with Null TEC values,
    /// where
    /// - (x/long=0, y/lat=0) is the SW corner
    /// - (x/long=1, y/lat=0) is the SE corner
    /// - (x/long=0, y/lat=1) is the NW corner
    /// - (x/long=1, y/lat=1) is the NE corner
    ///
    /// ```
    /// use ionex::prelude::{MapCell, Epoch, TEC};
    ///
    /// let epoch = Epoch::default();
    /// let tec = TEC::from_tecu(1.0);
    /// let cell = MapCell::unitary_null_tec(epoch);
    ///
    /// // Null values!
    /// assert_eq!(cell.north_east.tec.tecu(), 0.0);
    /// assert_eq!(cell.north_west.tec.tecu(), 0.0);
    /// assert_eq!(cell.south_east.tec.tecu(), 0.0);
    /// assert_eq!(cell.south_west.tec.tecu(), 0.0);
    ///
    /// // 1.0° unitary ECEF span
    /// assert_eq!(cell.latitude_longitude_span_degrees(), (1.0, 1.0));
    ///
    /// // ECEF!
    /// assert!((cell.geodesic_perimeter() - 443770.0).abs() < 1.0);
    /// assert!((cell.geodesic_area() - 12308778361.0).abs() < 1.0);
    /// ```
    pub fn unitary_null_tec(epoch: Epoch) -> Self {
        let null_tec = TEC::default();
        Self::from_unitary_tec(epoch, null_tec, null_tec, null_tec, null_tec)
    }

    /// Returns central [Point] of this [MapCell].
    pub fn center(&self) -> Point<f64> {
        geo::Point(self.borders().center())
    }

    /// Returns borders of this [MapCell] expressed as a [Rect]angle.
    pub fn borders(&self) -> Rect {
        Rect::new(self.south_west.point, self.north_east.point)
    }

    /// Returns geodesic perimeter (in meters) of this [MapCell].
    pub fn geodesic_perimeter(&self) -> f64 {
        self.borders().geodesic_perimeter()
    }

    /// Returns geodesic area (in squared meters) of this [MapCell].
    pub fn geodesic_area(&self) -> f64 {
        self.borders().geodesic_area_unsigned()
    }

    /// Returns true if following [Geometry] is contained within this [MapCell].
    pub fn contains(&self, geometry: &Geometry<f64>) -> bool {
        self.borders().contains(geometry)
    }

    /// Copies and updates the Northeastern TEC component
    pub fn with_northeastern_tec(mut self, tec: TEC) -> Self {
        self.north_east.tec = tec;
        self
    }

    /// Copies and updates the Northwestern TEC component
    pub fn with_northwestern_tec(mut self, tec: TEC) -> Self {
        self.north_west.tec = tec;
        self
    }

    /// Copies and updates the Southeastern TEC component
    pub fn with_southeastern_tec(mut self, tec: TEC) -> Self {
        self.south_east.tec = tec;
        self
    }

    /// Copies and updates the Southwestern TEC component
    pub fn with_southwestern_tec(mut self, tec: TEC) -> Self {
        self.south_west.tec = tec;
        self
    }

    /// Returns the (latitude, longitude) span of this [MapCell]
    /// as tuplet in degrees
    pub fn latitude_longitude_span_degrees(&self) -> (f64, f64) {
        (self.latitude_span_degrees(), self.longitude_span_degrees())
    }

    /// Returns latitude span of this [MapCell] in degrees
    pub fn latitude_span_degrees(&self) -> f64 {
        let borders = self.borders();
        borders.max().y - borders.min().y
    }

    /// Returns longitude span of this [MapCell] in degrees
    pub fn longitude_span_degrees(&self) -> f64 {
        let borders = self.borders();
        borders.max().x - borders.min().x
    }

    /// Spatial interpolation of the [TEC] value using planery equation
    /// and 4 boundaries of this [MapCell]. [MapCell::contains] should be true
    /// for the proposed geometry for this to be correct.
    /// This method does not verify this assertion, it is up to you to use valid coordinates here.
    ///
    /// Example: unitary cell
    /// ```
    /// use ionex::prelude::{MapCell, Epoch, Point, TEC, Unit};
    ///
    /// // create unitary cell with simple values
    /// let t0 = Epoch::default();
    /// let one_tec = TEC::from_tecu(1.0);
    ///
    /// let cell = MapCell::from_unitary_tec(t0, one_tec, one_tec, one_tec, one_tec);
    ///
    /// // central point
    /// let center = Point::new(0.5, 0.5);
    /// let tec = cell.spatial_interpolation(center);
    /// assert_eq!(tec.tecu(), 1.0);
    /// ```
    ///
    /// Example: North East gradient
    /// ```
    /// use ionex::prelude::{MapCell, Epoch, Point, TEC, Unit};
    ///
    /// // create unitary cell with simple values
    /// let t0 = Epoch::default();
    ///
    /// let gradient = (
    ///     TEC::from_tecu(0.0),
    ///     TEC::from_tecu(0.0),
    ///     TEC::from_tecu(0.0),
    ///     TEC::from_tecu(1.0),
    /// );
    ///
    /// let cell = MapCell::from_unitary_tec(t0, gradient.0, gradient.1, gradient.2, gradient.3);
    ///
    /// // central point
    /// let tec = cell.spatial_interpolation(Point::new(0.5, 0.5));
    /// assert_eq!(tec.tecu(), 0.25);
    ///
    /// // SW boundary
    /// let tec = cell.spatial_interpolation(Point::new(0.0, 0.0));
    /// assert_eq!(tec.tecu(), 1.0);
    ///
    /// // SWern point
    /// let tec = cell.spatial_interpolation(Point::new(0.1, 0.1));
    /// assert_eq!(tec.tecu(), 0.81);
    ///
    /// // SWwern point
    /// let tec = cell.spatial_interpolation(Point::new(0.01, 0.01));
    /// assert_eq!(tec.tecu(), 0.9801);
    /// ```
    pub fn spatial_interpolation(&self, point: Point<f64>) -> TEC {
        let (latitude_span, longitude_span) = self.latitude_longitude_span_degrees();

        let (p, q) = (point.y() / latitude_span, point.x() / longitude_span);

        let (e00, e10, e01, e11) = (
            self.south_west.tec.tecu(),
            self.south_east.tec.tecu(),
            self.north_west.tec.tecu(),
            self.north_east.tec.tecu(),
        );

        let tecu =
            (1.0 - p) * (1.0 - q) * e00 + p * (1.0 - q) * e10 + q * (1.0 - p) * e01 + p * q * e11;

        TEC::from_tecu(tecu)
    }

    /// Spatial + Temporal Interpolation of [TEC] value using planery equation
    /// and rhs [MapCell], which should be closely sampled in time.
    /// [MapCell::contains] should be true both [MapCell]s and proposed geometry
    /// for the results to be correct, but this is not verified here: it is up to you
    /// to use valid coordinates here.
    /// Proposed [Epoch] should lie within both observation instants, otherwise this method
    /// returns None.
    ///
    /// ```
    /// use ionex::prelude::{MapCell, Epoch, Point, TEC, Unit};
    ///
    /// // create two unitary cells with simple values
    /// let t0 = Epoch::default();
    /// let t1 = t0 + 30.0 * Unit::Second;
    /// let t_ok = t0 + 15.0 * Unit::Second; // within interval
    /// let t_nok = t0 + 45.0 * Unit::Second; // outside interval
    ///
    /// let one_tec = TEC::from_tecu(1.0);
    ///
    /// let center = Point::new(0.5, 0.5); // unitary cell
    /// let cell0 = MapCell::from_unitary_tec(t0, one_tec, one_tec, one_tec, one_tec);
    /// let cell1 = MapCell::from_unitary_tec(t1, one_tec, one_tec, one_tec, one_tec);
    ///
    /// // verify central point value
    /// let central_tec0 = cell0.spatial_interpolation(center);
    /// assert_eq!(central_tec0.tecu(), 1.0);
    ///
    /// // verify central point value
    /// let central_tec1 = cell1.spatial_interpolation(center);
    /// assert_eq!(central_tec1.tecu(), 1.0);
    ///
    /// // spatial + temporal interpolation
    /// // <!> outside sampling interval
    /// assert!(cell0.temporal_spatial_interpolation(t_nok, center, &cell1).is_none());
    ///
    /// // spatial + temporal interpolation
    /// let tec = cell0.temporal_spatial_interpolation(t_ok, center, &cell1).unwrap();
    /// assert_eq!(tec.tecu(), 1.0);
    /// ```
    pub fn temporal_spatial_interpolation(
        &self,
        epoch: Epoch,
        point: Point<f64>,
        rhs: &Self,
    ) -> Option<TEC> {
        // interpolate at exact coordinates
        let (tecu_0, tecu_1) = (
            self.spatial_interpolation(point).tecu(),
            rhs.spatial_interpolation(point).tecu(),
        );

        if epoch >= self.epoch && epoch < rhs.epoch {
            // forward
            let dt = (rhs.epoch - self.epoch).to_seconds();

            let tecu = (rhs.epoch - epoch).to_seconds() / dt * tecu_0
                + (epoch - self.epoch).to_seconds() / dt * tecu_1;

            Some(TEC::from_tecu(tecu))
        } else if epoch >= rhs.epoch && epoch < self.epoch {
            // backwards
            let dt = (self.epoch - rhs.epoch).to_seconds();

            let tecu = (self.epoch - epoch).to_seconds() / dt * tecu_1
                + (epoch - rhs.epoch).to_seconds() / dt * tecu_0;

            Some(TEC::from_tecu(tecu))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::prelude::{Epoch, Geometry, Point, TEC, Unit};

    #[test]
    fn spatial_unitary_interpolation() {
        let epoch = Epoch::default();

        let northeast_tec = TEC::from_tecu(1.0);
        let northwest_tec = TEC::from_tecu(1.0);
        let southeast_tec = TEC::from_tecu(1.0);
        let southwest_tec = TEC::from_tecu(1.0);

        let cell = MapCell::from_unitary_tec(
            epoch,
            northeast_tec,
            northwest_tec,
            southeast_tec,
            southwest_tec,
        );

        assert_eq!(cell.latitude_longitude_span_degrees(), (1.0, 1.0));

        assert!((cell.geodesic_perimeter() - 443770.0).abs() < 1.0);
        assert!((cell.geodesic_area() - 12308778361.0).abs() < 1.0);

        let center = Point::new(0.5, 0.5);
        let outside = Point::new(1.5, 0.5);

        assert!(cell.contains(&Geometry::Point(center)));
        assert!(!cell.contains(&Geometry::Point(outside)));
        assert_eq!(center, cell.center());

        let interpolated = cell.spatial_interpolation(center);
        assert_eq!(interpolated.tecu(), 1.0);
    }

    #[test]
    fn spatial_south_west_gradient_interpolation() {
        let epoch = Epoch::default();

        let northeast_tec = TEC::from_tecu(0.0);
        let northwest_tec = TEC::from_tecu(0.0);
        let southeast_tec = TEC::from_tecu(0.0);
        let southwest_tec = TEC::from_tecu(1.0);

        let cell = MapCell::from_unitary_tec(
            epoch,
            northeast_tec,
            northwest_tec,
            southeast_tec,
            southwest_tec,
        );

        for (x_deg, y_deg, tecu) in [
            (0.5, 0.5, 0.25),
            (0.1, 0.1, 0.81),
            (0.01, 0.01, 0.9801),
            (0.0, 0.0, 1.0),
        ] {
            let point = Point::new(x_deg, y_deg);
            let interpolated = cell.spatial_interpolation(point).tecu();
            assert_eq!(interpolated, tecu, "failed at (x={}, y={})", x_deg, y_deg);
        }
    }

    #[test]
    fn temporal_interpolation() {
        let t0 = Epoch::default();
        let t1 = t0 + 1.0 * Unit::Second;

        let t_ok = t0 + 0.5 * Unit::Second;
        let t_nok = t1 + 2.0 * Unit::Second;

        let center = Point::new(0.5, 0.5);

        let northeast_tec_0 = TEC::from_tecu(1.0);
        let northwest_tec_0 = TEC::from_tecu(1.0);
        let southeast_tec_0 = TEC::from_tecu(1.0);
        let southwest_tec_0 = TEC::from_tecu(1.0);

        let cell0 = MapCell::from_unitary_tec(
            t0,
            northeast_tec_0,
            northwest_tec_0,
            southeast_tec_0,
            southwest_tec_0,
        );

        let northeast_tec_1 = TEC::from_tecu(1.0);
        let northwest_tec_1 = TEC::from_tecu(1.0);
        let southeast_tec_1 = TEC::from_tecu(1.0);
        let southwest_tec_1 = TEC::from_tecu(1.0);

        let cell1 = MapCell::from_unitary_tec(
            t1,
            northeast_tec_1,
            northwest_tec_1,
            southeast_tec_1,
            southwest_tec_1,
        );

        assert!(
            cell0
                .temporal_spatial_interpolation(t_nok, center, &cell1)
                .is_none(),
            "interpolation is temporally incorrect!"
        );

        let tec = cell0
            .temporal_spatial_interpolation(t_ok, center, &cell1)
            .unwrap();

        assert_eq!(tec.tecu(), 1.0);
    }
}
