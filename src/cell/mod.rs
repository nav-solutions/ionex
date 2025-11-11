use std::collections::HashMap;

use geo::{Contains, GeodesicArea, Geometry, Point, Rect};

use crate::{
    prelude::{Epoch, Error, TEC},
    rectangle_to_cardinals,
};

mod three_by_three;
pub use three_by_three::Cell3x3;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TecPoint {
    /// TEC
    pub tec: TEC,

    /// [Point]
    pub point: Point<f64>,
}

/// [MapCell] describing a 4 corner region that we can interpolate.
/// In the processing workflow, [MapCell]s are constructed from individual
/// quanta (smallest ROI) described in a IONEX map.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MapCell {
    /// Epoch of observation
    pub epoch: Epoch,

    /// North East [TecPoint]
    pub north_east: TecPoint,

    /// North West [TecPoint]
    pub north_west: TecPoint,

    /// South East [TecPoint]
    pub south_east: TecPoint,

    /// South West [TecPoint]
    pub south_west: TecPoint,
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
            north_east: TecPoint {
                point: Point::new(northeast_ddeg.0, northeast_ddeg.1),
                tec: northeast_tec,
            },
            north_west: TecPoint {
                point: Point::new(northwest_ddeg.0, northwest_ddeg.1),
                tec: northwest_tec,
            },
            south_east: TecPoint {
                point: Point::new(southeast_ddeg.0, southeast_ddeg.1),
                tec: southeast_tec,
            },
            south_west: TecPoint {
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
            north_east: TecPoint {
                point: Point::new(northeast_rad.0.to_degrees(), northeast_rad.1.to_degrees()),
                tec: northeast_tec,
            },
            north_west: TecPoint {
                point: Point::new(northwest_rad.0.to_degrees(), northwest_rad.1.to_degrees()),
                tec: northwest_tec,
            },
            south_east: TecPoint {
                point: Point::new(southeast_rad.0.to_degrees(), southeast_rad.1.to_degrees()),
                tec: southeast_tec,
            },
            south_west: TecPoint {
                point: Point::new(southwest_rad.0.to_degrees(), southwest_rad.1.to_degrees()),
                tec: southwest_tec,
            },
        }
    }

    /// Define a new [MapCell] from all 4 [TecPoint]s describing each corner at this [Epoch].
    pub fn from_cardinal_points(
        epoch: Epoch,
        north_east: TecPoint,
        north_west: TecPoint,
        south_east: TecPoint,
        south_west: TecPoint,
    ) -> Self {
        Self {
            epoch,
            north_east,
            north_west,
            south_east,
            south_west,
        }
    }

    /// Returns true if both [MapCell]s describe the same spatial ROI
    pub fn spatial_match(&self, rhs: &Self) -> bool {
        if self.north_east.point == rhs.north_east.point {
            if self.north_west.point == rhs.north_west.point {
                if self.south_east.point == rhs.south_east.point {
                    if self.south_west.point == rhs.south_west.point {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Returns true if both [MapCell]s describe the same point in time
    pub fn temporal_match(&self, rhs: &Self) -> bool {
        self.epoch == rhs.epoch
    }

    /// Returns true if both [MapCell]s describe the same spatial region at the same instant.
    pub fn spatial_temporal_match(&self, rhs: &Self) -> bool {
        self.spatial_match(rhs) && self.temporal_match(rhs)
    }

    /// Returns true if self is the northern neighbor of provided (rhs) [MapCell].
    pub fn is_northern_neighbor(&self, rhs: &Self) -> bool {
        rhs.north_east.point == self.south_east.point
            && rhs.north_west.point == self.south_west.point
    }

    pub fn is_northwestern_neighbor(&self, rhs: &Self) -> bool {
        false // TODO
    }

    pub fn is_northeastern_neighbor(&self, rhs: &Self) -> bool {
        false // TODO
    }

    /// Returns true if self is the southern neighbor of provided (rhs) [MapCell].
    pub fn is_southern_neighbor(&self, rhs: &Self) -> bool {
        rhs.south_east.point == self.north_east.point
            && rhs.south_west.point == self.north_west.point
    }

    pub fn is_southeastern_neighbor(&self, rhs: &Self) -> bool {
        false // TODO
    }

    pub fn is_southwestern_neighbor(&self, rhs: &Self) -> bool {
        false // TODO
    }

    /// Returns true if self is the easthern neighbor of provided (rhs) [MapCell].
    pub fn is_eastern_neighbor(&self, rhs: &Self) -> bool {
        rhs.north_west.point == self.north_east.point
            && rhs.south_west.point == self.south_east.point
    }

    /// Returns true if self is the westhern neighbor of provided (rhs) [MapCell].
    pub fn is_western_neighbor(&self, rhs: &Self) -> bool {
        rhs.north_east.point == self.north_west.point
            && rhs.south_east.point == self.south_west.point
    }

    /// Returns true if both cells are neighbors, meaning, they share two corners.
    pub fn is_neighbor(&self, rhs: &Self) -> bool {
        self.is_northern_neighbor(rhs)
            || self.is_northwestern_neighbor(rhs)
            || self.is_northeastern_neighbor(rhs)
            || self.is_southern_neighbor(rhs)
            || self.is_southwestern_neighbor(rhs)
            || self.is_southeastern_neighbor(rhs)
            || self.is_western_neighbor(rhs)
            || self.is_eastern_neighbor(rhs)
    }

    /// Returns true if this [MapCell] contains (wrapps) entirely the spatial ROI
    /// described by the provided (rhs) [MapCell].
    /// Meaning, rhs is fully contained within self.
    pub fn wrapps_entirely(&self, rhs: &Self) -> bool {
        self.contains(&Geometry::Rect(rhs.bounding_rect_degrees()))
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
        geo::Point(self.bounding_rect_degrees().center())
    }

    /// Returns borders of this [MapCell] expressed as a [Rect]angle, in decimal degrees.
    /// This is a direct conversion of this [MapCell] in terms of spatial dimensions,
    /// discarding the associated TEC values.
    pub fn bounding_rect_degrees(&self) -> Rect {
        Rect::new(self.south_west.point, self.north_east.point)
    }

    /// Returns geodesic perimeter (in meters) of this [MapCell].
    pub fn geodesic_perimeter(&self) -> f64 {
        self.bounding_rect_degrees().geodesic_perimeter()
    }

    /// Returns geodesic area (in squared meters) of this [MapCell].
    pub fn geodesic_area(&self) -> f64 {
        self.bounding_rect_degrees().geodesic_area_unsigned()
    }

    /// Returns true if following [Geometry], expressed in decimal degrees,
    /// is contained within this [MapCell].
    pub fn contains(&self, geometry: &Geometry<f64>) -> bool {
        self.bounding_rect_degrees().contains(geometry)
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

    /// Copies and updates the temporal instant
    pub fn with_epoch(mut self, epoch: Epoch) -> Self {
        self.epoch = epoch;
        self
    }

    /// Returns the (latitude, longitude) span of this [MapCell]
    /// as tuplet in degrees
    pub fn latitude_longitude_span_degrees(&self) -> (f64, f64) {
        (self.latitude_span_degrees(), self.longitude_span_degrees())
    }

    /// Returns latitude span of this [MapCell] in degrees
    pub fn latitude_span_degrees(&self) -> f64 {
        let borders = self.bounding_rect_degrees();
        borders.max().y - borders.min().y
    }

    /// Returns longitude span of this [MapCell] in degrees
    pub fn longitude_span_degrees(&self) -> f64 {
        let borders = self.bounding_rect_degrees();
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
    /// let tec = cell.spatial_tec_interp(center);
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
    /// let tec = cell.spatial_tec_interp(Point::new(0.5, 0.5));
    /// assert_eq!(tec.tecu(), 0.25);
    ///
    /// // SW boundary
    /// let tec = cell.spatial_tec_interp(Point::new(0.0, 0.0));
    /// assert_eq!(tec.tecu(), 1.0);
    ///
    /// // SWern point
    /// let tec = cell.spatial_tec_interp(Point::new(0.1, 0.1));
    /// assert_eq!(tec.tecu(), 0.81);
    ///
    /// // SWwern point
    /// let tec = cell.spatial_tec_interp(Point::new(0.01, 0.01));
    /// assert_eq!(tec.tecu(), 0.9801);
    /// ```
    pub fn spatial_tec_interp(&self, point: Point<f64>) -> Result<TEC, Error> {
        if !self.contains(&Geometry::Point(point)) {
            return Err(Error::OutsideSpatialBoundaries);
        }

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

        Ok(TEC::from_tecu(tecu))
    }

    /// Returns a stretched (either upscaled or downscaled, resized in dimension) ROI,
    /// by applying the interpolation equation on each corners.
    /// Although this operation may apply to any [MapCell], for best precision
    /// it is recommend to restrict the upscaling factor to small values (<2), otherwise,
    /// the [Cell3x3] should be prefered and will give more accurate results, by
    /// taking into account the neighboring cells.
    pub fn stretching_mut(&mut self, factor: f64) -> Result<(), Error> {
        if !factor.is_normal() {
            return Err(Error::InvalidStretchFactor);
        }

        // apply interpolation eq. at 4 coordinates
        let (north_east, north_west, south_east, south_west) = (
            Point::new(
                self.north_east.point.x() * factor,
                self.north_east.point.y() * factor,
            ),
            Point::new(
                self.north_west.point.x() * factor,
                self.north_west.point.y() * factor,
            ),
            Point::new(
                self.south_east.point.x() * factor,
                self.south_east.point.y() * factor,
            ),
            Point::new(
                self.south_west.point.x() * factor,
                self.south_west.point.y() * factor,
            ),
        );

        let (north_east, north_west, south_east, south_west) = (
            TecPoint {
                point: north_east,
                tec: self.spatial_tec_interp(north_east)?,
            },
            TecPoint {
                point: north_west,
                tec: self.spatial_tec_interp(north_west)?,
            },
            TecPoint {
                point: south_east,
                tec: self.spatial_tec_interp(south_east)?,
            },
            TecPoint {
                point: south_west,
                tec: self.spatial_tec_interp(south_west)?,
            },
        );

        self.north_east = north_east;
        self.north_west = north_west;
        self.south_east = south_east;
        self.south_west = south_west;

        Ok(())
    }

    /// Returns a stretched (either upscaled or downscaled, resized in dimension) ROI,
    /// by applying the interpolation equation on each corners.
    /// Although this operation may apply to any [MapCell], for best precision
    /// it is recommend to restrict the upscaling factor to small values (<2), otherwise,
    /// the [Cell3x3] should be prefered and will give more accurate results, by
    /// taking into account the neighboring cells.
    pub fn stretched(&self, factor: f64) -> Result<MapCell, Error> {
        let mut s = self.clone();
        s.stretching_mut(factor)?;
        Ok(s)
    }

    // /// Determines the northeastern cell amongst a grouping of 4
    // pub fn northeasternmost_cell4(cell1: &Self, cell2: &Self, cell3: &Self, cell4: &Self) -> Self {
    //     let min_x = cell1
    //         .point
    //         .x()
    //         .min(cell2.point.x())
    //         .min(cell3.point.x())
    //         .min(cell4.point.x());
    //     let min_y = cell1
    //         .point
    //         .y()
    //         .min(cell2.point.y())
    //         .min(cell3.point.y())
    //         .min(cell4.point.y());

    //     if cell1.point.x() == min_x && cell1.point.y() == min_y {
    //         *cell_1
    //     } else if cell2.point.x() == min_x && cell2.point.y() == min_y {
    //         *cell_2
    //     } else if cell3.point.x() == min_x && cell3.point.y() == min_y {
    //         *cell_3
    //     } else {
    //         *cell_4
    //     }
    // }

    // /// Determines the northwestern cell amongst a grouping of 4
    // pub fn northeasternmost_cell4(cell1: &Self, cell2: &Self, cell3: &Self, cell4: &Self) -> Self {
    //     let min_x = cell1
    //         .point
    //         .x()
    //         .min(cell2.point.x())
    //         .min(cell3.point.x())
    //         .min(cell4.point.x());
    //     let min_y = cell1
    //         .point
    //         .y()
    //         .min(cell2.point.y())
    //         .min(cell3.point.y())
    //         .min(cell4.point.y());

    //     if cell1.point.x() == min_x && cell1.point.y() == min_y {
    //         *cell_1
    //     } else if cell2.point.x() == min_x && cell2.point.y() == min_y {
    //         *cell_2
    //     } else if cell3.point.x() == min_x && cell3.point.y() == min_y {
    //         *cell_3
    //     } else {
    //         *cell_4
    //     }
    // }

    // /// Determines the southwestern cell amongst a grouping of 4
    // pub fn southwesternmost_cell4(cell1: &Self, cell2: &Self, cell3: &Self, cell4: &Self) -> Self {
    //     let min_x = cell1
    //         .point
    //         .x()
    //         .min(cell2.point.x())
    //         .min(cell3.point.x())
    //         .min(cell4.point.x());
    //     let min_y = cell1
    //         .point
    //         .y()
    //         .min(cell2.point.y())
    //         .min(cell3.point.y())
    //         .min(cell4.point.y());

    //     if cell1.point.x() == min_x && cell1.point.y() == min_y {
    //         *cell_1
    //     } else if cell2.point.x() == min_x && cell2.point.y() == min_y {
    //         *cell_2
    //     } else if cell3.point.x() == min_x && cell3.point.y() == min_y {
    //         *cell_3
    //     } else {
    //         *cell_4
    //     }
    // }

    // /// Determines the southwestern cell amongst a grouping of 4
    // pub fn southwesternmost_cell4(cell1: &Self, cell2: &Self, cell3: &Self, cell4: &Self) -> Self {
    //     let min_x = cell1
    //         .point
    //         .x()
    //         .min(cell2.point.x())
    //         .min(cell3.point.x())
    //         .min(cell4.point.x());
    //     let min_y = cell1
    //         .point
    //         .y()
    //         .min(cell2.point.y())
    //         .min(cell3.point.y())
    //         .min(cell4.point.y());

    //     if cell1.point.x() == min_x && cell1.point.y() == min_y {
    //         *cell_1
    //     } else if cell2.point.x() == min_x && cell2.point.y() == min_y {
    //         *cell_2
    //     } else if cell3.point.x() == min_x && cell3.point.y() == min_y {
    //         *cell_3
    //     } else {
    //         *cell_4
    //     }
    // }

    // /// Interpolate the TEC at 4 points described by provided [Rect]angle, returning a new [MapCell].
    // /// We use 4 neighboring [MapCell]s to apply the interpolation equation with the highest precision
    // /// on each corner of the [Rect]angle. This is to be used when the ROI does not align with the grid space.
    // pub fn interpolate_at_grouping4(
    //     &self,
    //     roi: Rect,
    //     neighbor_1: &Self,
    //     neighbor_2: &Self,
    //     neighbor_3: &Self,
    // ) -> Result<Self, Error> {
    //     let (
    //         (roi_ne_lat, roi_ne_long),
    //         (roi_se_lat, roi_se_long),
    //         (roi_sw_lat, roi_sw_long),
    //         (roi_nw_lat, roi_nw_long),
    //     ) = rectangle_to_cardinals(roi);

    //     // determines NE, SE, NW, NE cells conveniently
    //     // so we tolerate a random order (but they need to be neighboring cells)
    //     let ne_cell = Self::northeasternmost_cell4(self, neighbor_1, neighbor_2, neighbor_3);
    //     let se_cell = Self::southasternmost_cell4(self, neighbor_1, neighbor_2, neighbor_3);
    //     let nw_cell = Self::northwesternmost_cell4(self, neighbor_1, neighbor_2, neighbor_3);
    //     let ne_cell = Self::northeasternmost_cell4(self, neighbor_1, neighbor_2, neighbor_3);

    //     // verifies they are all neighboring cells
    //     if !nw_cell.is_western_neighbor(ne_cell) {
    //         return Err();
    //     }
    //     if !ne_cell.is_northern_neighbor(se_cell) {
    //         return Err();
    //     }
    //     if !se_cell.is_eastern_neighbor(sw_cell) {
    //         return Err();
    //     }
    //     if !nw_cell.is_northern_neighbor(sw_cell) {
    //         return Err();
    //     }
    // }

    // /// Merges two neighboring [MapCell]s forming a new (upscaled) [MapCell].
    // /// Both cells must be synchronous.
    // pub fn merge_neighbors(&self, rhs: &Self) -> Result<Self, Error> {
    //     if !self.temporal_match(rhs) {
    //         return Err(Error::TemporalMismatch);
    //     }

    //     // 1: determine the matching corners. The center
    //     // point of the matching line is to become the new center.
    //     // 2: form a new cell, of the boundary rect, interpolate
    //     // both TEC at the new center

    //     if self.is_northern_neighbor(rhs) {
    //         Ok(Self::default()) // TODO
    //     } else if self.is_southern_neighbor(rhs) {
    //         Ok(Self::default()) // TODO
    //     } else if self.is_western_neighbor(rhs) {
    //         Ok(Self::default()) // TODO
    //     } else if self.is_eastern_neighbor(rhs) {
    //         Ok(Self::default()) // TODO
    //     } else {
    //         Err(Error::SpatialMismatch)
    //     }
    // }

    // /// Interpolates two [MapCell]s that must describe the same area,
    // /// but a different point in time.
    // ///
    // /// ## Input
    // /// - epoch: [Epoch] of interpolation, must be temporally in between
    // /// [Self] and rhs.
    // pub fn temporally_interpolated(&self, epoch: Epoch, rhs: &Self) -> Result<Self, Error> {
    //     if !self.spatial_match(rhs) {
    //         return Err(Error::SpatialMismatch);
    //     }

    //     let (min_t, max_t) =
    //     (
    //         std::cmp::min(self.epoch, rhs.epoch),
    //         std::cmp::max(self.epoch, rhs.epoch),
    //     );

    //     if epoch < min_t && epoch > max_t {
    //         return Err(Error::InvalidTemporalPoint);
    //     }

    //     let (num_1, num_2, dt) = if self.epoch > rhs.epoch {
    //         (
    //             (self.epoch - epoch).to_seconds(),
    //             (epoch - rhs.epoch).to_seconds(),
    //             (self.epoch - rhs.epoch).to_seconds(),
    //         )
    //     } else {
    //         (
    //             (rhs.epoch - epoch).to_seconds(),
    //             (epoch - self.epoch).to_seconds(),
    //             (rhs.epoch - self.epoch).to_seconds(),
    //         )
    //     };

    //     let (ne_1, ne_2) = if self.epoch > rhs.epoch {
    //        (self.north_east.tec.tecu(), rhs.north_east.tec.tecu())
    //     } else {
    //        (rhs.north_east.tec.tecu(), self.north_east.tec.tecu())
    //     };
    //
    //     let (nw_1, nw_2) = if self.epoch > rhs.epoch {
    //        (self.north_west.tec.tecu(), rhs.north_west.tec.tecu())
    //     } else {
    //        (rhs.north_west.tec.tecu(), self.north_west.tec.tecu())
    //     };

    //     let (se_1, se_2) = if self.epoch > rhs.epoch {
    //        (self.south_east.tec.tecu(), rhs.south_east.tec.tecu())
    //     } else {
    //        (rhs.south_east.tec.tecu(), self.south_east.tec.tecu())
    //     };
    //
    //     let (sw_1, sw_2) = if self.epoch > rhs.epoch {
    //        (self.south_west.tec.tecu(), rhs.south_west.tec.tecu())
    //     } else {
    //        (rhs.south_west.tec.tecu(), self.south_west.tec.tecu())
    //     };

    //     Ok(Self {
    //         epoch,
    //         north_east: TecPoint {
    //             point: self.north_east.point,
    //             tec: TEC::from_tecu(
    //                 num_1 * ne_1 /dt + num_2 * ne_2 /dt
    //             ),
    //         },
    //         north_west: TecPoint {
    //             point: self.north_west.point,
    //             tec: TEC::from_tecu(
    //                 num_1 * nw_1 /dt + num_2 * nw_2 /dt
    //             ),
    //         },
    //         south_east: TecPoint {
    //             point: self.south_east.point,
    //             tec: TEC::from_tecu(
    //                 num_1 * se_1 /dt + num_2 * se_2 /dt
    //             ),
    //         },
    //         south_west: TecPoint {
    //             point: self.south_west.point,
    //             tec: TEC::from_tecu(
    //                 num_1 * sw_1 /dt + num_2 * sw_2 /dt
    //             ),
    //         },
    //     })
    // }

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
    /// let central_tec0 = cell0.spatial_tec_interp(center);
    /// assert_eq!(central_tec0.tecu(), 1.0);
    ///
    /// // verify central point value
    /// let central_tec1 = cell1.spatial_tec_interp(center);
    /// assert_eq!(central_tec1.tecu(), 1.0);
    ///
    /// // spatial + temporal interpolation
    /// // <!> outside sampling interval
    /// assert!(cell0.temporal_spatial_tec_interp(t_nok, center, &cell1).is_none());
    ///
    /// // spatial + temporal interpolation
    /// let tec = cell0.temporal_spatial_tec_interp(t_ok, center, &cell1).unwrap();
    /// assert_eq!(tec.tecu(), 1.0);
    /// ```
    pub fn temporal_spatial_tec_interp(
        &self,
        epoch: Epoch,
        coordinates: Point<f64>,
        rhs: &Self,
    ) -> Result<TEC, Error> {
        // interpolate at exact coordinates
        let (tecu_0, tecu_1) = (
            self.spatial_tec_interp(coordinates)?.tecu(),
            rhs.spatial_tec_interp(coordinates)?.tecu(),
        );

        if epoch >= self.epoch && epoch < rhs.epoch {
            // forward
            let dt = (rhs.epoch - self.epoch).to_seconds();

            let tecu = (rhs.epoch - epoch).to_seconds() / dt * tecu_0
                + (epoch - self.epoch).to_seconds() / dt * tecu_1;

            Ok(TEC::from_tecu(tecu))
        } else if epoch >= rhs.epoch && epoch < self.epoch {
            // backwards
            let dt = (self.epoch - rhs.epoch).to_seconds();

            let tecu = (self.epoch - epoch).to_seconds() / dt * tecu_1
                + (epoch - rhs.epoch).to_seconds() / dt * tecu_0;

            Ok(TEC::from_tecu(tecu))
        } else {
            Err(Error::TemporalMismatch)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::prelude::{Epoch, Geometry, Point, Unit, TEC};

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

        let interpolated = cell.spatial_tec_interp(center).unwrap_or_else(|e| {
            panic!("should have been feasible: {}", e);
        });

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

            let interpolated = cell
                .spatial_tec_interp(point)
                .unwrap_or_else(|e| {
                    panic!("should have been feasible! {}", e);
                })
                .tecu();

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
                .temporal_spatial_tec_interp(t_nok, center, &cell1)
                .is_err(),
            "interpolation is temporally incorrect!"
        );

        let tec = cell0
            .temporal_spatial_tec_interp(t_ok, center, &cell1)
            .unwrap_or_else(|e| {
                panic!("should have been feasible! {}", e);
            });

        assert_eq!(tec.tecu(), 1.0);
    }
}
