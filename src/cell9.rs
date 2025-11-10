use std::collections::HashMap;

use geo::{Contains, GeodesicArea, Geometry, Point, Rect};

use crate::prelude::{Cardinal, Epoch, Error, MapCell, TEC};

/// A structure holding 9 synchronous neighboring [MapCells]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Cell9 {
    /// The central [MapCell]
    pub central: MapCell,

    /// 8 Neighboring [MapCells]
    pub neighbors: HashMap<Cardinal, MapCell>,
}

impl Cell9 {
    /// Returns true if this [Cell9] definition is "complete"
    pub(crate) fn is_complete(&self) -> bool {
        self.neighbors.len() == 8
    }

    /// Builds a new updated [Cell9] with updated central cell.
    pub fn with_central_cell(mut self, center: MapCell) -> Self {
        self.central = center;
        self
    }

    /// Builds a new [Cell9] conveniently from 9 [MapCells], correctly
    /// electing the central ROI, and the 8 neighbors.
    /// Returns None if no central ROI exists or could not determine 8 neighbors.
    pub fn from_slice(cells: [MapCell; 9]) -> Option<Self> {
        for i in 0..9 {
            // determine whether the ith cell is the potential center
            let mut all_neighbors = true;
            // must be neighbor with all other 8 ROIs
            for j in 0..9 {
                if j != i {
                    if !cells[i].is_neighbor(&cells[j]) {
                        all_neighbors = false;
                        break;
                    }
                }
            }

            if all_neighbors {
                let mut ret = Self::default().with_central_cell(cells[i]);

                // i is the central ROI, order other ROIs correctly & exit
                for j in 0..9 {
                    if j != i {
                        if cells[j].is_northwestern_neighbor(&cells[i]) {
                            ret.neighbors.insert(Cardinal::NorthWest, cells[j]);
                        } else if cells[j].is_northeastern_neighbor(&cells[i]) {
                            ret.neighbors.insert(Cardinal::NorthEast, cells[j]);
                        } else if cells[j].is_northern_neighbor(&cells[i]) {
                            ret.neighbors.insert(Cardinal::North, cells[j]);
                        } else if cells[j].is_southwestern_neighbor(&cells[i]) {
                            ret.neighbors.insert(Cardinal::SouthWest, cells[j]);
                        } else if cells[j].is_southeastern_neighbor(&cells[i]) {
                            ret.neighbors.insert(Cardinal::SouthEast, cells[j]);
                        } else if cells[j].is_southern_neighbor(&cells[i]) {
                            ret.neighbors.insert(Cardinal::South, cells[j]);
                        }
                    }
                }

                if ret.is_complete() {
                    return Some(ret);
                }
            }
        }

        None
    }

    /// Upscale or downscale the central ROI of this [Cell9] by a fractional positive scalar.
    pub fn upscale(&self, factor: f64) -> Result<MapCell, Error> {
        // interpolate the 4 new central coordinates
        // interpolate using the 4 neighbors
        // take the average of both for each 4 coordinntes
    }

    pub fn downscale(&self, factor: f64) -> Result<MapCell, Error> {
        // interpolate the central ROI
        self.central.downscale(factor)
    }
}
