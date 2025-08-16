mod formatting;
mod parsing;

use std::collections::{
    btree_map::{Iter, Keys},
    BTreeMap, HashMap,
};
use std::str::FromStr;
use thiserror::Error;

use crate::prelude::{Comments, Epoch, Header, Key, Quantized, TEC};

/// IONEX [Record] describes [TEC] values at specific
/// coordinates and time
#[derive(Clone, Debug, Default)]
pub struct Record {
    map: BTreeMap<Key, TEC>,
}

impl Record {
    /// Insert a new [TEC] value into IONEX [Record]
    pub fn insert(&mut self, key: Key, tec: TEC) {
        self.map.insert(key, tec);
    }

    pub fn iter(&self) -> Iter<'_, Key, TEC> {
        self.map.iter()
    }

    /// Obtain [TEC] value from IONEX [Record], at specified
    /// coordinates and time, if it exists.
    pub fn get(&self, key: &Key) -> Option<&TEC> {
        self.map.get(key)
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
        Box::new(self.map.keys().map(|k| k.epoch))
    }
}
