use std::collections::HashMap;

use geo::{Contains, GeodesicArea, Geometry, Point, Rect};

use crate::prelude::{Epoch, Error, MapCell, TEC};

// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub enum Cardinal {
//     /// NE [Cardinal]
//     NorthEast,
//
//     /// N [Cardinal]
//     North,
//
//     /// NW [Cardinal]
//     NorthWest,
//
//     /// W [Cardinal]
//     West,
//
//     /// SW [Cardinal]
//     SouthWest,
//
//     /// S [Cardinal]
//     South,
//
//     /// SE [Cardinal]
//     SouthEast,
//
//     /// E [Cardinal]
//     East,
// }

/// A synchronous 3x3 ROI made of a central [MapCell] element and 8 neighboring [MapCell]s.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cell3x3 {
    /// The central [MapCell]
    pub center: MapCell,

    /// The northeastern neighboring [MapCell]
    pub northeast: MapCell,

    /// The northern neighboring [MapCell]
    pub north: MapCell,

    /// The northwestern neighboring [MapCell]
    pub northwest: MapCell,

    /// The western neighboring [MapCell]
    pub west: MapCell,

    /// The southwestern neighboring [MapCell]
    pub southwest: MapCell,

    /// The southern neighboring [MapCell]
    pub south: MapCell,

    /// The southeastern neighboring [MapCell]
    pub southeast: MapCell,

    /// The eastern neighboring [MapCell]
    pub east: MapCell,
}

impl Cell3x3 {
    /// Returns true if both [Cell3x3] cells represent the same spatial region
    pub fn spatial_match(&self, rhs: &Self) -> bool {
        self.center.spatial_match(&rhs.center)
            && self.northeast.spatial_match(&rhs.northeast)
            && self.north.spatial_match(&rhs.north)
            && self.northwest.spatial_match(&rhs.northwest)
            && self.west.spatial_match(&rhs.west)
            && self.southwest.spatial_match(&rhs.southwest)
            && self.south.spatial_match(&rhs.south)
            && self.southeast.spatial_match(&rhs.southeast)
            && self.east.spatial_match(&rhs.east)
    }

    /// Returns true if both [Cell3x3] cells are synchronous.
    pub fn temporal_match(&self, rhs: &Self) -> bool {
        self.center.temporal_match(&rhs.center)
            && self.northeast.temporal_match(&rhs.northeast)
            && self.north.temporal_match(&rhs.north)
            && self.northwest.temporal_match(&rhs.northwest)
            && self.west.temporal_match(&rhs.west)
            && self.southwest.temporal_match(&rhs.southwest)
            && self.south.temporal_match(&rhs.south)
            && self.southeast.temporal_match(&rhs.southeast)
            && self.east.temporal_match(&rhs.east)
    }

    /// Returns true if both [Cell3x3] cells represent the same spatial region
    /// at the same instant.
    pub fn spatial_temporal_match(&self, rhs: &Self) -> bool {
        self.spatial_match(rhs) && self.temporal_match(rhs)
    }

    /// Builds a new [Cell3x3] updated in time
    pub fn with_epoch(mut self, epoch: Epoch) -> Self {
        self.center.epoch = epoch;
        self.northeast.epoch = epoch;
        self.north.epoch = epoch;
        self.northwest.epoch = epoch;
        self.east.epoch = epoch;
        self.southeast.epoch = epoch;
        self.south.epoch = epoch;
        self.southwest.epoch = epoch;
        self.west.epoch = epoch;
        self
    }

    /// Builds a new [Cell3x3] with updated central cell, which must be synchronous.
    pub fn with_central_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.center.epoch {
            self.center = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated western cell, which must be synchronous.
    pub fn with_western_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.west.epoch {
            self.west = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated eastern cell, which must be synchronous.
    pub fn with_eastern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.east.epoch {
            self.east = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated southern cell, which must be synchronous.
    pub fn with_southern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.south.epoch {
            self.south = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated northern cell, which must be synchronous.
    pub fn with_northern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.north.epoch {
            self.north = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated southwestern cell, which must be synchronous.
    pub fn with_southwestern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.southwest.epoch {
            self.southwest = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated southeastern cell, which must be synchronous.
    pub fn with_southeastern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.southeast.epoch {
            self.southeast = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated northwestern cell, which must be synchronous.
    pub fn with_northwestern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.northwest.epoch {
            self.northwest = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] with updated northeastern cell, which must be synchronous.
    pub fn with_northeastern_cell(mut self, cell: MapCell) -> Result<Self, Error> {
        if cell.epoch == self.northeast.epoch {
            self.northeast = cell;
            Ok(self)
        } else {
            Err(Error::TemporalMismatch)
        }
    }

    /// Builds a new [Cell3x3] from a slice of 9 unordered [MapCell]s, by electing a central element (if feasible)
    /// and the neighboring cells, which must all be synchronous.
    pub fn from_slice(cells: [MapCell; 9]) -> Option<Self> {
        for i in 0..9 {
            // determine whether the ith cell is the potential center

            // must be neighbor with all other 8 ROIs and must be synchronous
            let mut all_synchronous_neighbors = true;

            for j in 0..9 {
                if j != i {
                    if !cells[i].is_neighbor(&cells[j]) {
                        all_synchronous_neighbors = false;
                        break;
                    }
                    if !cells[i].temporal_match(&cells[j]) {
                        all_synchronous_neighbors = false;
                        break;
                    }
                }
            }

            if all_synchronous_neighbors {
                let mut count = 0;

                let mut ret = Self::default()
                    .with_epoch(cells[i].epoch)
                    .with_central_cell(cells[i])
                    .unwrap(); // infaillible

                // i is the central ROI, order other ROIs correctly & exit
                for j in 0..9 {
                    if j != i {
                        if cells[j].is_northwestern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_northwestern_cell(cells[j]).unwrap();
                        // infaillible
                        } else if cells[j].is_northeastern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_northeastern_cell(cells[j]).unwrap();
                        // infaillible
                        } else if cells[j].is_northern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_northern_cell(cells[j]).unwrap(); // infaillible
                        } else if cells[j].is_southwestern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_southwestern_cell(cells[j]).unwrap();
                        // infaillible
                        } else if cells[j].is_southeastern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_southeastern_cell(cells[j]).unwrap();
                        // infaillible
                        } else if cells[j].is_southern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_southern_cell(cells[j]).unwrap(); // infaillible
                        } else if cells[j].is_eastern_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_eastern_cell(cells[j]).unwrap(); // infaillible
                        } else if cells[j].is_western_neighbor(&cells[i]) {
                            count += 1;
                            ret = ret.with_western_cell(cells[j]).unwrap(); // infaillible
                        }
                    }
                }

                if count == 8 {
                    // 3x3 roi completed
                    return Some(ret);
                }
            }
        }

        None
    }

    /// Returns a stretched (spatially upscaled or downscaled) [MapCell] by
    /// stretching the central element and taking the relative neighboring values into
    /// account.
    pub fn stretched(&self, factor: f64) -> Result<MapCell, Error> {
        Err(Error::OutsideSpatialBoundaries) // TODO
    }
}
