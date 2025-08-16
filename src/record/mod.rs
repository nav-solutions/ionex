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
    pub(crate) map: BTreeMap<Key, TEC>,
}

impl Record {
    /// Insert new [TEC] value into IONEX [Record]
    pub fn insert(&mut self, key: Key, tec: TEC) {
        self.map.insert(key, tec);
    }

    /// Obtain [Record] iterator.
    pub fn iter(&self) -> Iter<'_, Key, TEC> {
        self.map.iter()
    }

    /// Obtain mutable [Record] iterator.
    pub fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (Key, &mut TEC)> + '_> {
        Box::new(self.map.iter_mut().map(|(k, v)| (*k, v)))
    }

    /// Obtain [Record] Iterator at specific point in time
    pub fn synchronous_iter(&self, epoch: Epoch) -> Box<dyn Iterator<Item = (Key, TEC)> + '_> {
        Box::new(self.iter().filter_map(move |(k, v)| {
            if k.epoch == epoch {
                Some((*k, *v))
            } else {
                None
            }
        }))
    }

    /// Obtain mutable synchronous [Record] iterator
    pub fn synchronous_iter_mut(
        &mut self,
        epoch: Epoch,
    ) -> Box<dyn Iterator<Item = (Key, &mut TEC)> + '_> {
        Box::new(self.iter_mut().filter_map(
            move |(k, v)| {
                if k.epoch == epoch {
                    Some((k, v))
                } else {
                    None
                }
            },
        ))
    }

    /// Obtain [TEC] value from IONEX [Record], at specified spatial and temporal point that must exist.
    pub fn get(&self, key: &Key) -> Option<&TEC> {
        self.map.get(key)
    }

    /// Obtain mutable [TEC] reference from IONEX [Record], at specified spatial and temporal point that must exist.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut TEC> {
        self.map.get_mut(key)
    }

    /// Obtain [Epoch]s Iterator in chronological order.
    pub fn epochs_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.map.keys().unique().map(|k| k.epoch))
    }

    /// Returns first [Epoch] in chronological order
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epochs_iter().nth(0)
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
