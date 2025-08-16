use geo::{coord, Area, Contains, Coord, CoordNum, Geometry, Point, Rect};

use crate::prelude::TEC;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPoint {
    /// [Point]
    pub point: Point<f64>,

    /// TEC
    pub tec: TEC,
}

impl Default for MapPoint {
    /// Builds a default [MapCell] of null width with null central value.
    fn default() -> Self {
        Self {
            point: Default::default(),
            tec: Default::default(),
        }
    }
}

/// [MapCell] describing a small region.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapCell {
    /// North East [MapPoint]
    pub north_east: MapPoint,

    /// North West [MapPoint]
    pub north_west: MapPoint,

    /// South East [MapPoint]
    pub south_east: MapPoint,

    /// South West [MapPoint]
    pub south_west: MapPoint,
}

impl Default for MapCell {
    /// Builds a default unitary [MapCell] of with unitary width and null central TEC.
    fn default() -> Self {
        Self {
            north_east: MapPoint {
                point: Point::new(1.0, 1.0),
                tec: Default::default(),
            },
            north_west: MapPoint {
                point: Point::new(0.0, 1.0),
                tec: Default::default(),
            },
            south_east: MapPoint {
                point: Point::new(1.0, 0.0),
                tec: Default::default(),
            },
            south_west: MapPoint {
                point: Point::new(0.0, 0.0),
                tec: Default::default(),
            },
        }
    }
}

impl MapCell {
    /// Define a new [MapCell] from all 4 [MapPoint]s describing each corner.
    pub fn from_corners(
        north_east: MapPoint,
        north_west: MapPoint,
        south_east: MapPoint,
        south_west: MapPoint,
    ) -> Self {
        Self {
            north_east,
            north_west,
            south_east,
            south_west,
        }
    }

    /// Copies and updates the North East TEC component
    pub fn with_ne_tec(mut self, tec: TEC) -> Self {
        self.north_east.tec = tec;
        self
    }

    /// Copies and updates the North West TEC component
    pub fn with_nw_tec(mut self, tec: TEC) -> Self {
        self.north_west.tec = tec;
        self
    }

    /// Copies and updates the South East TEC component
    pub fn with_se_tec(mut self, tec: TEC) -> Self {
        self.south_east.tec = tec;
        self
    }

    /// Copies and updates the South West TEC component
    pub fn with_sw_tec(mut self, tec: TEC) -> Self {
        self.south_west.tec = tec;
        self
    }

    /// Define a new [MapCell] from the bounding [Rect]angle
    /// describing the Northern Eastern most (upper left) point
    /// and Southern Western most (lower right) point.
    /// NB: the TEC values are null for the NW and SE point, and should be manually defined.
    pub fn from_ne_sw_borders(north_east: MapPoint, south_west: MapPoint) -> Self {
        Self {
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

    /// Stretch this [MapCell] into a newer [MapCell], refer to [Self::stretch_mut] for more information.
    pub fn stretch(&self, factor: f64) -> Self {
        let mut s = self.clone();
        s.stretch_mut(factor);
        s
    }

    /// Stretch this [MapCell] into a newer [MapCell].
    pub fn stretch_mut(&mut self, factor: f64) {}

    /// Returns latitude width of this [MapCell] in degrees
    pub fn latitude_width_degrees(&self) -> f64 {
        let borders = self.borders();
        borders.max().y - borders.min().y
    }

    /// Returns longitude width of this [MapCell] in degrees
    pub fn longitude_width_degrees(&self) -> f64 {
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
