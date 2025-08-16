use geo::{coord, Area, Contains, Coord, CoordNum, Geometry, Point, Rect};

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
    /// Define a new [MapCell] from all 4 [MapPoint]s describing each corner at this [Epoch].
    pub fn from_corners(
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

    /// Define a new [MapCell] from the bounding [Rect]angle
    /// describing the Northeastern most (upper left) point
    /// and Southwestern most (lower right) point observed at this [Epoch].
    /// NB: the TEC values are null for the NW and SE point, and should be manually defined.
    pub fn from_ne_sw_borders(epoch: Epoch, north_east: MapPoint, south_west: MapPoint) -> Self {
        Self {
            epoch,
            north_west: MapPoint {
                tec: Default::default(),
                point: Point::new(south_west.point.x(), north_east.point.y()),
            },
            south_east: MapPoint {
                tec: Default::default(),
                point: Point::new(north_east.point.x(), south_west.point.y()),
            },
            north_east,
            south_west,
        }
    }

    /// Returns area of this [MapCell]
    pub fn area(&self) -> f64 {
        self.borders().unsigned_area()
    }

    /// Returns borders of this [MapCell] expressed as a [Rect]angle
    pub fn borders(&self) -> Rect {
        Rect::new(self.south_west.point, self.north_east.point)
    }

    /// Returns true if following [Geometry] is contained within this [MapCell].
    pub fn contains(&self, geometry: &Geometry<f64>) -> bool {
        self.borders().contains(geometry)
    }

    /// Copies and updates the Northeast TEC component
    pub fn with_northeast_tec(mut self, tec: TEC) -> Self {
        self.north_east.tec = tec;
        self
    }

    /// Copies and updates the Northwest TEC component
    pub fn with_northwest_tec(mut self, tec: TEC) -> Self {
        self.north_west.tec = tec;
        self
    }

    /// Copies and updates the Southeast TEC component
    pub fn with_southeast_tec(mut self, tec: TEC) -> Self {
        self.south_east.tec = tec;
        self
    }

    /// Copies and updates the Southwest TEC component
    pub fn with_southwest_tec(mut self, tec: TEC) -> Self {
        self.south_west.tec = tec;
        self
    }

    /// Returns latitude width of this [MapCell] in degrees
    pub fn latitude_span_degrees(&self) -> f64 {
        let borders = self.borders();
        borders.max().y - borders.min().y
    }

    /// Returns longitude width of this [MapCell] in degrees
    pub fn longitude_span_degrees(&self) -> f64 {
        let borders = self.borders();
        borders.max().x - borders.min().x
    }

    /// Obtain interpolated [TEC] value using standard planary interpolation equation.
    /// [MapCell::contains] should be true for this to be correct.
    pub fn interpolate(&self, point: Point<f64>) -> TEC {
        let borders = self.borders();
        let (lambda_1, lambda_2) = (borders.min().x, borders.max().x);
        let (phi_1, phi_2) = (borders.min().y, borders.max().y);

        let x = (point.x() - lambda_1) / (lambda_2 - lambda_1);
        let y = (point.y() - phi_1) / (phi_2 - phi_1);

        let tecu = self.south_west.tec.tecu() * (1.0 - x) * (1.0 - y)
            + self.south_east.tec.tecu() * (1.0 - y)
            + self.north_west.tec.tecu() * y * (1.0 - x)
            + self.north_east.tec.tecu() * x * y;

        TEC::from_tecu(tecu)
    }
}
