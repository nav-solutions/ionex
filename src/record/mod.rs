mod formatting;
mod parsing;

use std::collections::{
    btree_map::{Iter, IterMut, Keys},
    BTreeMap, HashMap,
};

use itertools::Itertools;
use std::str::FromStr;
use thiserror::Error;

use geo::Rect;

use crate::prelude::{Comments, Epoch, Header, Key, MapCell, Quantized, TEC};

/// IONEX [Record] describes [TEC] values at specific
/// coordinates and time
#[derive(Clone, Debug, Default)]
pub struct Record {
    pub(crate) map: BTreeMap<Key, TEC>,
}

impl Record {
    /// Insert a new [TEC] value into IONEX [Record]
    pub fn insert(&mut self, key: Key, tec: TEC) {
        self.map.insert(key, tec);
    }

    /// Obtain [Record] iterator.
    pub fn iter(&self) -> Iter<'_, Key, TEC> {
        self.map.iter()
    }

    /// Obtain mutable [Record] iterator.
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, TEC> {
        self.map.iter_mut()
    }

    /// Obtain [TEC] value from IONEX [Record], at specified
    /// coordinates and time, which must exist. If you want to obtain
    /// interpolated TEC values at _any_ coordinates, you should
    /// obtain a [MapCell] using other methods available, for example
    /// - [Self::surface_cells_iter]
    /// - [Self::surface_cell_at]
    pub fn get(&self, key: &Key) -> Option<&TEC> {
        self.map.get(key)
    }

    /// Obtain [TEC] value at any coordinates and time, by applying 2D interpolation. Coordinates must be specified in degrees and kilometers.
    /// ## Inputs
    /// - instant as [Epoch] which must be defined
    /// - latitude angle in degrees
    /// - longitude angle in degrees
    /// - altitude in kilometers
    /// ## Returns
    /// - None if [Epoch] does not exit
    /// - Zero if map grid is not valid at this [Epoch]
    /// - Interpolated value otherwise
    pub fn get_interpolated(
        &self,
        epoch: Epoch,
        latitude_ddeg: f64,
        longitude_ddeg: f64,
        altitude_km: f64,
    ) -> Option<f64> {
        let mut ret = None;

        ret
    }

    /// Obtain mutable [TEC] reference from IONEX [Record], as specified
    /// coordinates and time, if it exists.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut TEC> {
        self.map.get_mut(key)
    }

    /// Obtain Iterator over individual indexing [Key]s
    pub fn keys(&self) -> Keys<'_, Key, TEC> {
        self.map.keys()
    }

    /// Obtain [Epoch]s Iterator in chronological order.
    pub fn epochs_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.map.keys().map(|k| k.epoch).unique())
    }

    /// Returns first [Epoch] in chronological order
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epochs_iter().nth(0)
    }

    /// Obtain a [Rect]angle grid Iterator, of desired latitude
    /// and longitude widths, both expressed in degrees.
    /// ## Inputs
    /// - cell
    pub fn surface_cell_iter(
        &self,
        cell_lat_width_deg: f64,
        cell_long_width_deg: f64,
    ) -> Box<dyn Iterator<Item = MapCell>> {
        Box::new([].into_iter())
    }

    /// Obtain synchronous [MapCell] Iterator of desired latitude and longitude width (both expressed in degrees).
    pub fn synchronous_surface_cell_iter(
        &self,
        epoch: Epoch,
        cell_lat_width_deg: f64,
        cell_long_width_deg: f64,
    ) -> Box<dyn Iterator<Item = MapCell>> {
        Box::new([].into_iter())
    }
}
