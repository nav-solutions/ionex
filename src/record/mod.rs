mod formatting;
mod parsing;

use std::collections::{
    btree_map::{Iter, IterMut, Keys},
    BTreeMap, HashMap,
};

use itertools::Itertools;
use std::str::FromStr;
use thiserror::Error;

use geo::{Geometry, Polygon, Rect};

use crate::{
    prelude::{Comments, Epoch, Header, Key, MapCell, TEC},
    quantized::Quantized,
};

/// IONEX [Record] contains [MapCell]s in chronological order.
#[derive(Clone, Debug, Default)]
pub struct Record {
    pub(crate) map: BTreeMap<Epoch, MapCell>,
}

impl Record {
    /// Insert a new [MapCell] into IONEX [Record]
    pub fn insert(&mut self, epoch: Epoch, cell: MapCell) {
        self.map.insert(epoch, cell);
    }

    /// Obtain [Record] iterator.
    pub fn iter(&self) -> Iter<'_, Epoch, MapCell> {
        self.map.iter()
    }

    /// Obtain [MapCell] Iterator (starting on northern eastern most to southern western most cell), at this point in time.
    pub fn synchronous_iter(&self, epoch: Epoch) -> Box<dyn Iterator<Item = MapCell> + '_> {
        Box::new(
            self.iter()
                .filter_map(move |(k, v)| if *k == epoch { Some(*v) } else { None }),
        )
    }

    /// Obtain mutable [Record] iterator.
    pub fn iter_mut(&mut self) -> IterMut<'_, Epoch, MapCell> {
        self.map.iter_mut()
    }

    /// Obtain [MapCell] local region from IONEX [Record], at specified point in time.
    pub fn get(&self, epoch: &Epoch) -> Option<&MapCell> {
        self.map.get(epoch)
    }

    /// Obtain mutable [MapCell] reference from IONEX [Record], as specified point in time.
    pub fn get_mut(&mut self, epoch: &Epoch) -> Option<&mut MapCell> {
        self.map.get_mut(epoch)
    }

    /// Obtain [Epoch]s Iterator in chronological order.
    pub fn epochs_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.map.keys().unique().map(|k| *k))
    }

    /// Returns first [Epoch] in chronological order
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epochs_iter().nth(0)
    }

    /// Obtain the [MapCell] that contains following [Geometry] completely.
    /// ## Input
    /// - point: coordinates as [Point]
    /// ## Returns
    /// - None if map grid does not cointain these coordinates
    /// - MapCell that wraps these coordinates
    pub fn wrapping_map_cell(&self, geometry: &Geometry<f64>) -> Option<MapCell> {
        let first_epoch = self.first_epoch()?;

        for cell in self.synchronous_iter(first_epoch) {
            if cell.contains(&geometry) {
                return Some(cell);
            }
        }

        None
    }

    /// Synchronous [MapCell] Iterators (starting on northern eastern most to souther western most cell)

    /// Obtain interpolated [TEC] value at any coordinates and time,
    /// using temporal 2D interpolation formula.
    /// Coordinates must be specified in degrees
    /// ## Inputs
    /// - instant as [Epoch] which should be within
    /// the timeframe of this IONEX for the results to be correct.
    /// - latitude angle in degrees which should lie within the map borders
    /// - longitude angle in degrees which should lie within the map borders
    /// ## Returns
    /// - None if [Epoch] does not exit
    /// - Zero if map grid is not valid at this [Epoch]
    /// - Interpolated value otherwise
    pub fn get_interpolated(
        &self,
        epoch: Epoch,
        latitude_ddeg: f64,
        longitude_ddeg: f64,
    ) -> Option<f64> {
        let mut ret = None;

        ret
    }
}
