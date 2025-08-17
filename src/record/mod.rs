mod formatting;
mod parsing;

use std::collections::{btree_map::Iter, BTreeMap};

use itertools::Itertools;

use crate::prelude::{Epoch, Key, MapCell, TEC};

/// IONEX [Record] contains [MapCell]s in chronological order.
#[derive(Clone, Debug, Default, PartialEq)]
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

    /// Obtain [TEC] value from IONEX [Record], at specified spatial and temporal coordinates that must exist.
    /// This is an indexing method, not an interpolation method.  
    /// For interpolation, use the [MapCell] API.
    pub fn get(&self, key: &Key) -> Option<&TEC> {
        self.map.get(key)
    }

    /// Obtain mutable [TEC] reference from IONEX [Record], at specified spatial and temporal coordinates that must exist.
    /// This is an indexing method, not an interpolation method.  
    /// For interpolation, use the [MapCell] API.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut TEC> {
        self.map.get_mut(key)
    }

    /// Collect a IONEX [Record] from a list of [MapCell].
    pub fn from_map_cells(
        fixed_altitude_km: f64,
        min_latitude_ddeg: f64,
        max_latitude_ddeg: f64,
        min_longitude_ddeg: f64,
        max_longitude_ddeg: f64,
        cells: &[MapCell],
    ) -> Self {
        let mut map = Default::default();
        //let (mut new_lat, mut new_long) = (true, true);
        //let (mut prev_lat, mut prev_long) = (0.0_f64, 0.0_f64);

        for cell in cells.iter() {
            // SW bound is always introduced
            let sw_key = Key::from_decimal_degrees_km(
                cell.epoch,
                cell.south_west.point.y(),
                cell.south_west.point.x(),
                fixed_altitude_km,
            );

            // SE bound is introduced for any but first cell
            // if !new_lat {

            // }

            //if cell.north_east.point.y() == max_latitude {
            //}
        }

        Self { map }
    }

    /// Obtain [Epoch]s Iterator in chronological order.
    pub fn epochs_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.map.keys().map(|k| k.epoch).unique())
    }

    /// Returns first [Epoch] in chronological order
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epochs_iter().nth(0)
    }
}
